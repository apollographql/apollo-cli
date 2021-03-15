use std::fmt::{self, Display};

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

// NOTE(@lrlna): This impl is specifically to be use for introspection encoding
// to make creating FieldTypes as short as possible. When used outside of
// Introspection, it's best to use `FieldType::List` and `FieldType::Type` invocations.
impl FieldType {
    /// Create new List Field Type.
    pub fn new_list(ty: FieldType, is_nullable: bool) -> Self {
        Self::List {
            ty: Box::new(ty),
            is_nullable,
        }
    }

    /// Create new Type Field Type.
    pub fn new_type(ty: String, is_nullable: bool, default: Option<FieldType>) -> Self {
        Self::Type {
            ty,
            is_nullable,
            default: default.map(Box::new),
        }
    }

    /// Set is_nullable in all variants.
    pub fn set_is_nullable(&mut self, nullable: bool) {
        match self {
            FieldType::List { is_nullable, .. } => *is_nullable = nullable,
            FieldType::Type { is_nullable, .. } => *is_nullable = nullable,
        }
    }

    /// Get is_nullable in all variants.
    pub fn is_nullable(&self) -> bool {
        match self {
            FieldType::List { is_nullable, .. } => *is_nullable,
            FieldType::Type { is_nullable, .. } => *is_nullable,
        }
    }
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
