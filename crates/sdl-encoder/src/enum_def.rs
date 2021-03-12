use std::fmt::{self, Display};

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
