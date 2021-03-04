mod check;
mod introspect;
mod fetch;
mod push;

use serde::Serialize;
use structopt::StructOpt;

use crate::command::RoverStdout;
use crate::utils::{client::StudioClientConfig, git::GitContext};
use crate::Result;

#[derive(Debug, Serialize, StructOpt)]
pub struct Graph {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, Serialize, StructOpt)]
pub enum Command {
    /// Check for breaking changes in a local graph schema
    /// against a graph schema in the Apollo graph registry
    Check(check::Check),

    /// Fetch a graph schema from the Apollo graph registry
    Fetch(fetch::Fetch),

    /// Push an updated graph schema to the Apollo graph registry
    Push(push::Push),

    /// Introspect current graph schema.
    Introspect(introspect::Introspect),
}

impl Graph {
    pub fn run(
        &self,
        client_config: StudioClientConfig,
        git_context: GitContext,
    ) -> Result<RoverStdout> {
        match &self.command {
            Command::Fetch(command) => command.run(client_config),
            Command::Push(command) => command.run(client_config, git_context),
            Command::Check(command) => command.run(client_config, git_context),
            Command::Introspect(command) => command.run(),
        }
    }
}
