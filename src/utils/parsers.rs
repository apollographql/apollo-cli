use regex::Regex;
use serde::Serialize;
use std::{convert::TryInto, fmt, path::PathBuf};

use crate::{error::RoverError, Result};
use url::Url;

#[derive(Debug, PartialEq)]
pub enum SchemaSource {
    Stdin,
    File(PathBuf),
}

pub fn parse_schema_source(loc: &str) -> Result<SchemaSource> {
    if loc == "-" {
        Ok(SchemaSource::Stdin)
    } else if loc.is_empty() {
        Err(RoverError::parse_error(
            "The path provided to find a schema is empty",
        ))
    } else {
        let path = PathBuf::from(loc);
        Ok(SchemaSource::File(path))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphRef {
    pub name: String,
    pub variant: String,
}

impl fmt::Display for GraphRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.variant)
    }
}

/// NOTE: THIS IS A TEMPORARY SOLUTION. IN THE FUTURE, ALL GRAPH ID PARSING
/// WILL HAPPEN IN THE BACKEND TO KEEP EVERYTHING CONSISTENT. THIS IS AN
/// INCOMPLETE PLACEHOLDER, AND MAY NOT COVER EVERY SINGLE VALID USE CASE
pub fn parse_graph_ref(graph_id: &str) -> Result<GraphRef> {
    let pattern = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]{0,63}$").unwrap();
    let variant_pattern = Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]{0,63})@(.{0,63})$").unwrap();

    let valid_graph_name_only = pattern.is_match(graph_id);
    let valid_graph_with_variant = variant_pattern.is_match(graph_id);

    if valid_graph_name_only {
        Ok(GraphRef {
            name: graph_id.to_string(),
            variant: "current".to_string(),
        })
    } else if valid_graph_with_variant {
        let matches = variant_pattern.captures(graph_id).unwrap();
        let name = matches.get(1).unwrap().as_str();
        let variant = matches.get(2).unwrap().as_str();
        Ok(GraphRef {
            name: name.to_string(),
            variant: variant.to_string(),
        })
    } else {
        Err(RoverError::parse_error("Graph IDs must be in the format <NAME> or <NAME>@<VARIANT>, where <NAME> can only contain letters, numbers, or the characters `-` or `_`, and must be 64 characters or less. <VARIANT> must be 64 characters or less."))
    }
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct ValidationPeriod {
    // these timestamps could be represented as i64, but the API expects
    // Option<String>
    pub from: Option<String>,
    pub to: Option<String>,
}

// Validation period is parsed as human readable time.
// such as "10m 50s"
pub fn parse_validation_period(period: &str) -> Result<ValidationPeriod> {
    // attempt to parse strings like
    // 15h 10m 2s into number of seconds
    if period.contains("ns") || period.contains("us") || period.contains("ms") {
        return Err(RoverError::parse_error(
            "You can only specify a duration as granular as seconds.",
        ));
    };
    let duration = humantime::parse_duration(period).map_err(RoverError::parse_error)?;
    let window: i64 = duration
        .as_secs()
        .try_into()
        .map_err(RoverError::parse_error)?;

    if window > 0 {
        Ok(ValidationPeriod {
            // search "from" a negative time window
            from: Some(format!("{}", -window)),
            // search "to" now (-0) seconds
            to: Some("-0".to_string()),
        })
    } else {
        Err(RoverError::parse_error(
            "The number of seconds must be a positive integer.",
        ))
    }
}

pub fn parse_query_count_threshold(threshold: &str) -> Result<i64> {
    let threshold = threshold.parse::<i64>()?;
    if threshold < 1 {
        Err(RoverError::parse_error(
            "The number of queries must be a positive integer.",
        ))
    } else {
        Ok(threshold)
    }
}

pub fn parse_query_percentage_threshold(threshold: &str) -> Result<f64> {
    let threshold = threshold.parse::<i64>()?;
    if !(0..=100).contains(&threshold) {
        Err(RoverError::parse_error(
            "Valid numbers are in the range 0 <= x <= 100",
        ))
    } else {
        Ok((threshold / 100) as f64)
    }
}

/// Parse Urls from the command line.
// TODO: @lrlna return error for parse url
pub fn parse_url(url: &str) -> Result<Url> {
    let res = Url::parse(url)?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::{parse_graph_ref, parse_schema_source, GraphRef, SchemaSource};

    #[test]
    fn it_correctly_parses_stdin_flag() {
        assert_eq!(parse_schema_source("-").unwrap(), SchemaSource::Stdin);
    }

    #[test]
    fn it_correctly_parses_path_option() {
        let loc = parse_schema_source("./schema.graphql").unwrap();
        match loc {
            SchemaSource::File(buf) => {
                assert_eq!(buf.to_str().unwrap(), "./schema.graphql");
            }
            _ => panic!("parsed incorrectly as stdin"),
        }
    }

    #[test]
    fn it_errs_with_empty_path() {
        let loc = parse_schema_source("");
        assert!(loc.is_err());
    }

    #[test]
    fn parse_graph_ref_works() {
        assert!(parse_graph_ref("engine#%^").is_err());
        assert!(parse_graph_ref(
            "1234567890123456789012345678901234567890123456789012345678901234567890"
        )
        .is_err());
        assert!(parse_graph_ref("1boi").is_err());
        assert!(parse_graph_ref("_eng").is_err());
        assert!(parse_graph_ref(
            "engine@1234567890123456789012345678901234567890123456789012345678901234567890"
        )
        .is_err());
        assert!(parse_graph_ref(
            "engine1234567890123456789012345678901234567890123456789012345678901234567890@prod"
        )
        .is_err());

        assert_eq!(
            parse_graph_ref("engine@okay").unwrap(),
            GraphRef {
                name: "engine".to_string(),
                variant: "okay".to_string()
            }
        );
        assert_eq!(
            parse_graph_ref("studio").unwrap(),
            GraphRef {
                name: "studio".to_string(),
                variant: "current".to_string()
            }
        );
        assert_eq!(
            parse_graph_ref("this_should_work").unwrap(),
            GraphRef {
                name: "this_should_work".to_string(),
                variant: "current".to_string()
            }
        );
        assert_eq!(
            parse_graph_ref("it-is-cool@my-special/variant:from$hell").unwrap(),
            GraphRef {
                name: "it-is-cool".to_string(),
                variant: "my-special/variant:from$hell".to_string()
            }
        );
    }
}
