//! SDL Encoder provides methods to serialise a GraphQL Schema.
//! For mor information on GraphQL Schema Types, please refer to [official
//! documentation](https://graphql.org/learn/schema/).
//!
//! ## Example
//! ```rust
//! use sdl_encoder::{Schema, TypeDef, SchemaDef, Field, VecState};
//! use indoc::indoc;
//!
//! let mut schema = Schema::new();
//!
//! let mut type_field = Field::new("cat".to_string(), "SpaceCatEnum".to_string());
//! type_field.vec_state(VecState::NonNullable);
//! let mut type_ = TypeDef::new("Query".to_string(), type_field);
//! type_.description("Example Query type".to_string());
//!
//! let mut schema_field = Field::new("treat".to_string(), "String".to_string());
//! schema_field.description("Good cats get treats".to_string());
//! let mut schema_def = SchemaDef::new(schema_field);
//! schema_def.description("Example schema Def".to_string());
//! schema.schema(schema_def);
//! schema.type_(type_);
//! assert_eq!(schema.finish(), indoc! { r#"
//!     """
//!     Example schema Def
//!     """
//!     schema {
//!       """
//!       Good cats get treats
//!       """
//!       treat: String!,
//!     }
//!     """
//!     Example Query type
//!     """
//!     type Query {
//!       cat: [SpaceCatEnum!]!,
//!     }
//!
//! "#})
//! ```

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, future_incompatible, unreachable_pub, rust_2018_idioms)]

use std::fmt::{self, Display};

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
    pub fn type_(&mut self, type_: TypeDef) {
        self.buf.push_str(&type_.to_string());
    }

    /// Adds a new Schema Definition.
    ///
    /// The schema type is only used when the root GraphQL type is different
    /// from default GraphQL types.
    pub fn schema(&mut self, schema: SchemaDef) {
        self.buf.push_str(&schema.to_string());
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

/// Type Definition used to define different types in SDL.
#[derive(Debug)]
pub struct TypeDef {
    name: String,
    description: Option<String>,
    fields: Vec<Field>,
}

impl TypeDef {
    /// Create a new instance of TypeDef with a name.
    pub fn new(name: String, field: Field) -> Self {
        Self {
            name,
            description: None,
            fields: vec![field],
        }
    }

    /// Set the TypeDef's description field.
    pub fn description(&mut self, description: String) {
        self.description = Some(description)
    }

    /// Push a Field to type def's fields vector.
    pub fn field(&mut self, field: Field) {
        self.fields.push(field)
    }
}

impl Display for TypeDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            writeln!(f, "\"\"\"\n{}\n\"\"\"", description)?;
        }

        let mut fields = String::new();
        for field in &self.fields {
            fields += &format!("\n{}", field.to_string());
        }

        write!(f, "type {} {{", &self.name)?;
        write!(f, "{}", fields)?;
        writeln!(f, "\n}}")
    }
}

/// A definition used when a root GraphQL type differs from default types.
#[derive(Debug)]
pub struct SchemaDef {
    description: Option<String>,
    fields: Vec<Field>,
}

impl SchemaDef {
    /// Create a new instance of SchemaDef.
    pub fn new(field: Field) -> Self {
        Self {
            description: None,
            fields: vec![field],
        }
    }

    /// Set the schema def's description.
    pub fn description(&mut self, description: String) {
        self.description = Some(description)
    }

    /// Push a Field to schema def's fields vector.
    pub fn field(&mut self, field: Field) {
        self.fields.push(field)
    }
}

impl Display for SchemaDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            writeln!(f, "\"\"\"\n{}\n\"\"\"", description)?;
        }

        let mut fields = String::new();
        for field in &self.fields {
            fields += &format!("\n{}", field.to_string());
        }

        write!(f, "schema {{")?;
        write!(f, "{}", fields)?;
        writeln!(f, "\n}}")
    }
}

/// Define whether a Field is a vector.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum VecState {
    /// The type is not a vector: e.g. `String`.
    None,
    /// The type is a nullable vector: e.g. `[String]`.
    Nullable,
    /// The type is a non nullable vector: e.g. `[String]!`.
    NonNullable,
}
/// Field in a given SDL type.
#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    description: Option<String>,
    name: String,
    type_: String,
    default: Option<String>,
    is_nullable: bool,
    vec_state: VecState,
}

impl Field {
    /// Create a new instance of Field.
    pub fn new(name: String, type_: String) -> Self {
        Self {
            description: None,
            name,
            type_,
            default: None,
            is_nullable: true,
            vec_state: VecState::None,
        }
    }

    /// Set the field's description.
    pub fn description(&mut self, description: String) {
        self.description = Some(description);
    }

    /// Set the field's default.
    pub fn default(&mut self, default: String) {
        self.default = Some(default);
    }

    /// Set the field's is nullable.
    pub fn is_nullable(&mut self, nullable: bool) {
        self.is_nullable = nullable;
    }

    /// Set the field's vec state.
    pub fn vec_state(&mut self, vec_state: VecState) {
        self.vec_state = vec_state;
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            // Let's indent description on a field level for now, as all fields
            // are always on the same level and are indented by 2 spaces.
            writeln!(f, "  \"\"\"\n  {}\n  \"\"\"", description)?;
        }
        let null = if self.is_nullable { "!" } else { "" };
        let type_ = match self.vec_state {
            VecState::None => format!("{}{}", &self.type_, null),
            VecState::Nullable => format!("[{}{}]", &self.type_, null),
            VecState::NonNullable => format!("[{}{}]!", &self.type_, null),
        };
        let default = if let Some(default) = &self.default {
            match self.vec_state {
                VecState::None => format!(" = {}{}", default, null),
                VecState::Nullable => format!(" = [{}{}]", default, null),
                VecState::NonNullable => format!(" = [{}{}]!", default, null),
            }
        } else {
            String::new()
        };
        // TODO(@lrlna): double check with folks if it's a valid SDL if the last
        // field in a type has a comma. If not, we can move the 'comma logic' to
        // TypeDef/SchemaDef Display implementations.
        write!(f, "  {}: {}{},", &self.name, type_, default)
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
        let mut field = Field::new("cat".to_string(), "String".to_string());
        field.vec_state(VecState::NonNullable);
        field.description("Very good cats".to_string());
        let mut type_ = TypeDef::new("Query".to_string(), field.clone());
        let mut schema_def = SchemaDef::new(field);
        type_.description("Example Query type".to_string());
        schema_def.description("Simple schema".to_string());
        schema.schema(schema_def);
        schema.type_(type_);

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
                  cat: [String!]!,
                }
                """
                Example Query type
                """
                type Query {
                  """
                  Very good cats
                  """
                  cat: [String!]!,
                }
            "# }
        );
    }

    #[test]
    fn smoke_test_2() {
        let mut schema = Schema::new();

        let mut type_field = Field::new("cat".to_string(), "SpaceCatEnum".to_string());
        type_field.vec_state(VecState::NonNullable);
        let mut type_ = TypeDef::new("Query".to_string(), type_field);
        type_.description("Example Query type".to_string());

        let mut schema_field = Field::new("treat".to_string(), "String".to_string());
        schema_field.description("Good cats get treats".to_string());
        let mut schema_def = SchemaDef::new(schema_field);
        schema_def.description("Example schema Def".to_string());
        schema.schema(schema_def);
        schema.type_(type_);
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
                  treat: String!,
                }
                """
                Example Query type
                """
                type Query {
                  cat: [SpaceCatEnum!]!,
                }
        "#}
        )
    }

    #[test]
    fn it_encodes_fields() {
        let mut field = Field::new("cat".to_string(), "String".to_string());
        field.is_nullable(false);
        field.description("Very good cats".to_string());
        field.vec_state(VecState::Nullable);

        assert_eq!(
            field.to_string(),
            r#"  """
  Very good cats
  """
  cat: [String],"#
        );

        let mut field_2 = Field::new("spaceCat".to_string(), "SpaceProgram".to_string());
        field_2.description("Very good space cats".to_string());
        field_2.default("VoskhodCats".to_string());
        field_2.vec_state(VecState::NonNullable);

        assert_eq!(
            field_2.to_string(),
            r#"  """
  Very good space cats
  """
  spaceCat: [SpaceProgram!]! = [VoskhodCats!]!,"#
        );

        let field_3 = Field::new("spaceCat".to_string(), "SpaceProgram".to_string());

        assert_eq!(field_3.to_string(), r#"  spaceCat: SpaceProgram!,"#);
    }
}
