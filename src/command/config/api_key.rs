use anyhow::{Error, Result};
use config::Profile;
use console::{self, style};
use houston as config;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ApiKey {
    #[structopt(long = "profile", default_value = "default")]
    profile_name: String,
}

impl ApiKey {
    pub fn run(&self) -> Result<()> {
        let api_key = api_key_prompt()?;
        Profile::set_api_key(&self.profile_name, api_key)?;
        Profile::get_api_key(&self.profile_name).map(|_| {
            log::info!("Successfully saved API key.");
        })
    }
}

fn api_key_prompt() -> Result<String> {
    let term = console::Term::stdout();
    log::info!(
        "Go to {} and create a new Personal API Key.",
        style("https://studio.apollographql.com/user-settings").cyan()
    );
    log::info!("Copy the key and paste it into the prompt below.");
    let api_key = term.read_secure_line()?;
    if is_valid(&api_key) {
        Ok(api_key)
    } else {
        Err(Error::msg("Received an empty API Key. Please try again."))
    }
}

fn is_valid(api_key: &str) -> bool {
    !api_key.is_empty()
}

#[cfg(test)]
mod tests {
    use assert_fs::TempDir;
    use serial_test::serial;

    use houston::Profile;

    const DEFAULT_PROFILE: &str = "default";
    const DEFAULT_KEY: &str = "default-key";

    const CUSTOM_PROFILE: &str = "custom";
    const CUSTOM_KEY: &str = "custom-key";

    #[test]
    #[serial]
    fn it_can_set_default_api_key() {
        let temp = TempDir::new().unwrap();
        std::env::set_var("APOLLO_CONFIG_HOME", temp.path());

        Profile::set_api_key(DEFAULT_PROFILE, DEFAULT_KEY.into()).unwrap();
        let result = Profile::get_api_key(DEFAULT_PROFILE).unwrap();
        assert_eq!(result, DEFAULT_KEY);
    }

    #[test]
    #[serial]
    fn it_can_set_custom_api_key() {
        let temp = TempDir::new().unwrap();
        std::env::set_var("APOLLO_CONFIG_HOME", temp.path());

        Profile::set_api_key(CUSTOM_PROFILE, CUSTOM_KEY.into()).unwrap();
        let result = Profile::get_api_key(CUSTOM_PROFILE).unwrap();
        assert_eq!(result, CUSTOM_KEY);
    }
}