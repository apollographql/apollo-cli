//! SDL Encoder provides methods to serialise a GraphQL Schema.
//! For mor information on GraphQL Schema Types, please refer to [official
//! documentation](https://graphql.org/learn/schema/).
//!
//! ## Example
//! ```rust
//! use sdl_encoder::{Schema, TypeDef};
//!
//! let mut schema = Schema::new();
//! let mut type_ = TypeDef::new("Query".to_string());
//! type_.description("Example Query type".to_string());
//! schema.type_(type_);
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
}

impl TypeDef {
    /// Create a new instance of TypeDef with a name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
        }
    }

    /// Set the TypeDef's description field.
    pub fn description(&mut self, description: String) {
        self.description = Some(description)
    }
}

impl Display for TypeDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            writeln!(f, "\"\"\"\n{}\n\"\"\"", description)?;
        }
        writeln!(f, "type {} {{}}", self.name)
    }
}

/// A definition used when a root GraphQL type differs from default types.
#[derive(Debug)]
pub struct SchemaDef {
    description: Option<String>,
}

impl SchemaDef {
    /// Create a new instance of SchemaDef.
    pub fn new() -> Self {
        Self { description: None }
    }

    /// Set the schema def's description.
    pub fn description(&mut self, description: String) {
        self.description = Some(description)
    }
}

impl Display for SchemaDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            writeln!(f, "\"\"\"\n{}\n\"\"\"", description)?;
        }
        writeln!(f, "schema {{}}")
    }
}

impl Default for SchemaDef {
    fn default() -> Self {
        SchemaDef::new()
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
        let mut type_ = TypeDef::new("Query".to_string());
        let mut schema_def = SchemaDef::new();
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
                schema {}
                """
                Example Query type
                """
                type Query {}
            "# }
        );
    }
}
