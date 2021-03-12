use crate::Field;
use std::fmt::{self, Display};

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
