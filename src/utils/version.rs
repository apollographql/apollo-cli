use std::{fs, time::SystemTime};

use ansi_term::Colour::{Cyan, Yellow};
use billboard::{Alignment, Billboard};
use camino::Utf8PathBuf;
use semver::Version;

use crate::{Result, PKG_VERSION};
use houston as config;
use rover_client::releases::get_latest_release;

const ONE_HOUR: u64 = 60 * 60;
const ONE_DAY: u64 = ONE_HOUR * 24;

/// check for newer versions of rover.
///
/// If this fn is run explicitly from a user-facing command, we pass `force` to
/// check for newer versions, even if we recently checked for updates.
///
/// If `force` is not passed, we check for updates every day at most
pub fn check_for_update(config: config::Config, force: bool) -> Result<()> {
    let version_file = config.home.join("version.toml");
    let current_time = SystemTime::now();
    // if we don't end up checking, we don't want to overwrite the last checked time
    let mut checked = false;

    // check fs for last check time
    let last_checked_time = get_last_checked_time_from_disk(&version_file);

    match last_checked_time {
        Some(last_checked_time) => {
            let time_since_check = current_time.duration_since(last_checked_time)?.as_secs();
            tracing::debug!(
                "Time since last update check: {:?}h",
                time_since_check / ONE_HOUR
            );

            if force || time_since_check > ONE_DAY {
                do_update_check(&mut checked)?;
            } else {
                tracing::debug!(
                    "No need to check for updates. Automatic checks happen once per day"
                );
            }
        }
        // we haven't checked for updates before -- check now :)
        None => {
            do_update_check(&mut checked)?;
        }
    }

    if checked {
        tracing::debug!("Checked for available updates. Writing current time to disk");
        fs::write(&version_file, toml::to_string(&current_time)?)?;
    }

    Ok(())
}

fn do_update_check(checked: &mut bool) -> Result<()> {
    let latest = get_latest_release()?;
    let update_available = is_latest_newer(&latest, PKG_VERSION)?;

    if update_available {
        let message = format!(
            "There is a newer version of Rover available: {} (currently running v{})\n\nFor instructions on how to install, run {}", 
            Cyan.normal().paint(format!("v{}", latest)), 
            PKG_VERSION,
            Yellow.normal().paint("`rover docs open start`")
        );
        Billboard::builder()
            .box_alignment(Alignment::Left)
            .build()
            .eprint(message);
    }

    *checked = true;
    Ok(())
}

fn get_last_checked_time_from_disk(version_file: &Utf8PathBuf) -> Option<SystemTime> {
    match fs::read_to_string(&version_file) {
        Ok(contents) => match toml::from_str(&contents) {
            Ok(last_checked_version) => Some(last_checked_version),
            Err(_) => {
                tracing::debug!("Failed to parse last update check time from version file");
                None
            }
        },
        Err(_) => {
            tracing::debug!("Failed to read version file containing last update check time");
            None
        }
    }
}

fn is_latest_newer(latest: &str, running: &str) -> Result<bool> {
    let latest = Version::parse(latest)?;
    let running = Version::parse(running)?;
    Ok(latest > running)
}
