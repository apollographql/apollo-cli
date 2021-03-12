//! SDL Encoder provides methods to serialise a GraphQL Schema.
//! For mor information on GraphQL Schema Types, please refer to [official
//! documentation](https://graphql.org/learn/schema/).
//!
//! ## Example
//! ```rust
//! use sdl_encoder::{Schema, ObjectDef, SchemaDef, Field, InputDef, ScalarDef, EnumDef, FieldType};
//! use indoc::indoc;
//!
//! let mut schema = Schema::new();

//! // create a field
//! let field_type = FieldType::Type {
//!     ty: "String".to_string(),
//!     is_nullable: false,
//!     default: None,
//! };
//! let mut field = Field::new("cat".to_string(), field_type);
//! field.description(Some("Very good cats".to_string()));

//! // a schema definition
//! let mut schema_def = SchemaDef::new(field.clone());
//! schema_def.description(Some("Simple schema".to_string()));
//! schema.schema(schema_def);

//! // object type defintion
//! let mut object_def = ObjectDef::new("Query".to_string());
//! object_def.description(Some("Example Query type".to_string()));
//! object_def.field(field.clone());
//! schema.object(object_def);

//! // enum definition
//! let mut enum_ = EnumDef::new("VeryGoodCats".to_string());
//! enum_.variant("NORI".to_string());
//! enum_.variant("CHASHU".to_string());
//! schema.enum_(enum_);

//! let mut scalar = ScalarDef::new("NoriCacheControl".to_string());
//! scalar.description(Some("Scalar description".to_string()));
//! schema.scalar(scalar);

//! // input definition
//! let input_def = InputDef::new("SpaceCat".to_string(), field);
//! schema.input(input_def);

//! assert_eq!(
//!     schema.finish(),
//!     indoc! { r#"
//!         """
//!         Simple schema
//!         """
//!         schema {
//!           """
//!           Very good cats
//!           """
//!           cat: String!
//!         }
//!         """
//!         Example Query type
//!         """
//!         type Query {
//!           """
//!           Very good cats
//!           """
//!           cat: String!
//!         }
//!         enum VeryGoodCats {
//!           NORI
//!           CHASHU
//!         }
//!         """
//!         Scalar description
//!         """
//!         scalar NoriCacheControl
//!         input SpaceCat {
//!           """
//!           Very good cats
//!           """
//!           cat: String!
//!         }
//!     "# }
//! );
//! ```

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, future_incompatible, unreachable_pub, rust_2018_idioms)]

mod field;
pub use field::Field;

mod field_type;
pub use field_type::FieldType;

mod object_def;
pub use object_def::ObjectDef;

mod scalar_def;
pub use scalar_def::ScalarDef;

mod enum_def;
pub use enum_def::EnumDef;

mod input_def;
pub use input_def::InputDef;

mod schema_def;
pub use schema_def::SchemaDef;

/// An SDL representation of a GraphQLSchema.
#[derive(Debug)]
pub struct Schema {
    buf: String,
}

impl Schema {
    /// Creates a new instance of Schema.
    pub fn new() -> Self {
        Self { buf: String::new() }
    }

    /// Adds a new Type Definition.
    pub fn object(&mut self, object: ObjectDef) {
        self.buf.push_str(&object.to_string());
    }

    /// Adds a new Schema Definition.
    ///
    /// The schema type is only used when the root GraphQL type is different
    /// from default GraphQL types.
    pub fn schema(&mut self, schema: SchemaDef) {
        self.buf.push_str(&schema.to_string());
    }

    /// Adds a new Input object definition
    pub fn input(&mut self, input: InputDef) {
        self.buf.push_str(&input.to_string());
    }

    /// Adds a new Enum type definition
    pub fn enum_(&mut self, enum_: EnumDef) {
        self.buf.push_str(&enum_.to_string());
    }

    /// Adds a new Enum type definition
    pub fn scalar(&mut self, scalar: ScalarDef) {
        self.buf.push_str(&scalar.to_string());
    }

    /// Return the encoded SDL string after all types have been
    pub fn finish(self) -> String {
        self.buf
    }
}

impl Default for Schema {
    fn default() -> Self {
        Schema::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn smoke_test() {
        let mut schema = Schema::new();

        // create a field
        let field_type = FieldType::Type {
            ty: "String".to_string(),
            is_nullable: false,
            default: None,
        };
        let mut field = Field::new("cat".to_string(), field_type);
        field.description(Some("Very good cats".to_string()));

        // a schema definition
        let mut schema_def = SchemaDef::new(field.clone());
        schema_def.description(Some("Simple schema".to_string()));
        schema.schema(schema_def);

        // object type defintion
        let mut object_def = ObjectDef::new("Query".to_string());
        object_def.description(Some("Example Query type".to_string()));
        object_def.field(field.clone());
        schema.object(object_def);

        // enum definition
        let mut enum_ = EnumDef::new("VeryGoodCats".to_string());
        enum_.variant("NORI".to_string());
        enum_.variant("CHASHU".to_string());
        schema.enum_(enum_);

        let mut scalar = ScalarDef::new("NoriCacheControl".to_string());
        scalar.description(Some("Scalar description".to_string()));
        schema.scalar(scalar);

        // input definition
        let input_def = InputDef::new("SpaceCat".to_string(), field);
        schema.input(input_def);

        assert_eq!(
            schema.finish(),
            indoc! { r#"
                """
                Simple schema
                """
                schema {
                  """
                  Very good cats
                  """
                  cat: String!
                }
                """
                Example Query type
                """
                type Query {
                  """
                  Very good cats
                  """
                  cat: String!
                }
                enum VeryGoodCats {
                  NORI
                  CHASHU
                }
                """
                Scalar description
                """
                scalar NoriCacheControl
                input SpaceCat {
                  """
                  Very good cats
                  """
                  cat: String!
                }
            "# }
        );
    }

    #[test]
    fn smoke_test_2() {
        let mut schema = Schema::new();

        let field_type_1 = FieldType::Type {
            ty: "SpaceCatEnum".to_string(),
            is_nullable: true,
            default: None,
        };

        let field_type_2 = FieldType::List {
            ty: Box::new(field_type_1.clone()),
            is_nullable: false,
        };

        let object_field = Field::new("cat".to_string(), field_type_2);
        let mut object_def = ObjectDef::new("Query".to_string());
        object_def.description(Some("Example Query type".to_string()));
        object_def.field(object_field);

        let mut schema_field = Field::new("treat".to_string(), field_type_1);
        schema_field.description(Some("Good cats get treats".to_string()));
        let mut schema_def = SchemaDef::new(schema_field);
        schema_def.description(Some("Example schema Def".to_string()));
        schema.schema(schema_def);
        schema.object(object_def);
        assert_eq!(
            schema.finish(),
            indoc! { r#"
                """
                Example schema Def
                """
                schema {
                  """
                  Good cats get treats
                  """
                  treat: SpaceCatEnum
                }
                """
                Example Query type
                """
                type Query {
                  cat: [SpaceCatEnum]!
                }
        "#}
        )
    }
}
