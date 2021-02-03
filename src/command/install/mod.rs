use serde::Serialize;
use structopt::StructOpt;

use binstall::Installer;

use crate::command::RoverStdout;
use crate::{anyhow, Context, Result};

use std::env;
use std::path::PathBuf;

#[derive(Debug, Serialize, StructOpt)]
pub struct Install {
    #[structopt(long = "force", short = "f")]
    force: bool,
}

impl Install {
    pub fn run(&self, override_install_path: Option<PathBuf>) -> Result<RoverStdout> {
        let binary_name = env!("CARGO_PKG_NAME").to_string();

        if let Ok(executable_location) = env::current_exe() {
            let install_location = Installer {
                binary_name: binary_name.clone(),
                force_install: self.force,
                override_install_path,
                executable_location,
            }
            .install()
            .with_context(|| format!("could not install {}", &binary_name))?;

            if let Some(install_location) = install_location {
                log::info!("{} was successfully installed to `{}`. You may need to reload your terminal for the binary to be loaded into your PATH.", &binary_name, install_location.display())
            } else {
                log::info!("{} was not installed. To override the existing installation, you can pass the `--force` flag to the installer.", &binary_name);
            }
            Ok(RoverStdout::None)
        } else {
            Err(anyhow!("Failed to get the current executable's path.").into())
        }
    }
}
