use crate::Field;
use std::fmt::{self, Display};

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
