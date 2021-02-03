use ansi_term::Colour::{Cyan, Yellow};
use serde::Serialize;
use structopt::StructOpt;

use crate::client::StudioClientConfig;
use crate::command::RoverStdout;
use crate::utils::parsers::{parse_graph_ref, GraphRef};
use crate::Result;

use rover_client::query::subgraph::delete::{self, DeleteServiceResponse};

#[derive(Debug, Serialize, StructOpt)]
pub struct Delete {
    /// <NAME>@<VARIANT> of federated graph in Apollo Studio to delete subgraph from.
    /// @<VARIANT> may be left off, defaulting to @current
    #[structopt(name = "GRAPH_REF", parse(try_from_str = parse_graph_ref))]
    #[serde(skip_serializing)]
    graph: GraphRef,

    /// Name of configuration profile to use
    #[structopt(long = "profile", default_value = "default")]
    #[serde(skip_serializing)]
    profile_name: String,

    /// Name of subgraph in federated graph to delete
    #[structopt(long = "name")]
    #[serde(skip_serializing)]
    subgraph: String,

    /// Skips the step where the command asks for user confirmation before
    /// deleting the subgraph. Also skips preview of composition errors that
    /// might occur
    #[structopt(long)]
    confirm: bool,
}

impl Delete {
    pub fn run(&self, client_config: StudioClientConfig) -> Result<RoverStdout> {
        let client = client_config.get_client(&self.profile_name)?;
        let graph_ref = self.graph.to_string();
        log::info!(
            "Checking for composition errors resulting from deleting subgraph {} from {} using credentials from the {} profile.",
            Cyan.normal().paint(&self.subgraph),
            Cyan.normal().paint(&graph_ref),
            Yellow.normal().paint(&self.profile_name)
        );

        // this is probably the normal path -- preview a subgraph delete
        // and make the user confirm it manually.
        if !self.confirm {
            // run delete with dryRun, so we can preview composition errors
            let delete_dry_run_response = delete::run(
                delete::delete_service_mutation::Variables {
                    graph_id: self.graph.name.clone(),
                    graph_variant: self.graph.variant.clone(),
                    name: self.subgraph.clone(),
                    dry_run: true,
                },
                &client,
            )?;

            handle_dry_run_response(delete_dry_run_response, &self.subgraph, &graph_ref);

            // I chose not to error here, since this is a perfectly valid path
            if !confirm_delete()? {
                log::info!("Delete cancelled by user");
                return Ok(RoverStdout::None);
            }
        }

        let delete_response = delete::run(
            delete::delete_service_mutation::Variables {
                graph_id: self.graph.name.clone(),
                graph_variant: self.graph.variant.clone(),
                name: self.subgraph.clone(),
                dry_run: false,
            },
            &client,
        )?;

        handle_response(delete_response, &self.subgraph, &graph_ref);
        Ok(RoverStdout::None)
    }
}

fn handle_dry_run_response(response: DeleteServiceResponse, subgraph: &str, graph_ref: &str) {
    if let Some(errors) = response.composition_errors {
        log::warn!(
                "Deleting the {} subgraph from {} would result in the following composition errors: \n{}",
                subgraph,
                graph_ref,
                errors.join("\n")
            );
        log::warn!("Note: This is only a prediction. If the graph changes before confirming, these errors could change.");
    } else {
        log::info!("At the time of checking, there would be no composition errors resulting from the deletion of this subgraph.");
        log::warn!("Note: This is only a prediction. If the graph changes before confirming, there could be composition errors.")
    }
}

fn confirm_delete() -> Result<bool> {
    log::info!("Would you like to continue [y/n]");
    let term = console::Term::stdout();
    let confirm = term.read_line()?;
    if confirm.to_lowercase() == *"y" {
        Ok(true)
    } else {
        Ok(false)
    }
}

fn handle_response(response: DeleteServiceResponse, subgraph: &str, graph_ref: &str) {
    if response.updated_gateway {
        log::info!(
            "The {} subgraph was removed from {}. Remaining subgraphs were composed.",
            Cyan.normal().paint(subgraph),
            Cyan.normal().paint(graph_ref),
        )
    } else {
        log::error!(
            "The gateway for {} was not updated. Check errors below.",
            Cyan.normal().paint(graph_ref)
        )
    }

    if let Some(errors) = response.composition_errors {
        log::error!(
            "There were composition errors as a result of deleting the subgraph: \n{}",
            errors.join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{handle_response, DeleteServiceResponse};

    #[test]
    fn handle_response_doesnt_error_with_all_successes() {
        let response = DeleteServiceResponse {
            composition_errors: None,
            updated_gateway: true,
        };

        handle_response(response, "accounts", "my-graph@current");
    }

    #[test]
    fn handle_response_doesnt_error_with_all_failures() {
        let response = DeleteServiceResponse {
            composition_errors: Some(vec![
                "a bad thing happened".to_string(),
                "another bad thing".to_string(),
            ]),
            updated_gateway: false,
        };

        handle_response(response, "accounts", "my-graph@prod");
    }

    // TODO: test the actual output of the logs whenever we do design work
    // for the commands :)
}
