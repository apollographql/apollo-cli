use ansi_term::Colour::{Cyan, Yellow};
use serde::Serialize;
use structopt::StructOpt;

use rover_client::query::graph::push;

use crate::client::StudioClientConfig;
use crate::command::RoverStdout;
use crate::git::GitContext;
use crate::utils::loaders::load_schema_from_flag;
use crate::utils::parsers::{parse_graph_ref, parse_schema_source, GraphRef, SchemaSource};
use crate::Result;

#[derive(Debug, Serialize, StructOpt)]
pub struct Push {
    /// <NAME>@<VARIANT> of graph in Apollo Studio to push to.
    /// @<VARIANT> may be left off, defaulting to @current
    #[structopt(name = "GRAPH_REF", parse(try_from_str = parse_graph_ref))]
    #[serde(skip_serializing)]
    graph: GraphRef,

    /// The schema file to push
    /// Can pass `-` to use stdin instead of a file
    #[structopt(long, short = "s", parse(try_from_str = parse_schema_source))]
    #[serde(skip_serializing)]
    schema: SchemaSource,

    /// Name of configuration profile to use
    #[structopt(long = "profile", default_value = "default")]
    #[serde(skip_serializing)]
    profile_name: String,
}

impl Push {
    pub fn run(
        &self,
        client_config: StudioClientConfig,
        git_context: GitContext,
    ) -> Result<RoverStdout> {
        let client = client_config.get_client(&self.profile_name)?;
        let graph_ref = self.graph.to_string();
        log::info!(
            "Pushing SDL to {} using credentials from the {} profile.",
            Cyan.normal().paint(&graph_ref),
            Yellow.normal().paint(&self.profile_name)
        );

        let schema_document = load_schema_from_flag(&self.schema, std::io::stdin())?;

        log::debug!("Schema Document to push:\n{}", &schema_document);

        let push_response = push::run(
            push::push_schema_mutation::Variables {
                graph_id: self.graph.name.clone(),
                variant: self.graph.variant.clone(),
                schema_document: Some(schema_document),
                git_context: git_context.into(),
            },
            &client,
        )?;

        let hash = handle_response(&self.graph, push_response);
        Ok(RoverStdout::SchemaHash(hash))
    }
}

/// handle all output logging from operation
fn handle_response(graph: &GraphRef, response: push::PushResponse) -> String {
    log::info!(
        "{}#{} Pushed successfully {}",
        graph,
        response.schema_hash,
        response.change_summary
    );

    response.schema_hash
}

#[cfg(test)]
mod tests {
    use super::{handle_response, push, GraphRef};

    #[test]
    fn handle_response_doesnt_err() {
        let expected_hash = "123456".to_string();
        let graph = GraphRef {
            name: "harambe".to_string(),
            variant: "inside-job".to_string(),
        };
        let actual_hash = handle_response(
            &graph,
            push::PushResponse {
                schema_hash: expected_hash.clone(),
                change_summary: "".to_string(),
            },
        );
        assert_eq!(actual_hash, expected_hash);
    }
}
