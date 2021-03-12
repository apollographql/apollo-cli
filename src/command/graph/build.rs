use crate::{anyhow, command::RoverStdout, Result};

use ansi_term::Colour::Red;
use camino::Utf8PathBuf;
use harmonizer::{self, ServiceDefinition};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, StructOpt)]
pub struct Build {
    /// The path to the service list definition
    #[structopt(long = "services")]
    #[serde(skip_serializing)]
    service_list_file: Utf8PathBuf,
}

#[derive(Deserialize)]
struct Services {
    subgraphs: HashMap<String, Subgraph>,
}

#[derive(Deserialize)]
struct Subgraph {
    url: String,
    schema: Utf8PathBuf,
}

impl Build {
    pub fn run(&self) -> Result<RoverStdout> {
        let service_list_contents = fs::read_to_string(&self.service_list_file)?;
        let parsed_service_list: Services = toml::from_str(&service_list_contents)?;
        let mut service_list = Vec::new();
        for (subgraph_name, subgraph_data) in parsed_service_list.subgraphs {
            let relative_schema_path = if let Some(parent) = self.service_list_file.parent() {
                let mut schema_path = parent.to_path_buf();
                schema_path.push(subgraph_data.schema);
                schema_path
            } else {
                subgraph_data.schema
            };
            let schema = fs::read_to_string(relative_schema_path)?;
            let service_definition =
                ServiceDefinition::new(subgraph_name, subgraph_data.url, &schema);
            service_list.push(service_definition);
        }

        match harmonizer::harmonize(service_list) {
            Ok(csdl) => Ok(RoverStdout::CSDL(csdl)),
            Err(composition_errors) => {
                let num_failures = composition_errors.len();
                for composition_error in composition_errors {
                    eprintln!("{} {}", Red.bold().paint("error:"), &composition_error)
                }
                match num_failures {
                    0 => Ok(RoverStdout::None),
                    1 => Err(
                        anyhow!("Encountered 1 composition error while composing the graph.")
                            .into(),
                    ),
                    _ => Err(anyhow!(
                        "Encountered {} composition errors while composing the graph.",
                        num_failures
                    )
                    .into()),
                }
            }
        }
    }
}
