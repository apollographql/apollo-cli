use serde::Serialize;
use structopt::StructOpt;

use houston as config;

use crate::command::RoverStdout;
use crate::Result;

#[derive(Debug, Serialize, StructOpt)]
/// Delete a configuration profile
///
/// Pass a name argument to choose which profile to delete.
/// Not passing a profile name will delete the `default` profile
///
/// WARNING: Deleting the `default` profile will result in
/// errors when running commands without specifying a `--profile`.
pub struct Delete {
    #[serde(skip_serializing)]
    name: String,
}

impl Delete {
    pub fn run(&self, config: config::Config) -> Result<RoverStdout> {
        config::Profile::delete(&self.name, &config)?;
        log::info!("Successfully deleted profile \"{}\"", &self.name);
        Ok(RoverStdout::None)
    }
}
