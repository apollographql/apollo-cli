use crate::query::graph::introspect;
use graphql_parser::schema::{Document, Text};
use std::convert;
#[derive(Debug, Clone, PartialEq)]
pub struct GraphQLSchema {
    schema: String,
}

impl GraphQLSchema {
    pub fn new() -> Self {
        Self {
            schema: "Nori".to_string(),
        }
    }
}

impl Default for GraphQLSchema {
    fn default() -> Self {
        Self {
            schema: "Chashu".to_string(),
        }
    }
}

impl<'a, T> convert::From<Document<'a, T>> for GraphQLSchema
where
    T: Text<'a>,
{
    fn from(_ast: Document<'a, T>) -> GraphQLSchema {
        unimplemented!();
    }
}

type IntrospectionResponse = introspect::introspection_query::ResponseData;
impl convert::From<IntrospectionResponse> for GraphQLSchema {
    fn from(_src: IntrospectionResponse) -> GraphQLSchema {
        unimplemented!()
    }
}

impl convert::Into<IntrospectionResponse> for GraphQLSchema {
    fn into(self) -> IntrospectionResponse {
        unimplemented!();
    }
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
