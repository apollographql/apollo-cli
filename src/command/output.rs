use std::collections::HashMap;
use std::fmt::Debug;

use ansi_term::Colour::Yellow;
use rover_client::query::subgraph::list::ListDetails;

use crate::utils::table::{self, cell, row};

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
    DocsList(HashMap<&'static str, &'static str>),
    SDL(String),
    CSDL(String),
    SchemaHash(String),
    SubgraphList(ListDetails),
    VariantList(Vec<String>),
    Profiles(Vec<String>),
    None,
}

impl RoverStdout {
    pub fn print(&self) {
        match self {
            RoverStdout::DocsList(shortlinks) => {
                eprintln!(
                    "You can open any of these documentation pages by running {}.\n",
                    Yellow.normal().paint("`rover docs open <slug>`")
                );
                let mut table = table::get_table();

                // bc => sets top row to be bold and center
                table.add_row(row![bc => "Slug", "Description"]);
                for (shortlink_slug, shortlink_description) in shortlinks {
                    table.add_row(row![shortlink_slug, shortlink_description]);
                }
                println!("{}", table);
            }
            RoverStdout::SDL(sdl) => {
                eprintln!("SDL:");
                println!("{}", &sdl);
            }
            RoverStdout::CSDL(csdl) => {
                eprintln!("CSDL:");
                println!("{}", &csdl);
            }
            RoverStdout::SchemaHash(hash) => {
                eprint!("Schema Hash: ");
                println!("{}", &hash);
            }
            RoverStdout::SubgraphList(details) => {
                let mut table = table::get_table();

                // bc => sets top row to be bold and center
                table.add_row(row![bc => "Name", "Routing Url", "Last Updated"]);

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
            RoverStdout::VariantList(variants) => {
                eprintln!("Variants:");
                for variant in variants {
                    println!("{}", variant);
                }
            }
            RoverStdout::Profiles(profiles) => {
                if profiles.is_empty() {
                    eprintln!("No profiles found.");
                } else {
                    eprintln!("Profiles:")
                }

                for profile in profiles {
                    println!("{}", profile);
                }
            }
            RoverStdout::None => (),
        }
    }
}
