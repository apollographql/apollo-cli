use std::env;
use std::fmt::Debug;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::{Installer, InstallerError};

pub fn add_binary_to_path(installer: &Installer) -> Result<(), InstallerError> {
    for shell in get_available_shells() {
        let source_cmd = shell.source_string(installer)?;
        let script = shell.env_script();
        script.write(installer)?;

        for rc in shell.update_rcs() {
            if !rc.is_file() || !fs::read_to_string(&rc)?.contains(&source_cmd) {
                log::debug!("updating {}", &rc.display());
                let mut dest_file = fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(&rc)?;
                writeln!(&mut dest_file, "{}", &source_cmd)?;
                dest_file.sync_data()?;
            }
        }
    }

    Ok(())
}

fn get_available_shells() -> impl Iterator<Item = Shell> {
    let shells: Vec<Shell> = if cfg!(test) {
        vec![]
    } else {
        vec![Box::new(Posix), Box::new(Bash), Box::new(Zsh)]
    };
    shells.into_iter().filter(|sh| sh.does_exist())
}

type Shell = Box<dyn UnixShell>;

#[derive(Debug, PartialEq)]
pub struct ShellScript {
    content: String,
    name: String,
}

impl ShellScript {
    pub fn write(&self, installer: &Installer) -> Result<(), InstallerError> {
        let home = installer.get_base_dir_path()?;
        let path_to_bin = installer.get_bin_dir_path()?;
        let bin_str = path_to_bin.to_str().ok_or(InstallerError::PathNotUnicode)?;
        let path = home.join(&self.name);
        let contents = &self.content.replace("{path_to_bin}", bin_str);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path.as_path())?;

        io::Write::write_all(&mut file, contents.as_bytes())?;

        file.sync_data()?;

        Ok(())
    }
}

trait UnixShell: Debug {
    // Detects if a shell "exists". Users have multiple shells, so an "eager"
    // heuristic should be used, assuming shells exist if any traces do.
    fn does_exist(&self) -> bool;

    // Gives all rcfiles of a given shell that we are concerned with.
    // Used primarily in checking rcfiles for cleanup.
    fn rcfiles(&self) -> Vec<PathBuf>;

    // Gives rcs that should be written to.
    fn update_rcs(&self) -> Vec<PathBuf>;

    // Writes the relevant env file.
    fn env_script(&self) -> ShellScript {
        ShellScript {
            name: "env".to_string(),
            content: include_str!("env.sh").to_string(),
        }
    }

    fn source_string(&self, installer: &Installer) -> Result<String, InstallerError> {
        Ok(format!(
            r#"source "{}/env""#,
            installer.get_base_dir_path()?.to_string_lossy()
        ))
    }
}

#[derive(Debug)]
struct Posix;

impl UnixShell for Posix {
    fn does_exist(&self) -> bool {
        true
    }

    fn rcfiles(&self) -> Vec<PathBuf> {
        match crate::get_home_dir_path().ok() {
            Some(dir) => vec![dir.join(".profile")],
            _ => vec![],
        }
    }

    fn update_rcs(&self) -> Vec<PathBuf> {
        // Write to .profile even if it doesn't exist. It's the only rc in the
        // POSIX spec so it should always be set up.
        self.rcfiles()
    }
}

#[derive(Debug)]
struct Bash;

impl UnixShell for Bash {
    fn does_exist(&self) -> bool {
        !self.update_rcs().is_empty()
    }

    fn rcfiles(&self) -> Vec<PathBuf> {
        // Bash also may read .profile, however we already includes handling
        // .profile as part of POSIX and always does setup for POSIX shells.
        [".bash_profile", ".bash_login", ".bashrc"]
            .iter()
            .filter_map(|rc| crate::get_home_dir_path().ok().map(|dir| dir.join(rc)))
            .collect()
    }

    fn update_rcs(&self) -> Vec<PathBuf> {
        self.rcfiles()
            .into_iter()
            .filter(|rc| rc.is_file())
            .collect()
    }
}

#[derive(Debug)]
struct Zsh;

impl Zsh {
    fn zdotdir() -> Result<PathBuf, InstallerError> {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        if matches!(env::var("SHELL"), Ok(sh) if sh.contains("zsh")) {
            match env::var("ZDOTDIR") {
                Ok(dir) if !dir.is_empty() => Ok(PathBuf::from(dir)),
                _ => Err(InstallerError::ZshSetup),
            }
        } else {
            match std::process::Command::new("zsh")
                .args(&["-c", "'echo $ZDOTDIR'"])
                .output()
            {
                Ok(io) if !io.stdout.is_empty() => Ok(PathBuf::from(OsStr::from_bytes(&io.stdout))),
                _ => Err(InstallerError::ZshSetup),
            }
        }
    }
}

fn has_cmd(cmd: &str) -> bool {
    let cmd = format!("{}{}", cmd, env::consts::EXE_SUFFIX);
    let path = env::var_os("PATH").unwrap_or_default();
    env::split_paths(&path)
        .map(|p| p.join(&cmd))
        .any(|p| p.exists())
}

impl UnixShell for Zsh {
    fn does_exist(&self) -> bool {
        // zsh has to either be the shell or be callable for zsh setup.
        matches!(env::var("SHELL"), Ok(sh) if sh.contains("zsh")) || matches!(has_cmd("zsh"), true)
    }

    fn rcfiles(&self) -> Vec<PathBuf> {
        [Zsh::zdotdir().ok(), crate::get_home_dir_path().ok()]
            .iter()
            .filter_map(|dir| dir.as_ref().map(|p| p.join(".zshenv")))
            .collect()
    }

    fn update_rcs(&self) -> Vec<PathBuf> {
        // zsh can change $ZDOTDIR both _before_ AND _during_ reading .zshenv,
        // so we: write to $ZDOTDIR/.zshenv if-exists ($ZDOTDIR changes before)
        // OR write to $HOME/.zshenv if it exists (change-during)
        // if neither exist, we create it ourselves, but using the same logic,
        // because we must still respond to whether $ZDOTDIR is set or unset.
        // In any case we only write once.
        self.rcfiles()
            .into_iter()
            .filter(|env| env.is_file())
            .chain(self.rcfiles().into_iter())
            .take(1)
            .collect()
    }
}
