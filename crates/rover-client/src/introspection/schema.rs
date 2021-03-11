//! Schema code generation module used to work with Introspection result.
use crate::query::graph::introspect;
use graphql_parser::schema::{Document, Text};
use sdl_encoder::{EnumDef, Field, FieldType, ObjectDef, ScalarDef, Schema as SDL};
use serde::Deserialize;
use std::convert;

pub type Introspection = introspect::introspection_query::ResponseData;
pub type SchemaTypes = introspect::introspection_query::IntrospectionQuerySchemaTypes;
pub type __TypeKind = introspect::introspection_query::__TypeKind;
// pub type FullType = introspect::introspection_query::IntrospectionQuerySchemaTypes::FullType;
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
            Schema {
                types: schema.types,
                directives: schema.directives,
            }
        } else {
            todo!()
        }
    }

    pub fn encode_schema(self) -> String {
        let mut sdl = SDL::new();
        for type_ in self.types {
            match type_.full_type.kind {
                __TypeKind::OBJECT => {
                    let mut object_def =
                        ObjectDef::new(type_.full_type.name.unwrap_or("".to_string()));
                    if let Some(field) = type_.full_type.fields {
                        for f in field {
                            match f.type_.type_ref.kind {
                                __TypeKind::SCALAR => {
                                    let field_type = FieldType::Type {
                                        ty: f.type_.type_ref.name.unwrap_or("".to_string()),
                                        is_nullable: true,
                                        default: None,
                                    };
                                    let mut field_def = Field::new(f.name, field_type);
                                    field_def.description(f.description);
                                    object_def.field(field_def);
                                }
                                __TypeKind::NON_NULL => (),
                                __TypeKind::OBJECT => (),
                                __TypeKind::INTERFACE => (),
                                __TypeKind::UNION => (),
                                __TypeKind::ENUM => (),
                                __TypeKind::INPUT_OBJECT => (),
                                __TypeKind::LIST => (),
                                _ => (),
                            }
                        }
                        sdl.object(object_def);
                    }
                }
                __TypeKind::SCALAR => {
                    let mut scalar_def =
                        ScalarDef::new(type_.full_type.name.unwrap_or("".to_string()));
                    scalar_def.description(type_.full_type.description);
                    sdl.scalar(scalar_def);
                }
                __TypeKind::ENUM => {
                    let mut enum_def = EnumDef::new(type_.full_type.name.unwrap_or("".to_string()));
                    if let Some(enums) = type_.full_type.enum_values {
                        for enum_ in enums {
                            enum_def.variant(enum_.name);
                        }
                    }
                    sdl.enum_(enum_def);
                }
                _ => (),
            }
        }

        sdl.finish()
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
