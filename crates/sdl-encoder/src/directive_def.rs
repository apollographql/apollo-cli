use std::fmt::{self, Display};

/// The __Directive type represents a Directive that a service supports.
#[derive(Debug)]
pub struct Directive {
    name: String,
    description: Option<String>,
    locations: Vec<String>,
}

impl Directive {
    /// Create a new instance of Directive type.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            locations: Vec::new(),
        }
    }

    /// Set the Directive's description.
    pub fn description(&mut self, description: Option<String>) {
        self.description = description;
    }

    /// Set the Directive's location.
    pub fn location(&mut self, location: String) {
        self.locations.push(location);
    }
}

impl Display for Directive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            writeln!(f, "\"\"\"\n{}\n\"\"\"", description)?;
        }

        let mut locations = String::new();
        for (i, location) in self.locations.iter().enumerate() {
            if i == 0 {
                locations += location;
                continue;
            }
            locations += &format!(" | {}", location);
        }

        writeln!(f, "directive @{} on {}", &self.name, locations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_directives_for_a_single_location() {
        let mut directive = Directive::new("infer".to_string());
        directive.description(Some("Infer field types from field values.".to_string()));
        directive.location("OBJECT".to_string());

        assert_eq!(
            directive.to_string(),
            r#""""
Infer field types from field values.
"""
directive @infer on OBJECT
"#
        );
    }

    #[test]
    fn it_encodes_directives_for_multiple_location() {
        let mut directive = Directive::new("infer".to_string());
        directive.description(Some("Infer field types from field values.".to_string()));
        directive.location("OBJECT".to_string());
        directive.location("FIELD_DEFINITION".to_string());
        directive.location("INPUT_FIELD_DEFINITION".to_string());

        assert_eq!(
            directive.to_string(),
            r#""""
Infer field types from field values.
"""
directive @infer on OBJECT | FIELD_DEFINITION | INPUT_FIELD_DEFINITION
"#
        );
    }
}
