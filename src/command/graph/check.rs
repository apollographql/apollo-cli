use prettytable::{cell, row, Table};
use serde::Serialize;
use structopt::StructOpt;

use rover_client::query::graph::check;

use crate::client::StudioClientConfig;
use crate::command::RoverStdout;
use crate::git::GitContext;
use crate::utils::loaders::load_schema_from_flag;
use crate::utils::parsers::{parse_graph_ref, parse_schema_source, GraphRef, SchemaSource};
use crate::Result;

#[derive(Debug, Serialize, StructOpt)]
pub struct Check {
    /// <NAME>@<VARIANT> of graph in Apollo Studio to validate.
    /// @<VARIANT> may be left off, defaulting to @current
    #[structopt(name = "GRAPH_REF", parse(try_from_str = parse_graph_ref))]
    #[serde(skip_serializing)]
    graph: GraphRef,

    /// Name of configuration profile to use
    #[structopt(long = "profile", default_value = "default")]
    #[serde(skip_serializing)]
    profile_name: String,

    /// The schema file to push
    /// Can pass `-` to use stdin instead of a file
    #[structopt(long, short = "s", parse(try_from_str = parse_schema_source))]
    #[serde(skip_serializing)]
    schema: SchemaSource,
}

impl Check {
    pub fn run(
        &self,
        client_config: StudioClientConfig,
        git_context: GitContext,
    ) -> Result<RoverStdout> {
        let client = client_config.get_client(&self.profile_name)?;
        let sdl = load_schema_from_flag(&self.schema, std::io::stdin())?;
        let res = check::run(
            check::check_schema_query::Variables {
                graph_id: self.graph.name.clone(),
                variant: Some(self.graph.variant.clone()),
                schema: Some(sdl),
                git_context: git_context.into(),
            },
            &client,
        )?;

        log::info!(
            "Validated the proposed subgraph against metrics from {}",
            &self.graph
        );

        let num_changes = res.changes.len();

        let msg = match num_changes {
            0 => "There is no difference between the proposed graph and the graph that already exists in the graph registry. Try making a change to your proposed graph before running this command.".to_string(),
            _ => format!("Compared {} schema changes against {} operations", res.changes.len(), res.number_of_checked_operations),
        };

        log::info!("{}", &msg);

        let num_failures = print_changes(&res.changes);

        if let Some(url) = res.target_url {
            log::info!("View full details here");
            log::info!("{}", url.to_string());
        }

        match num_failures {
            0 => Ok(RoverStdout::None),
            1 => Err(anyhow::anyhow!("Encountered 1 failure.").into()),
            _ => Err(anyhow::anyhow!("Encountered {} failures.", num_failures).into()),
        }
    }
}

fn print_changes(
    checks: &[check::check_schema_query::CheckSchemaQueryServiceCheckSchemaDiffToPreviousChanges],
) -> u64 {
    let mut num_failures = 0;

    if !checks.is_empty() {
        let mut table = Table::new();
        table.add_row(row!["Change", "Code", "Description"]);
        for check in checks {
            let change = match check.severity {
                check::check_schema_query::ChangeSeverity::NOTICE => "PASS",
                check::check_schema_query::ChangeSeverity::FAILURE => {
                    num_failures += 1;
                    "FAIL"
                }
                _ => unreachable!("Unknown change severity"),
            };
            table.add_row(row![change, check.code, check.description]);
        }

        eprintln!("{}", table);
    }

    num_failures
}
