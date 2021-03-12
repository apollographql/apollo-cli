mod code;
mod suggestion;

pub(crate) use code::Code;
pub(crate) use suggestion::Suggestion;

use houston::HoustonProblem;
use rover_client::RoverClientError;

use crate::utils::env::RoverEnvKey;

use std::{env, fmt::Display};

/// Metadata contains extra information about specific errors
/// Currently this includes an optional error `Code`
/// and an optional `Suggestion`
#[derive(Default, Debug)]
pub struct Metadata {
    pub suggestion: Option<Suggestion>,
    pub code: Option<Code>,
    pub is_parse_error: bool,
}

impl Metadata {
    pub fn parse_error(suggestion: impl Display) -> Self {
        Metadata {
            suggestion: Some(Suggestion::Adhoc(suggestion.to_string())),
            code: None,
            is_parse_error: true,
        }
    }
}

/// `Metadata` structs can be created from an `anyhow::Error`
/// This works by downcasting the errors to their underlying types
/// and creating `Suggestion`s and `Code`s where applicable
impl From<&mut anyhow::Error> for Metadata {
    fn from(error: &mut anyhow::Error) -> Self {
        if let Some(rover_client_error) = error.downcast_ref::<RoverClientError>() {
            let (suggestion, code) = match rover_client_error {
                RoverClientError::InvalidJSON(_)
                | RoverClientError::InvalidHeaderName(_)
                | RoverClientError::InvalidHeaderValue(_)
                | RoverClientError::SendRequest(_)
                | RoverClientError::MalformedResponse { null_field: _ }
                | RoverClientError::InvalidSeverity => (Some(Suggestion::SubmitIssue), None),
                RoverClientError::ExpectedFederatedGraph { graph: _ } => {
                    (Some(Suggestion::UseFederatedGraph), None)
                }
                RoverClientError::NoSchemaForVariant {
                    graph,
                    invalid_variant: _,
                } => (
                    Some(Suggestion::RunGraphList {
                        graph: graph.to_string(),
                    }),
                    None,
                ),
                RoverClientError::NoSubgraphInGraph {
                    invalid_subgraph: _,
                    valid_subgraphs,
                } => (
                    Some(Suggestion::ProvideValidSubgraph(valid_subgraphs.clone())),
                    None,
                ),
                RoverClientError::NoService { graph: _ } => {
                    (Some(Suggestion::CheckGraphNameAndAuth), None)
                }
                RoverClientError::AdhocError { msg: _ }
                | RoverClientError::GraphQL { msg: _ }
                | RoverClientError::IntrospectionError { msg: _ } => (None, None),
                RoverClientError::InvalidKey => (Some(Suggestion::CheckKey), None),
            };
            return Metadata {
                suggestion,
                code,
                is_parse_error: false,
            };
        }

        if let Some(houston_problem) = error.downcast_ref::<HoustonProblem>() {
            let (suggestion, code) = match houston_problem {
                HoustonProblem::NoNonSensitiveConfigFound(_) => {
                    (Some(Suggestion::RerunWithSensitive), None)
                }
                HoustonProblem::CouldNotCreateConfigHome(_)
                | HoustonProblem::DefaultConfigDirNotFound
                | HoustonProblem::InvalidOverrideConfigDir(_) => {
                    (Some(Suggestion::SetConfigHome), None)
                }
                HoustonProblem::NoConfigFound(_) => {
                    let code = None;
                    let suggestion = if env::var_os(RoverEnvKey::ConfigHome.to_string()).is_some() {
                        Some(Suggestion::MigrateConfigHomeOrCreateConfig)
                    } else {
                        Some(Suggestion::CreateConfig)
                    };
                    (suggestion, code)
                }
                HoustonProblem::ProfileNotFound(_) => (Some(Suggestion::ListProfiles), None),
                HoustonProblem::TomlDeserialization(_)
                | HoustonProblem::TomlSerialization(_)
                | HoustonProblem::IOError(_) => (Some(Suggestion::SubmitIssue), None),
            };
            return Metadata {
                suggestion,
                code,
                is_parse_error: false,
            };
        }

        Metadata::default()
    }
}
