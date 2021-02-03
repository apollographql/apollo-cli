use serde::Serialize;
use structopt::StructOpt;

use crate::stringify::from_display;
use crate::Result;
use crate::{
    client::StudioClientConfig,
    command::{self, RoverStdout},
};
use crate::{
    env::{RoverEnv, RoverEnvKey},
    git::GitContext,
};
use config::Config;
use houston as config;
use timber::{Level, DEFAULT_LEVEL, LEVELS};

use std::path::PathBuf;

#[derive(Debug, Serialize, StructOpt)]
#[structopt(name = "Rover", global_settings = &[structopt::clap::AppSettings::ColoredHelp], about = "
Rover - Your Graph Companion
Read the getting started guide: https://go.apollo.dev/r/start

To begin working with Rover and to authenticate with Apollo Studio, 
run the following command:

    $ rover config auth

This will prompt you for an API Key that can be generated in Apollo Studio.

The most common commands from there are:

    - rover graph fetch: Fetch a graph schema from the Apollo graph registry
    - rover graph check: Check for breaking changes in a local graph schema against a graph schema in the Apollo graph registry
    - rover graph push: Push an updated graph schema to the Apollo graph registry

You can find full documentation for Rover here: https://go.apollo.dev/r/docs
")]
pub struct Rover {
    #[structopt(subcommand)]
    pub command: Command,

    #[structopt(long = "log", short = "l", global = true, default_value = DEFAULT_LEVEL, possible_values = &LEVELS, case_insensitive = true)]
    #[serde(serialize_with = "from_display")]
    pub log_level: Level,

    #[structopt(skip)]
    #[serde(skip_serializing)]
    pub env_store: RoverEnv,
}

impl Rover {
    pub(crate) fn get_rover_config(&self) -> Result<Config> {
        let override_home: Option<PathBuf> = self
            .env_store
            .get(RoverEnvKey::ConfigHome)?
            .map(|p| PathBuf::from(&p));
        let override_api_key = self.env_store.get(RoverEnvKey::Key)?;
        Ok(Config::new(override_home.as_ref(), override_api_key)?)
    }

    pub(crate) fn get_client_config(&self) -> Result<StudioClientConfig> {
        let override_endpoint = self.env_store.get(RoverEnvKey::RegistryUrl)?;
        let config = self.get_rover_config()?;
        Ok(StudioClientConfig::new(override_endpoint, config))
    }

    pub(crate) fn get_install_override_path(&self) -> Result<Option<PathBuf>> {
        Ok(self
            .env_store
            .get(RoverEnvKey::Home)?
            .map(|p| PathBuf::from(&p)))
    }

    pub(crate) fn get_git_context(&self) -> GitContext {
        // constructing GitContext with a set of overrides from env vars
        let git_context = GitContext::with_env(&self.env_store);
        log::debug!("GitContext: {:?}", &git_context);
        git_context
    }
}

#[derive(Debug, Serialize, StructOpt)]
pub enum Command {
    /// Configuration profile commands
    Config(command::Config),

    /// Non-federated schema/graph commands
    Graph(command::Graph),

    /// Federated schema/graph commands
    Subgraph(command::Subgraph),

    #[structopt(setting(structopt::clap::AppSettings::Hidden))]
    Install(command::Install),
}

impl Rover {
    pub fn run(&self) -> Result<RoverStdout> {
        match &self.command {
            Command::Config(command) => command.run(self.get_rover_config()?),
            Command::Graph(command) => {
                command.run(self.get_client_config()?, self.get_git_context())
            }
            Command::Subgraph(command) => {
                command.run(self.get_client_config()?, self.get_git_context())
            }
            Command::Install(command) => command.run(self.get_install_override_path()?),
        }
    }
}
