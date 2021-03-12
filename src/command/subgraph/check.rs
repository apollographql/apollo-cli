use ansi_term::Colour::Red;
use serde::Serialize;
use structopt::StructOpt;

use crate::Result;
use rover_client::query::subgraph::check;

use crate::command::RoverStdout;
use crate::utils::client::StudioClientConfig;
use crate::utils::git::GitContext;
use crate::utils::loaders::load_schema_from_flag;
use crate::utils::parsers::{
    parse_graph_ref, parse_query_count_threshold, parse_query_percentage_threshold,
    parse_schema_source, parse_validation_period, GraphRef, SchemaSource, ValidationPeriod,
};
use crate::utils::table::{self, cell, row};

#[derive(Debug, Serialize, StructOpt)]
pub struct Check {
    /// <NAME>@<VARIANT> of graph in Apollo Studio to validate.
    /// @<VARIANT> may be left off, defaulting to @current
    #[structopt(name = "GRAPH_REF", parse(try_from_str = parse_graph_ref))]
    #[serde(skip_serializing)]
    graph: GraphRef,

    /// Name of the subgraph to validate
    #[structopt(long = "name")]
    #[serde(skip_serializing)]
    subgraph: String,

    /// Name of configuration profile to use
    #[structopt(long = "profile", default_value = "default")]
    #[serde(skip_serializing)]
    profile_name: String,

    /// The schema file to push
    /// Can pass `-` to use stdin instead of a file
    #[structopt(long, short = "s", parse(try_from_str = parse_schema_source))]
    #[serde(skip_serializing)]
    schema: SchemaSource,

    /// The minimum number of times a query or mutation must have been executed
    /// in order to be considered in the check operation
    #[structopt(long, parse(try_from_str = parse_query_count_threshold))]
    query_count_threshold: Option<i64>,

    /// Minimum percentage of times a query or mutation must have been executed
    /// in the time window, relative to total request count, for it to be
    /// considered in the check. Valid numbers are in the range 0 <= x <= 100
    #[structopt(long, parse(try_from_str = parse_query_percentage_threshold))]
    query_percentage_threshold: Option<f64>,

    /// Size of the time window with which to validate schema against (i.e "24h" or "1w 2d 5h")
    #[structopt(long, parse(try_from_str = parse_validation_period))]
    validation_period: Option<ValidationPeriod>,
}

impl Check {
    pub fn run(
        &self,
        client_config: StudioClientConfig,
        git_context: GitContext,
    ) -> Result<RoverStdout> {
        let client = client_config.get_client(&self.profile_name)?;

        let sdl = load_schema_from_flag(&self.schema, std::io::stdin())?;

        let partial_schema = check::check_partial_schema_query::PartialSchemaInput {
            sdl: Some(sdl),
            // we never need to send the hash since the back end computes it from SDL
            hash: None,
        };

        let res = check::run(
            check::check_partial_schema_query::Variables {
                graph_id: self.graph.name.clone(),
                variant: self.graph.variant.clone(),
                partial_schema,
                implementing_service_name: self.subgraph.clone(),
                git_context: git_context.into(),
                config: check::check_partial_schema_query::HistoricQueryParameters {
                    query_count_threshold: self.query_count_threshold,
                    query_count_threshold_percentage: self.query_percentage_threshold,
                    from: self.validation_period.clone().unwrap_or_default().from,
                    to: self.validation_period.clone().unwrap_or_default().to,
                    // we don't support configuring these, but we can't leave them out
                    excluded_clients: None,
                    ignored_operations: None,
                    included_variants: None,
                },
            },
            &client,
        )?;

        eprintln!("Checked the proposed subgraph against {}", &self.graph);

        match res {
            check::CheckResponse::CompositionErrors(composition_errors) => {
                handle_composition_errors(&composition_errors)
            }
            check::CheckResponse::CheckResult(check_result) => handle_checks(check_result),
        }
    }
}

fn handle_checks(check_result: check::CheckResult) -> Result<RoverStdout> {
    let num_changes = check_result.changes.len();

    let msg = match num_changes {
        0 => "There were no changes detected in the composed schema.".to_string(),
        _ => format!(
            "Compared {} schema changes against {} operations",
            check_result.changes.len(),
            check_result.number_of_checked_operations
        ),
    };

    eprintln!("{}", &msg);

    let mut num_failures = 0;

    if !check_result.changes.is_empty() {
        let mut table = table::get_table();

        // bc => sets top row to be bold and center
        table.add_row(row![bc => "Change", "Code", "Description"]);
        for check in check_result.changes {
            let change = match check.severity {
                check::check_partial_schema_query::ChangeSeverity::NOTICE => "PASS",
                check::check_partial_schema_query::ChangeSeverity::FAILURE => {
                    num_failures += 1;
                    "FAIL"
                }
                _ => unreachable!("Unknown change severity"),
            };
            table.add_row(row![change, check.code, check.description]);
        }

        eprintln!("{}", table);
    }

    if let Some(url) = check_result.target_url {
        eprintln!("View full details at {}", &url);
    }

    match num_failures {
        0 => Ok(RoverStdout::None),
        1 => Err(anyhow::anyhow!("Encountered 1 failure while checking your subgraph.").into()),
        _ => Err(anyhow::anyhow!(
            "Encountered {} failures while checking your subgraph.",
            num_failures
        )
        .into()),
    }
}

fn handle_composition_errors(
    composition_errors: &[check::check_partial_schema_query::CheckPartialSchemaQueryServiceCheckPartialSchemaCompositionValidationResultErrors],
) -> Result<RoverStdout> {
    let num_failures = composition_errors.len();
    for error in composition_errors {
        eprintln!("{} {}", Red.bold().paint("error:"), &error.message);
    }
    match num_failures {
        0 => Ok(RoverStdout::None),
        1 => Err(
            anyhow::anyhow!("Encountered 1 composition error while composing the subgraph.").into(),
        ),
        _ => Err(anyhow::anyhow!(
            "Encountered {} composition errors while composing the subgraph.",
            num_failures
        )
        .into()),
    }
}
