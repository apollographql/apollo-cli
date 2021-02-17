use crate::query::graph::introspect;
use graphql_parser::schema::{Document, Text};
use serde::Deserialize;
use std::convert;

pub type Introspection = introspect::introspection_query::ResponseData;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    query_type: Option<SchemaQueryType>,
    mutation_type: Option<SchemaMutationType>,
    subscription_type: Option<>,
    types: Option<Vec<Option<SchemaTypes>>>,
    description: Option<String>,
    directives: Option<Vec<Option<SchemaDirectives>>>,
}

impl Schema {
    pub fn with_introspection(src: Introspection) -> Self {
        if let Some(schema) = src.schema {
            Schema {
                query_type: SchemaQueryType {
                    name: schema.query_type.name,
                },
                mutation_type: SchemaMutationType {
                    name: schema.mutation_type.name,
                }
                subscription_type: schema.subscription_type,
                types: schema_types(&schema),
                description: schema.description,
                directives: schema_directives(&schema),
            }
        } else {
            unimplemented();
        }
    }
}

// impl Default for Schema {
//     fn default() -> Self {
//         Self {
//             schema: "Chashu".to_string(),
//         }
//     }
// }

impl<'a, T> convert::From<Document<'a, T>> for Schema
where
    T: Text<'a>,
{
    fn from(_ast: Document<'a, T>) -> Schema {
        unimplemented!();
    }
}

impl convert::From<Introspection> for Schema {
    fn from(src: Introspection) -> Schema {
        Schema::with_introspection(src)
    }
}

impl convert::Into<Introspection> for Schema {
    fn into(self) -> Introspection {
        unimplemented!();
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaQueryType {
    pub name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaMutationType {
    pub name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaSubscriptionType {
    pub name: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDirectives {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaTypes {
    #[serde(flatten)]
    pub type_map: TypeMap,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeMap {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use graphql_parser::schema::parse_schema;

    #[test]
    fn it_build_simple_schema() {
        // let ast_from_sdl = parse_schema::<String>(
        //     r#"
        //     """
        //     Simple schema
        //     """
        //     schema {
        //         query: Simple
        //     }
        //     """
        //     This is a simple type
        //     """
        //     type Simple {
        //         """
        //         This is a string field
        //         """
        //         string: String
        //     }
        // "#,
        // )
        // .unwrap()
        // .to_owned();
        let ast = parse_schema::<String>(
            r#"
            schema {
                query: Query
            }
            type Query {
                users: [User!]!,
            }
            """
            Example user object

            This is just a demo comment.
            """
            type User {
                name: String!,
            }
        "#,
        )
        .unwrap()
        .to_owned();
        dbg!(ast);
        // dbg!(ast_from_sdl);
        // assert_eq!("cat", "cat");
    }
}
