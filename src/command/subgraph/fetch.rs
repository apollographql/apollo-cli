use ansi_term::Colour::{Cyan, Yellow};
use serde::Serialize;
use structopt::StructOpt;

use rover_client::query::subgraph::fetch;

use crate::client::StudioClientConfig;
use crate::command::RoverStdout;
use crate::utils::parsers::{parse_graph_ref, GraphRef};
use crate::Result;

#[derive(Debug, Serialize, StructOpt)]
pub struct Fetch {
    /// <NAME>@<VARIANT> of graph in Apollo Studio to fetch from.
    /// @<VARIANT> may be left off, defaulting to @current
    #[structopt(name = "GRAPH_REF", parse(try_from_str = parse_graph_ref))]
    #[serde(skip_serializing)]
    graph: GraphRef,

    /// Name of configuration profile to use
    #[structopt(long = "profile", default_value = "default")]
    #[serde(skip_serializing)]
    profile_name: String,

    /// Name of subgraph in federated graph to update
    #[structopt(long = "name")]
    #[serde(skip_serializing)]
    subgraph: String,
}

impl Fetch {
    pub fn run(&self, client_config: StudioClientConfig) -> Result<RoverStdout> {
        let client = client_config.get_client(&self.profile_name)?;
        let graph_ref = self.graph.to_string();
        log::info!(
            "Fetching SDL from {} (subgraph: {}) using credentials from the {} profile.",
            Cyan.normal().paint(&graph_ref),
            Cyan.normal().paint(&self.subgraph),
            Yellow.normal().paint(&self.profile_name)
        );

        let sdl = fetch::run(
            fetch::fetch_subgraph_query::Variables {
                graph_id: self.graph.name.clone(),
                variant: self.graph.variant.clone(),
            },
            &client,
            &self.subgraph,
        )?;

        Ok(RoverStdout::SDL(sdl))
    }
}
