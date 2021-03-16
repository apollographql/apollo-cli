//! Schema code generation module used to work with Introspection result.
use crate::query::graph::introspect;
use sdl_encoder::{
    Directive, EnumDef, Field, FieldType, ObjectDef, ScalarDef, Schema as SDL, Union,
};
use serde::Deserialize;
use std::convert::TryFrom;

pub type Introspection = introspect::introspection_query::ResponseData;
pub type SchemaTypes = introspect::introspection_query::IntrospectionQuerySchemaTypes;
pub type SchemaDirectives = introspect::introspection_query::IntrospectionQuerySchemaDirectives;
pub type FullTypeFields = introspect::introspection_query::FullTypeFields;
pub type __TypeKind = introspect::introspection_query::__TypeKind;

/// A representation of a GraphQL Schema.
///
/// Contains schema Types and Directives.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    types: Vec<SchemaTypes>,
    directives: Vec<SchemaDirectives>,
}

impl Schema {
    /// Encode Schema into an SDL.
    pub fn encode(self) -> String {
        let mut sdl = SDL::new();

        for directive in self.directives {
            Self::encode_directives(directive, &mut sdl)
        }

        for type_ in self.types {
            Self::encode_full_type(type_, &mut sdl)
        }

        sdl.finish()
    }

    fn encode_directives(directive: SchemaDirectives, sdl: &mut SDL) {
        let mut directive_ = Directive::new(directive.name);
        directive_.description(directive.description);
        for location in directive.locations {
            // Location is of a __DirectiveLocation enum that doesn't implement
            // Display (meaning we can't just do .to_string). This next line
            // just forces it into a String with format! debug.
            directive_.location(format!("{:?}", location));
        }

        sdl.directive(directive_)
    }

    fn encode_full_type(type_: SchemaTypes, sdl: &mut SDL) {
        match type_.full_type.kind {
            __TypeKind::OBJECT => {
                let mut object_def =
                    ObjectDef::new(type_.full_type.name.unwrap_or_else(String::new));
                if let Some(field) = type_.full_type.fields {
                    for f in field {
                        let field_def = Self::encode_field(f);
                        object_def.field(field_def);
                    }
                    sdl.object(object_def);
                }
            }
            __TypeKind::SCALAR => {
                let mut scalar_def =
                    ScalarDef::new(type_.full_type.name.unwrap_or_else(String::new));
                scalar_def.description(type_.full_type.description);
                sdl.scalar(scalar_def);
            }
            __TypeKind::UNION => {
                let mut union_def = Union::new(type_.full_type.name.unwrap_or_else(String::new));
                union_def.description(type_.full_type.description);
                if let Some(possible_types) = type_.full_type.possible_types {
                    for ty in possible_types {
                        union_def.member(ty.type_ref.name.unwrap_or_else(String::new));
                    }
                }
                sdl.union(union_def);
            }
            __TypeKind::ENUM => {
                let mut enum_def = EnumDef::new(type_.full_type.name.unwrap_or_else(String::new));
                if let Some(enums) = type_.full_type.enum_values {
                    for enum_ in enums {
                        enum_def.value(enum_.name);
                    }
                }
                sdl.enum_(enum_def);
            }
            _ => (),
        }
    }

    fn encode_field(field: FullTypeFields) -> Field {
        let ty = Self::encode_type(field.type_.type_ref);
        let mut field_def = Field::new(field.name, ty);
        field_def.description(field.description);
        field_def
    }

    fn encode_type(ty: impl introspect::OfType) -> FieldType {
        use introspect::introspection_query::__TypeKind::*;
        match ty.kind() {
            SCALAR | OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT => FieldType::Type {
                ty: ty.name().unwrap().to_string(),
                is_nullable: true,
                default: None,
            },
            NON_NULL => {
                let mut ty = Self::encode_type(ty.of_type().unwrap());
                ty.set_is_nullable(false);
                ty
            }
            LIST => {
                let ty = Self::encode_type(ty.of_type().unwrap());
                FieldType::List {
                    ty: Box::new(ty),
                    is_nullable: false,
                }
            }
            Other(ty) => panic!("Unknown type: {}", ty),
        }
    }
}

impl TryFrom<Introspection> for Schema {
    type Error = &'static str;

    fn try_from(src: Introspection) -> Result<Self, Self::Error> {
        match src.schema {
            Some(s) => Ok(Self {
                types: s.types,
                directives: s.directives,
            }),
            None => Err("Schema not found in Introspection Result."),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_build_simple_schema() {}
}
