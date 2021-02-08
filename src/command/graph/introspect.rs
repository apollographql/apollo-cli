use crate::Result;
use serde::Serialize;
use structopt::StructOpt;
use url::Url;

use rover_client::{blocking::Client, query::graph::introspect};

use crate::command::RoverStdout;
use crate::utils::parsers::parse_url;

#[derive(Debug, Serialize, StructOpt)]
pub struct Introspect {
    /// The endpoint of the graph to introspect
    #[structopt(parse(try_from_str = parse_url))]
    #[serde(skip_serializing)]
    pub endpoint: Url,
}

impl Introspect {
    pub fn run(&self) -> Result<RoverStdout> {
        let client = Client::new(&self.endpoint.to_string());

        let introspection_response = introspect::run(&client)?;

        Ok(RoverStdout::Introspection(introspection_response.result))
    }
}
