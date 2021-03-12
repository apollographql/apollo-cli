use crate::Field;
use std::fmt::{self, Display};
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
