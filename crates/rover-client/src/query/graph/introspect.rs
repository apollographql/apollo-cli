use std::{collections::HashMap, convert::TryFrom};

use crate::blocking::Client;
use crate::introspection::Schema;
use crate::RoverClientError;
use graphql_client::*;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "src/query/graph/introspect_query.graphql",
    schema_path = "src/query/graph/introspect_schema.graphql",
    response_derives = "PartialEq, Debug, Serialize, Deserialize",
    deprecated = "warn"
)]

/// This struct is used to generate the module containing `Variables` and
/// `ResponseData` structs.
/// Snake case of this name is the mod name. i.e. introspection_query
pub struct IntrospectionQuery;

/// this struct contains all the info needed to print the result of the delete.
/// `updated_gateway` is true when composition succeeds and the gateway config
/// is updated for the gateway to consume. `composition_errors` is just a list
/// of strings for when there are composition errors as a result of the delete.
#[derive(Debug, PartialEq)]
pub struct IntrospectionResponse {
    pub result: String,
}

/// The main function to be used from this module. This function fetches a
/// schema from apollo studio and returns it in either sdl (default) or json format
pub fn run(client: &Client) -> Result<IntrospectionResponse, RoverClientError> {
    let variables = introspection_query::Variables {};
    let response_data = client.post::<IntrospectionQuery>(variables, &HashMap::new())?;
    // println!("{:?}", &response_data);
    // println!("{:?}", serde_json::to_string_pretty(&response_data)?);
    build_response(response_data)
}

fn build_response(
    response: introspection_query::ResponseData,
) -> Result<IntrospectionResponse, RoverClientError> {
    // todo!()
    match Schema::try_from(response) {
        Ok(schema) => Ok(IntrospectionResponse {
            result: schema.encode(),
        }),
        Err(msg) => Err(RoverClientError::IntrospectionError { msg: msg.into() }),
    }
}
