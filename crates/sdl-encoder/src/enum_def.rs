use std::fmt::{self, Display};

/// Enum type in SDL.
#[derive(Debug, PartialEq, Clone)]
pub struct EnumDef {
    name: String,
    description: Option<String>,
    values: Vec<String>,
}

impl EnumDef {
    /// Create a new Enum type for SDL.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            values: Vec::new(),
        }
    }

    /// Set the enum's description.
    pub fn description(&mut self, description: Option<String>) {
        self.description = description;
    }

    /// Set the EnumDef's values.
    pub fn value(&mut self, value: String) {
        self.values.push(value)
    }
}

impl Display for EnumDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            writeln!(f, "\"\"\"\n{}\n\"\"\"", description)?;
        }

        let mut values = String::new();
        for value in &self.values {
            values += &format!("\n  {}", value);
        }

        write!(f, "enum {} {{", self.name)?;
        write!(f, "{}", values)?;
        writeln!(f, "\n}}")
    }
}
