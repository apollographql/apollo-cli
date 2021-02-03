use std::fmt::Debug;

use atty::{self, Stream};
use prettytable::{cell, row, Table};
use rover_client::query::subgraph::list::ListDetails;

/// RoverStdout defines all of the different types of data that are printed
/// to `stdout`. Every one of Rover's commands should return `anyhow::Result<RoverStdout>`
/// If the command needs to output some type of data, it should be structured
/// in this enum, and its print logic should be handled in `RoverStdout::print`
///
/// Not all commands will output machine readable information, and those should
/// return `Ok(RoverStdout::None)`. If a new command is added and it needs to
/// return something that is not described well in this enum, it should be added.
#[derive(Clone, PartialEq, Debug)]
pub enum RoverStdout {
    SDL(String),
    SchemaHash(String),
    SubgraphList(ListDetails),
    None,
}

impl RoverStdout {
    pub fn print(&self) {
        match self {
            RoverStdout::SDL(sdl) => {
                // we check to see if stdout is redirected
                // if it is, we don't print the content descriptor
                // this is because it would look strange to see
                // SDL:
                // and nothing after the colon if you piped the output
                // to another process or a file.
                if atty::is(Stream::Stdout) {
                    log::info!("SDL:");
                }
                println!("{}", &sdl);
            }
            RoverStdout::SchemaHash(hash) => {
                if atty::is(Stream::Stdout) {
                    log::info!("Schema Hash:");
                }
                println!("{}", &hash);
            }
            RoverStdout::SubgraphList(details) => {
                println!("Subgraphs:\n");

                let mut table = Table::new();
                table.add_row(row!["Name", "Routing Url", "Last Updated"]);

                for subgraph in &details.subgraphs {
                    // if the url is None or empty (""), then set it to "N/A"
                    let url = subgraph.url.clone().unwrap_or_else(|| "N/A".to_string());
                    let url = if url.is_empty() {
                        "N/A".to_string()
                    } else {
                        url
                    };
                    let formatted_updated_at: String = if let Some(dt) = subgraph.updated_at {
                        dt.format("%Y-%m-%d %H:%M:%S %Z").to_string()
                    } else {
                        "N/A".to_string()
                    };

                    table.add_row(row![subgraph.name, url, formatted_updated_at]);
                }

                println!("{}", table);
                println!(
                    "View full details at {}/graph/{}/service-list",
                    details.root_url, details.graph_name
                );
            }
            RoverStdout::None => (),
        }
    }
}
