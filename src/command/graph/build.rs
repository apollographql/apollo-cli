use crate::{command::RoverStdout, Result};

use serde::Serialize;
use structopt::StructOpt;

#[derive(Debug, Serialize, StructOpt)]
pub struct Build {}

impl Build {
    pub fn run(&self) -> Result<RoverStdout> {
        Ok(RoverStdout::None)
    }
}
