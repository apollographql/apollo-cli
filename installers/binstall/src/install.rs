use crate::InstallerError;

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use atty::{self, Stream};

pub struct Installer {
    pub binary_name: String,
    pub force_install: bool,
    pub executable_location: PathBuf,
    pub override_install_path: Option<PathBuf>,
}

impl Installer {
    pub fn install(&self) -> Result<Option<PathBuf>, InstallerError> {
        let install_path = self.do_install()?;

        Ok(install_path)
    }

    fn do_install(&self) -> Result<Option<PathBuf>, InstallerError> {
        let bin_destination = self.get_bin_path()?;

        if !self.force_install
            && bin_destination.exists()
            && !self.should_overwrite(&bin_destination)?
        {
            return Ok(None);
        }

        log::info!("creating directory for binary");
        self.create_bin_dir()?;

        log::info!("writing binary to {}", &bin_destination.display());
        self.write_bin_to_fs()?;

        log::info!("adding binary to PATH");
        self.add_binary_to_path()?;

        Ok(Some(bin_destination))
    }

    pub(crate) fn get_base_dir_path(&self) -> Result<PathBuf, InstallerError> {
        let base_dir = if let Some(base_dir) = &self.override_install_path {
            Ok(base_dir.to_owned())
        } else {
            crate::get_home_dir_path()
        }?;
        Ok(base_dir.join(&format!(".{}", &self.binary_name)))
    }

    pub(crate) fn get_bin_dir_path(&self) -> Result<PathBuf, InstallerError> {
        let bin_dir = self.get_base_dir_path()?.join("bin");
        Ok(bin_dir)
    }

    fn create_bin_dir(&self) -> Result<(), InstallerError> {
        fs::create_dir_all(self.get_bin_dir_path()?)?;
        Ok(())
    }

    fn get_bin_path(&self) -> Result<PathBuf, InstallerError> {
        Ok(self
            .get_bin_dir_path()?
            .join(&self.binary_name)
            .with_extension(env::consts::EXE_EXTENSION))
    }

    fn write_bin_to_fs(&self) -> Result<(), InstallerError> {
        let bin_path = self.get_bin_path()?;
        log::debug!("copying from: {}", &self.executable_location.display());
        log::debug!("copying to: {}", &bin_path.display());
        fs::copy(&self.executable_location, &bin_path)?;
        Ok(())
    }

    fn should_overwrite(&self, destination: &PathBuf) -> Result<bool, InstallerError> {
        // If we're not attached to a TTY then we can't get user input, so there's
        // nothing to do except inform the user about the `-f` flag.
        if !atty::is(Stream::Stdin) {
            return Err(io::Error::from(io::ErrorKind::AlreadyExists).into());
        }

        // It looks like we're at an interactive prompt, so ask the user if they'd
        // like to overwrite the previous installation.
        log::info!(
            "existing {} installation found at `{}`",
            &self.binary_name,
            destination.display()
        );
        log::info!("Would you like to overwrite this file? [y/N]: ");
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        if line.to_lowercase().starts_with('y') {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[cfg(windows)]
    fn add_binary_to_path(&self) -> Result<(), InstallerError> {
        crate::windows::add_binary_to_path(self)
    }

    #[cfg(not(windows))]
    fn add_binary_to_path(&self) -> Result<(), InstallerError> {
        crate::unix::add_binary_to_path(self)
    }
}
