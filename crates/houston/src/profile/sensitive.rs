use crate::{profile::Profile, Config, HoustonProblem};
use serde::{Deserialize, Serialize};

use std::fmt;
use std::fs;
use std::path::PathBuf;

/// Holds sensitive information regarding authentication.
#[derive(Debug, Serialize, Deserialize)]
pub struct Sensitive {
    pub api_key: String,
}

impl Sensitive {
    fn path(profile_name: &str, config: &Config) -> Result<PathBuf, HoustonProblem> {
        Ok(Profile::dir(profile_name, config)?.join(".sensitive"))
    }

    /// Serializes to toml and saves to file system at `$APOLLO_CONFIG_HOME/<profile_name>/.sensitive`.
    pub fn save(&self, profile_name: &str, config: &Config) -> Result<(), HoustonProblem> {
        let path = Sensitive::path(profile_name, config)?;
        let data = toml::to_string(self)?;

        if let Some(dirs) = &path.parent() {
            fs::create_dir_all(&dirs)?;
        }

        fs::write(&path, &data)?;
        log::debug!("saving API key to: \"{}\"", path.display());
        log::trace!("writing TOML:\n{}", &data);
        Ok(())
    }

    /// Opens and deserializes `$APOLLO_CONFIG_HOME/<profile_name>/.sensitive`.
    pub fn load(profile_name: &str, config: &Config) -> Result<Sensitive, HoustonProblem> {
        let path = Sensitive::path(profile_name, config)?;
        let data = fs::read_to_string(&path)?;
        log::debug!("loading API key from \"{}\"", path.display());
        log::trace!("TOML contents:\n{}", &data);
        Ok(toml::from_str(&data)?)
    }
}

impl fmt::Display for Sensitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "API Key: \"{}\"", self.api_key)
    }
}
