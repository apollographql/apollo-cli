use crate::blocking::Client;
use crate::RoverClientError;
use graphql_client::*;

// I'm not sure where this should live long-term
/// this is because of the custom GraphQLDocument scalar in the schema
type GraphQLDocument = String;

#[derive(GraphQLQuery)]
// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[graphql(
    query_path = "src/query/schema/get.graphql",
    schema_path = "schema.graphql",
    response_derives = "PartialEq, Debug, Serialize, Deserialize",
    deprecated = "warn"
)]
/// This struct is used to generate the module containing `Variables` and
/// `ResponseData` structs.
/// Snake case of this name is the mod name. i.e. get_schema_query
pub struct GetSchemaQuery;

/// The main function to be used from this module. This function fetches a
/// schema from apollo studio and returns it in either sdl (default) or json format
pub fn run(
    variables: get_schema_query::Variables,
    client: Client,
) -> Result<String, RoverClientError> {
    let res = client.post::<GetSchemaQuery>(variables);

    // if asking for a json response, try serializing the schema
    // first unwrap the Result<Option<ResponseData>>
    let data = res.expect("Error fetching schema");
    let data = data.expect("No data in response when trying to fetch schema");

    // get the schema document from ResponseData
    let schema = data.service.expect("Service not found in response").schema;
    let sdl = schema.expect("No schema found for this variant").document;

    // if we want json, we can parse & serialize it here

    Ok(sdl)
}
