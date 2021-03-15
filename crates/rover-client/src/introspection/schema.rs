//! Schema code generation module used to work with Introspection result.
use crate::query::graph::introspect;
use sdl_encoder::{EnumDef, Field, FieldType, ObjectDef, ScalarDef, Schema as SDL};
use serde::Deserialize;
use std::convert::TryFrom;

pub type Introspection = introspect::introspection_query::ResponseData;
pub type SchemaTypes = introspect::introspection_query::IntrospectionQuerySchemaTypes;
pub type FullTypeFields = introspect::introspection_query::FullTypeFields;
pub type __TypeKind = introspect::introspection_query::__TypeKind;
pub type SchemaDirectives = introspect::introspection_query::IntrospectionQuerySchemaDirectives;

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
        for type_ in self.types {
            Self::encode_full_type(type_, &mut sdl)
        }

        sdl.finish()
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

    // NOTE(lrlna): We manually unroll our ofTypes instead of recursing here
    // because nested types coming in from Introspection Result are all
    // different ... types:
    //
    // TypeRefOfType
    // TypeRefOfTypeType
    // TypeRefOfTypeOfTypeOfType
    // TypeRefOfTypeOfTypeOfTypeOfType
    // TypeRefOfTypeOfTypeOfTypeOfTypeOfType
    fn encode_field(field: FullTypeFields) -> Field {
        use introspect::introspection_query::__TypeKind::*;
        let type_ref = field.type_.type_ref;

        match type_ref.kind {
            SCALAR | OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT => {
                let ty = FieldType::new_type(type_ref.name.unwrap(), true, None);
                let mut field_def = Field::new(field.name, ty);
                field_def.description(field.description);
                field_def
            }
            // Type!
            NON_NULL => {
                let type_ref = type_ref.of_type.unwrap();
                match type_ref.kind {
                    SCALAR | OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT => {
                        let ty = FieldType::new_type(type_ref.name.unwrap(), false, None);
                        let mut field_def = Field::new(field.name, ty);
                        field_def.description(field.description);
                        field_def
                    }
                    // [Type]!
                    LIST => {
                        let type_ref = type_ref.of_type.unwrap();
                        match type_ref.kind {
                            SCALAR | OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT => {
                                let ty = FieldType::new_type(type_ref.name.unwrap(), true, None);
                                let list = FieldType::new_list(ty, false);
                                let mut field_def = Field::new(field.name, list);
                                field_def.description(field.description);
                                field_def
                            }
                            // [[Type]!]
                            LIST => {
                                let type_ref = type_ref.of_type.unwrap();
                                match type_ref.kind {
                                    SCALAR | OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT => {
                                        let ty =
                                            FieldType::new_type(type_ref.name.unwrap(), true, None);
                                        let list1 = FieldType::new_list(ty, false);
                                        let list2 = FieldType::new_list(list1, true);
                                        let mut field_def = Field::new(field.name, list2);
                                        field_def.description(field.description);
                                        field_def
                                    }
                                    LIST => panic!("Cannot nest more than 2 lists."),
                                    // [[Type]!]!
                                    NON_NULL => {
                                        let type_ref = type_ref.of_type.unwrap();
                                        match type_ref.kind {
                                            SCALAR | OBJECT | INTERFACE | UNION | ENUM
                                            | INPUT_OBJECT => {
                                                let ty = FieldType::new_type(
                                                    type_ref.name.unwrap(),
                                                    true,
                                                    None,
                                                );
                                                let list1 = FieldType::new_list(ty, false);
                                                let list2 = FieldType::new_list(list1, false);
                                                let mut field_def = Field::new(field.name, list2);
                                                field_def.description(field.description);
                                                field_def
                                            }
                                            LIST => panic!("Cannot nest more than 2 lists."),
                                            ty => panic!("Unknown type: {:?}", ty),
                                        }
                                    }
                                    ty => panic!("Unknown type: {:?}", ty),
                                }
                            }
                            NON_NULL => {
                                let type_ref = type_ref.of_type.unwrap();
                                match type_ref.kind {
                                    SCALAR | OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT => {
                                        let field_type = FieldType::Type {
                                            ty: type_ref.name.unwrap_or_else(String::new),
                                            is_nullable: false,
                                            default: None,
                                        };
                                        let mut field_def = Field::new(field.name, field_type);
                                        field_def.description(field.description);
                                        field_def
                                    }
                                    LIST => panic!("Cannot nest more than 2 lists."),
                                    ty => panic!("Unknown type: {:?}", ty),
                                }
                            }
                            ty => panic!("Unknown type: {:?}", ty),
                        }
                    }
                    ty => panic!("Unknown type: {:?}", ty),
                }
            }
            // [Type]
            LIST => {
                let type_ref = type_ref.of_type.unwrap();
                match type_ref.kind {
                    SCALAR | OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT => {
                        let ty = FieldType::new_type(type_ref.name.unwrap(), true, None);
                        let list = FieldType::new_list(ty, true);
                        let mut field_def = Field::new(field.name, list);
                        field_def.description(field.description);
                        field_def
                    }
                    // [[Type]]
                    LIST => {
                        let type_ref = type_ref.of_type.unwrap();
                        match type_ref.kind {
                            SCALAR | OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT => {
                                let ty = FieldType::new_type(type_ref.name.unwrap(), true, None);
                                let list1 = FieldType::new_list(ty, true);
                                let list2 = FieldType::new_list(list1, true);
                                let mut field_def = Field::new(field.name, list2);
                                field_def.description(field.description);
                                field_def
                            }
                            LIST => panic!("Cannot nest more than 2 lists."),
                            // [[Type!]]
                            NON_NULL => {
                                let type_ref = type_ref.of_type.unwrap();
                                match type_ref.kind {
                                    SCALAR | OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT => {
                                        let ty = FieldType::new_type(
                                            type_ref.name.unwrap(),
                                            false,
                                            None,
                                        );
                                        let list1 = FieldType::new_list(ty, true);
                                        let list2 = FieldType::new_list(list1, true);
                                        let mut field_def = Field::new(field.name, list2);
                                        field_def.description(field.description);
                                        field_def
                                    }
                                    LIST => panic!("Cannot nest more than 2 lists."),
                                    ty => panic!("Unknown type: {:?}", ty),
                                }
                            }
                            ty => panic!("Unknown type: {:?}", ty),
                        }
                    }

                    //[Type!]
                    NON_NULL => {
                        let type_ref = type_ref.of_type.unwrap();
                        match type_ref.kind {
                            SCALAR | OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT => {
                                let ty = FieldType::new_type(type_ref.name.unwrap(), false, None);
                                let list1 = FieldType::new_list(ty, true);
                                let list2 = FieldType::new_list(list1, true);
                                let mut field_def = Field::new(field.name, list2);
                                field_def.description(field.description);
                                field_def
                            }
                            // [[Type]]!
                            LIST => {
                                let type_ref = type_ref.of_type.unwrap();
                                match type_ref.kind {
                                    SCALAR | OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT => {
                                        let ty =
                                            FieldType::new_type(type_ref.name.unwrap(), true, None);
                                        let list1 = FieldType::new_list(ty, true);
                                        let list2 = FieldType::new_list(list1, false);
                                        let mut field_def = Field::new(field.name, list2);
                                        field_def.description(field.description);
                                        field_def
                                    }
                                    LIST => panic!("Cannot nest more than 2 lists."),
                                    // [[Type!]]!
                                    NON_NULL => {
                                        let type_ref = type_ref.of_type.unwrap();
                                        match type_ref.kind {
                                            SCALAR | OBJECT | INTERFACE | UNION | ENUM
                                            | INPUT_OBJECT => {
                                                let ty = FieldType::new_type(
                                                    type_ref.name.unwrap(),
                                                    false,
                                                    None,
                                                );
                                                let list1 = FieldType::new_list(ty, true);
                                                let list2 = FieldType::new_list(list1, false);
                                                let mut field_def = Field::new(field.name, list2);
                                                field_def.description(field.description);
                                                field_def
                                            }
                                            LIST => panic!("Cannot nest more than 2 lists."),
                                            ty => panic!("Unknown type: {:?}", ty),
                                        }
                                    }
                                    ty => panic!("Unknown type: {:?}", ty),
                                }
                            }
                            ty => panic!("Unknown type: {:?}", ty),
                        }
                    }
                    ty => panic!("Unknown type: {:?}", ty),
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
