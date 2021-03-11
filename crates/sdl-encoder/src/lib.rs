//! SDL Encoder provides methods to serialise a GraphQL Schema.
//! For mor information on GraphQL Schema Types, please refer to [official
//! documentation](https://graphql.org/learn/schema/).
//!
//! ## Example
//! ```rust
//! use sdl_encoder::{Schema, ObjectDef, SchemaDef, Field, VecState};
//! use indoc::indoc;
//!
//! let mut schema = Schema::new();
//!
//! let mut object_field = Field::new("cat".to_string(), "SpaceCatEnum".to_string());
//! object_field.vec_state(VecState::NonNullable);
//! let mut type_ = ObjectDef::new("Query".to_string(), object_field);
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
//!       treat: String!
//!     }
//!     """
//!     Example Query type
//!     """
//!     type Query {
//!       cat: [SpaceCatEnum!]!
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

/// Object Type Definition used to define different objects in SDL.
#[derive(Debug)]
pub struct ObjectDef {
    name: String,
    description: Option<String>,
    fields: Vec<Field>,
}

impl ObjectDef {
    /// Create a new instance of ObjectDef with a name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            fields: Vec::new(),
        }
    }

    /// Set the ObjectDef's description field.
    pub fn description(&mut self, description: Option<String>) {
        self.description = description
    }

    /// Push a Field to type def's fields vector.
    pub fn field(&mut self, field: Field) {
        self.fields.push(field)
    }
}

impl Display for ObjectDef {
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
#[derive(Debug, Clone)]
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
    pub fn description(&mut self, description: Option<String>) {
        self.description = description
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

/// Input types used to describe more complex objects in SDL.
#[derive(Debug, Clone)]
pub struct InputDef {
    description: Option<String>,
    name: String,
    fields: Vec<Field>,
}

impl InputDef {
    /// Create a new instance of ObjectDef with a name.
    pub fn new(name: String, field: Field) -> Self {
        Self {
            name,
            description: None,
            fields: vec![field],
        }
    }

    /// Set the ObjectDef's description field.
    pub fn description(&mut self, description: Option<String>) {
        self.description = description
    }

    /// Push a Field to type def's fields vector.
    pub fn field(&mut self, field: Field) {
        self.fields.push(field)
    }
}

impl Display for InputDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            writeln!(f, "\"\"\"\n{}\n\"\"\"", description)?;
        }

        let mut fields = String::new();
        for field in &self.fields {
            fields += &format!("\n{}", field.to_string());
        }

        write!(f, "input {} {{", &self.name)?;
        write!(f, "{}", fields)?;
        writeln!(f, "\n}}")
    }
}

/// Define Field Type.
#[derive(Debug, PartialEq, Clone)]
pub enum FieldType {
    /// The list field type.
    List {
        /// List field type.
        ty: Box<FieldType>,
        /// Nullable list.
        is_nullable: bool,
    },
    /// The type field type.
    Type {
        /// Type type.
        ty: String,
        /// Default field type type.
        default: Option<Box<FieldType>>,
        /// Nullable type.
        is_nullable: bool,
    },
}

impl Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldType::List { ty, is_nullable } => {
                let null = if *is_nullable { "" } else { "!" };
                write!(f, "[{}]{}", ty, null)
            }
            FieldType::Type {
                ty,
                // TODO(@lrlna): figure out the best way to encode default
                // values in fields
                default: _,
                is_nullable,
            } => {
                let null = if *is_nullable { "" } else { "!" };
                write!(f, "{}{}", ty, null)
            }
        }
    }
}

/// Field in a given SDL type.
#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    description: Option<String>,
    //TODO(@lrlna): fields for input objects can also take arguments. This
    //struct should also account for that.
    name: String,
    type_: FieldType,
}

impl Field {
    /// Create a new instance of Field.
    pub fn new(name: String, type_: FieldType) -> Self {
        Self {
            description: None,
            name,
            type_,
        }
    }

    /// Set the field's description.
    pub fn description(&mut self, description: Option<String>) {
        self.description = description;
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            // Let's indent description on a field level for now, as all fields
            // are always on the same level and are indented by 2 spaces.
            writeln!(f, "  \"\"\"\n  {}\n  \"\"\"", description)?;
        }
        // TODO(@lrlna): double check with folks if it's a valid SDL if the last
        // field in a type has a comma. If not, we can move the 'comma logic' to
        // ObjectDef/SchemaDef Display implementations.
        write!(f, "  {}: {}", self.name, self.type_)
    }
}

/// Enum type in SDL.
#[derive(Debug, PartialEq, Clone)]
pub struct EnumDef {
    name: String,
    description: Option<String>,
    variants: Vec<String>,
}

impl EnumDef {
    /// Create a new Enum type for SDL.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            variants: Vec::new(),
        }
    }

    /// Set the enum's description.
    pub fn description(&mut self, description: Option<String>) {
        self.description = description;
    }

    /// Set the EnumDef's variants.
    pub fn variant(&mut self, variant: String) {
        self.variants.push(variant)
    }
}

impl Display for EnumDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            writeln!(f, "\"\"\"\n{}\n\"\"\"", description)?;
        }

        let mut variants = String::new();
        for variant in &self.variants {
            variants += &format!("\n  {}", variant);
        }

        write!(f, "enum {} {{", self.name)?;
        write!(f, "{}", variants)?;
        writeln!(f, "\n}}")
    }
}

/// Scalar type for SDL.
#[derive(Debug, PartialEq, Clone)]
pub struct ScalarDef {
    name: String,
    description: Option<String>,
}

impl ScalarDef {
    /// Create a new instance of Scalar type.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
        }
    }

    /// Set the scalar def's description.
    pub fn description(&mut self, description: Option<String>) {
        self.description = description;
    }
}

impl Display for ScalarDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            writeln!(f, "\"\"\"\n{}\n\"\"\"", description)?;
        }

        writeln!(f, "scalar {}", self.name)
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

    #[test]
    fn it_encodes_fields() {
        let field_type_1 = FieldType::Type {
            ty: "SpaceProgram".to_string(),
            is_nullable: true,
            default: None,
        };

        let field_type_2 = FieldType::List {
            ty: Box::new(field_type_1),
            is_nullable: true,
        };

        let mut field = Field::new("cat".to_string(), field_type_2);
        field.description(Some("Very good cats".to_string()));

        assert_eq!(
            field.to_string(),
            r#"  """
  Very good cats
  """
  cat: [SpaceProgram]"#
        );

        let field_type_4 = FieldType::Type {
            ty: "SpaceProgram".to_string(),
            is_nullable: false,
            default: None,
        };

        let field_type_5 = FieldType::List {
            ty: Box::new(field_type_4),
            is_nullable: false,
        };

        let mut field_2 = Field::new("spaceCat".to_string(), field_type_5);
        field_2.description(Some("Very good space cats".to_string()));

        assert_eq!(
            field_2.to_string(),
            r#"  """
  Very good space cats
  """
  spaceCat: [SpaceProgram!]!"#
        );

        let field_type_6 = FieldType::Type {
            ty: "SpaceProgram".to_string(),
            is_nullable: true,
            default: None,
        };

        let field_type_7 = FieldType::List {
            ty: Box::new(field_type_6),
            is_nullable: false,
        };

        let field_type_8 = FieldType::List {
            ty: Box::new(field_type_7),
            is_nullable: false,
        };

        let field_3 = Field::new("spaceCat".to_string(), field_type_8);

        assert_eq!(field_3.to_string(), r#"  spaceCat: [[SpaceProgram]!]!"#);
    }
}
