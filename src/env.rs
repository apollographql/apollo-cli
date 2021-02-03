use std::collections::HashMap;
use std::env;
use std::fmt;

use heck::ShoutySnekCase;

/// RoverEnv allows us to mock environment variables while
/// running tests. That way we can run our tests in parallel,
/// and our local development environment will not have unintended
/// side effects on our tests.
#[derive(Debug, Clone)]
pub struct RoverEnv {
    mock_store: Option<HashMap<String, String>>,
}

impl Default for RoverEnv {
    fn default() -> RoverEnv {
        RoverEnv::new()
    }
}

impl RoverEnv {
    /// creates a new environment variable store
    pub fn new() -> RoverEnv {
        let mock_store = if cfg!(test) {
            Some(HashMap::new())
        } else {
            None
        };

        RoverEnv { mock_store }
    }

    /// returns the value of the environment variable if it exists
    pub fn get(&self, key: RoverEnvKey) -> std::io::Result<Option<String>> {
        let key = key.to_string();
        log::debug!("reading {}", &key);
        match &self.mock_store {
            Some(mock_store) => Ok(mock_store.get(&key).map(|v| v.to_owned())),
            None => match env::var(&key) {
                Ok(data) => Ok(Some(data)),
                Err(e) => match e {
                    env::VarError::NotPresent => Ok(None),
                    env::VarError::NotUnicode(_) => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "The value of the environment variable \"{}\" is not valid Unicode.",
                            &key
                        ),
                    )),
                },
            },
        }
    }

    /// sets an environment variable to a value
    pub fn insert(&mut self, key: RoverEnvKey, value: &str) {
        let key = key.to_string();
        log::debug!("writing {}:{}", &key, value);
        match &mut self.mock_store {
            Some(mock_store) => {
                mock_store.insert(key, value.into());
            }
            None => {
                env::set_var(&key, value);
            }
        }
    }

    /// unsets an environment variable
    pub fn remove(&mut self, key: RoverEnvKey) {
        let key = key.to_string();
        log::debug!("removing {}", &key);
        match &mut self.mock_store {
            Some(mock_store) => {
                mock_store.remove(&key);
            }
            None => {
                env::remove_var(&key);
            }
        }
    }
}

/// RoverEnvKey defines all of the environment variables
/// that are respected by Rover. Any time a new environment variable
/// is added to the public contract, it should be defined here.
/// Each environment variable is prefixed with `APOLLO_` and
/// the suffix is the name of the key defined here. It will automatically
/// be converted from CamelCase to SHOUTY_SNAKE_CASE.
/// For example, `RoverEnvKey::ConfigHome.to_string()` becomes `APOLLO_CONFIG_HOME`
#[derive(Debug, Copy, Clone)]
pub enum RoverEnvKey {
    ConfigHome,
    Home,
    Key,
    RegistryUrl,
    TelemetryUrl,
    TelemetryDisabled,
    VcsRemoteUrl,
    VcsBranch,
    VcsCommit,
    VcsCommitter,
}

impl fmt::Display for RoverEnvKey {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let dbg = format!("{:?}", self).TO_SHOUTY_SNEK_CASE();
        fmt.write_str(&format!("APOLLO_{}", &dbg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_config_home() {
        let expected_key = "APOLLO_CONFIG_HOME";
        assert_eq!(&RoverEnvKey::ConfigHome.to_string(), expected_key);
    }

    #[test]
    fn it_can_set_and_read_from_mock() {
        let expected_value = "hey whats the big idea anyway!??";
        let key = RoverEnvKey::ConfigHome;
        let mut env_store = RoverEnv::new();
        env_store.insert(key, expected_value);
        let actual_value = env_store.get(key).unwrap().unwrap();
        assert_eq!(expected_value, &actual_value)
    }

    #[test]
    fn it_can_remove_from_mock() {
        let expected_value = "hey whats the big idea anyway!??";
        let key = RoverEnvKey::ConfigHome;
        let mut env_store = RoverEnv::new();
        env_store.insert(key, expected_value);
        let actual_value = env_store.get(key).unwrap().unwrap();
        assert_eq!(expected_value, &actual_value);
        env_store.remove(key);
        let expected_value = None;
        let actual_value = env_store.get(key).unwrap();
        assert_eq!(expected_value, actual_value);
    }
}
