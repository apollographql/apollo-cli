use std::fmt::{self, Display};

/// Union in a given SDL.
///
/// Unions are an abstract type where no common fields are declared.
#[derive(Debug, PartialEq, Clone)]
pub struct Union {
    name: String,
    description: Option<String>,
    members: Vec<String>,
}

impl Union {
    /// Create a new instance of a Union.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            members: Vec::new(),
        }
    }

    /// Set the Unions description.
    pub fn description(&mut self, description: Option<String>) {
        self.description = description;
    }

    /// Set a Union member.
    pub fn member(&mut self, member: String) {
        self.members.push(member);
    }
}

impl Display for Union {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            // Let's indent description on a field level for now, as all fields
            // are always on the same level and are indented by 2 spaces.
            writeln!(f, "\"\"\"\n{}\n\"\"\"", description)?;
        }

        let mut members = String::new();
        for (i, member) in self.members.iter().enumerate() {
            if i == 0 {
                members += member;
                continue;
            }
            members += &format!(" | {}", member);
        }
        writeln!(f, "union {} = {}", self.name, members)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_union() {
        let mut union_ = Union::new("Cat".to_string());
        union_.description(Some(
            "A union of all cats represented within a household.".to_string(),
        ));
        union_.member("NORI".to_string());
        union_.member("CHASHU".to_string());

        assert_eq!(
            union_.to_string(),
            r#""""
A union of all cats represented within a household.
"""
union Cat = NORI | CHASHU
"#
        );
    }
}
