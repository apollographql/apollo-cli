use url::Url;

use std::path::PathBuf;

use crate::cli::Rover;
use crate::env::RoverEnvKey;
use sputnik::{Command, Report, SputnikError};

use std::collections::HashMap;

const DEV_TELEMETRY_URL: &str = "http://localhost:8787/telemetry";
const PROD_TELEMETRY_URL: &str = "https://install.apollographql.com/telemetry";

fn get_command_from_args(mut raw_arguments: &mut serde_json::Value) -> Command {
    let mut commands = Vec::new();
    let mut arguments = HashMap::new();
    let mut should_break = true;
    loop {
        let (command_name, leftover_arguments) = get_next_command(&mut raw_arguments);
        if let Some(command_name) = command_name {
            commands.push(command_name);
            should_break = false;
        }

        if let Some(serde_json::Value::Object(object)) = leftover_arguments {
            for argument in object.iter() {
                let (key, value) = argument;
                arguments.insert(key.to_lowercase(), value.to_owned());
            }
        }

        if should_break {
            break;
        } else {
            should_break = true;
        }
    }

    let name = commands.join(" ");
    Command { name, arguments }
}

fn get_next_command(
    raw_arguments: &mut serde_json::Value,
) -> (Option<String>, Option<serde_json::Value>) {
    let mut command_name = None;
    let mut leftover_arguments = None;

    if let Some(command_info) = raw_arguments.get("command") {
        match command_info {
            serde_json::Value::Object(object) => {
                if let Some(item) = object.clone().iter_mut().next() {
                    let (name, next) = item;
                    command_name = Some(name.to_lowercase());
                    *raw_arguments = next.to_owned();
                }
            }
            serde_json::Value::String(string) => {
                command_name = Some(string.to_lowercase());
                *raw_arguments = serde_json::Value::Null;
            }
            serde_json::Value::Null => command_name = None,
            _ => {
                command_name = Some(format!("{:?}", command_info).to_lowercase());
                *raw_arguments = serde_json::Value::Null;
            }
        }
    } else {
        leftover_arguments = Some(raw_arguments.clone());
    }

    (command_name, leftover_arguments)
}

impl Report for Rover {
    fn serialize_command(&self) -> Result<Command, SputnikError> {
        let json_args = serde_json::to_string(&self)?;
        let mut value_args = serde_json::from_str(&json_args)?;
        let serialized_command = get_command_from_args(&mut value_args);
        log::debug!("serialized command: {:?}", &serialized_command);
        Ok(serialized_command)
    }

    fn is_telemetry_enabled(&self) -> Result<bool, SputnikError> {
        let value = self.env_store.get(RoverEnvKey::TelemetryDisabled)?;
        let is_telemetry_disabled = value.is_some();
        if is_telemetry_disabled {
            log::info!("Anonymous usage reporting has been disabled.");
        }
        Ok(!is_telemetry_disabled)
    }

    fn endpoint(&self) -> Result<Url, SputnikError> {
        if let Some(url) = self.env_store.get(RoverEnvKey::TelemetryUrl)? {
            Ok(Url::parse(&url)?)
        } else if cfg!(debug_assertions) {
            Ok(DEV_TELEMETRY_URL.parse()?)
        } else {
            Ok(PROD_TELEMETRY_URL.parse()?)
        }
    }

    fn tool_name(&self) -> String {
        option_env!("CARGO_PKG_NAME")
            .unwrap_or("unknown")
            .to_string()
    }

    fn version(&self) -> String {
        option_env!("CARGO_PKG_VERSION")
            .unwrap_or("unknown")
            .to_string()
    }

    fn machine_id_config(&self) -> Result<PathBuf, SputnikError> {
        let config = self
            .get_rover_config()
            .map_err(|_| SputnikError::ConfigError)?;
        Ok(config.home.join("machine.txt"))
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::Rover;
    use crate::env::RoverEnvKey;
    use crate::telemetry::Report;

    use sputnik::Command;

    use serde_json::json;
    use structopt::StructOpt;

    use std::collections::HashMap;

    #[test]
    fn it_can_serialize_commands() {
        let cli_name = env!("CARGO_PKG_NAME");
        let args = vec![cli_name, "config", "list"];
        let rover = Rover::from_iter(args);
        let actual_serialized_command = rover
            .serialize_command()
            .expect("could not serialize command");
        let expected_serialized_command = Command {
            name: "config list".to_string(),
            arguments: HashMap::new(),
        };
        assert_eq!(actual_serialized_command, expected_serialized_command);
    }

    #[test]
    fn it_can_serialize_commands_with_arguments() {
        let cli_name = env!("CARGO_PKG_NAME");
        let args = vec![cli_name, "config", "show", "default", "--sensitive"];
        let rover = Rover::from_iter(args);
        let actual_serialized_command = rover
            .serialize_command()
            .expect("could not serialize command");
        let mut expected_arguments = HashMap::new();
        expected_arguments.insert("sensitive".to_string(), json!(true));
        let expected_serialized_command = Command {
            name: "config show".to_string(),
            arguments: expected_arguments,
        };
        assert_eq!(actual_serialized_command, expected_serialized_command);
    }

    #[test]
    fn it_respects_apollo_telemetry_url() {
        let apollo_telemetry_url = "https://example.com/telemetry";
        let cli_name = env!("CARGO_PKG_NAME");
        let args = vec![cli_name, "config", "list"];
        let mut rover = Rover::from_iter(args);
        rover
            .env_store
            .insert(RoverEnvKey::TelemetryUrl, apollo_telemetry_url);
        let actual_endpoint = rover
            .endpoint()
            .expect("could not parse telemetry URL")
            .to_string();
        let expected_endpoint = apollo_telemetry_url.to_string();

        assert_eq!(actual_endpoint, expected_endpoint);
    }

    #[test]
    fn it_can_be_disabled() {
        let cli_name = env!("CARGO_PKG_NAME");
        let args = vec![cli_name, "config", "list"];
        let mut rover = Rover::from_iter(args);
        rover.env_store.insert(RoverEnvKey::TelemetryDisabled, "1");
        let expect_enabled = false;
        let is_telemetry_enabled = rover.is_telemetry_enabled().unwrap();

        assert_eq!(is_telemetry_enabled, expect_enabled);
    }

    #[test]
    fn it_is_enabled_by_default() {
        let cli_name = env!("CARGO_PKG_NAME");
        let args = vec![cli_name, "config", "list"];
        let rover = Rover::from_iter(args);
        let expect_enabled = true;
        let is_telemetry_enabled = rover.is_telemetry_enabled().unwrap();
        assert_eq!(is_telemetry_enabled, expect_enabled);
    }
}
