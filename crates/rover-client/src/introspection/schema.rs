//! Schema code generation module used to work with Introspection result.
use crate::query::graph::introspect;
use graphql_parser::schema::{Document, Text};
use serde::Deserialize;
use std::convert;

pub type Introspection = introspect::introspection_query::ResponseData;
pub type SchemaTypes = introspect::introspection_query::IntrospectionQuerySchemaTypes;
pub type SchemaDirectives = introspect::introspection_query::IntrospectionQuerySchemaDirectives;

// TODO: @lrlna it would be *really* nice for this to have a Clone derive.
// Since at this point we are using graphql_client's introspection types, and
// they don't provide a clone implementation, we need to figure out a way to
// cast the types provided to us to our own types and then create our own clone
// impl. Maybe??

/// A representation of a GraphQL Schema.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    types: Vec<SchemaTypes>,
    directives: Vec<SchemaDirectives>,
}

impl Schema {
    // todo: @lrlna this could perhaps be private, since its likely to only be
    // used in `Schema::from(introspection_result)` form.

    /// Create an instance of Schema with an Introspection Result.
    pub fn with_introspection(src: Introspection) -> Self {
        if let Some(schema) = src.schema {
            return Schema {
                types: schema.types,
                directives: schema.directives,
            };
        }
        unimplemented!()
    }
}

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

#[cfg(test)]
mod tests {
    use graphql_parser::schema::parse_schema;

    #[test]
    fn it_build_simple_schema() {
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
    }
}
