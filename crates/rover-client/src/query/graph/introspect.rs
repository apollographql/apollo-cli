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

pub trait OfType {
    type TypeRef: OfType;

    fn kind(&self) -> &introspection_query::__TypeKind;
    fn name(&self) -> Option<&str>;
    fn of_type(self) -> Option<Self::TypeRef>;
}

macro_rules! impl_of_type {
    ($target:ty, $assoc:ty) => {
        impl OfType for $target {
            type TypeRef = $assoc;

            fn kind(&self) -> &introspection_query::__TypeKind {
                &self.kind
            }

            fn name(&self) -> Option<&str> {
                self.name.as_deref()
            }

            fn of_type(self) -> Option<Self::TypeRef> {
                self.of_type
            }
        }
    };
}

impl_of_type!(
    introspection_query::TypeRef,
    introspection_query::TypeRefOfType
);

impl_of_type!(
    introspection_query::TypeRefOfType,
    introspection_query::TypeRefOfTypeOfType
);

impl_of_type!(
    introspection_query::TypeRefOfTypeOfType,
    introspection_query::TypeRefOfTypeOfTypeOfType
);

impl_of_type!(
    introspection_query::TypeRefOfTypeOfTypeOfType,
    introspection_query::TypeRefOfTypeOfTypeOfTypeOfType
);

impl_of_type!(
    introspection_query::TypeRefOfTypeOfTypeOfTypeOfType,
    introspection_query::TypeRefOfTypeOfTypeOfTypeOfTypeOfType
);

impl_of_type!(
    introspection_query::TypeRefOfTypeOfTypeOfTypeOfTypeOfType,
    introspection_query::TypeRefOfTypeOfTypeOfTypeOfTypeOfTypeOfType
);

impl_of_type!(
    introspection_query::TypeRefOfTypeOfTypeOfTypeOfTypeOfTypeOfType,
    introspection_query::TypeRefOfTypeOfTypeOfTypeOfTypeOfTypeOfTypeOfType
);

// NOTE(lrlna): This is a **hack**. This makes sure that the last possible
// generated ofType by graphql_client can return a None for of_type method.
impl OfType for introspection_query::TypeRefOfTypeOfTypeOfTypeOfTypeOfTypeOfTypeOfType {
    type TypeRef = introspection_query::TypeRefOfTypeOfTypeOfTypeOfTypeOfTypeOfTypeOfType;

    fn kind(&self) -> &introspection_query::__TypeKind {
        &self.kind
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn of_type(self) -> Option<Self::TypeRef> {
        None
    }
}
