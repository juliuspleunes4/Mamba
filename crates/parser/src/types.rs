//! Type system for Mamba semantic analysis
//!
//! This module defines the type representation and inference system.

use std::fmt;

/// Represents a type in the Mamba type system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Integer type (i64)
    Int,
    /// Floating point type (f64)
    Float,
    /// String type
    String,
    /// Boolean type
    Bool,
    /// None type (Python's None)
    None,
    /// Unknown type (not yet inferred or invalid)
    Unknown,
}

impl Type {
    /// Check if this type is compatible with another type
    /// In Python, we allow dynamic typing, so this is mostly for error detection
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        match (self, other) {
            // Same types are always compatible
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::String, Type::String) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::None, Type::None) => true,
            (Type::Unknown, _) | (_, Type::Unknown) => true, // Unknown is compatible with anything
            
            // Numeric types can be compatible in some operations
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => true,
            
            // Bool can be treated as Int in some contexts (Python behavior)
            (Type::Bool, Type::Int) | (Type::Int, Type::Bool) => true,
            
            // Everything else is incompatible
            _ => false,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::String => write!(f, "str"),
            Type::Bool => write!(f, "bool"),
            Type::None => write!(f, "None"),
            Type::Unknown => write!(f, "?"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_equality() {
        assert_eq!(Type::Int, Type::Int);
        assert_eq!(Type::Float, Type::Float);
        assert_eq!(Type::String, Type::String);
        assert_ne!(Type::Int, Type::Float);
    }

    #[test]
    fn test_type_display() {
        assert_eq!(Type::Int.to_string(), "int");
        assert_eq!(Type::Float.to_string(), "float");
        assert_eq!(Type::String.to_string(), "str");
        assert_eq!(Type::Bool.to_string(), "bool");
        assert_eq!(Type::None.to_string(), "None");
        assert_eq!(Type::Unknown.to_string(), "?");
    }

    #[test]
    fn test_same_type_compatibility() {
        assert!(Type::Int.is_compatible_with(&Type::Int));
        assert!(Type::Float.is_compatible_with(&Type::Float));
        assert!(Type::String.is_compatible_with(&Type::String));
        assert!(Type::Bool.is_compatible_with(&Type::Bool));
        assert!(Type::None.is_compatible_with(&Type::None));
    }

    #[test]
    fn test_numeric_compatibility() {
        assert!(Type::Int.is_compatible_with(&Type::Float));
        assert!(Type::Float.is_compatible_with(&Type::Int));
        assert!(Type::Bool.is_compatible_with(&Type::Int));
        assert!(Type::Int.is_compatible_with(&Type::Bool));
    }

    #[test]
    fn test_unknown_compatibility() {
        assert!(Type::Unknown.is_compatible_with(&Type::Int));
        assert!(Type::Int.is_compatible_with(&Type::Unknown));
        assert!(Type::Unknown.is_compatible_with(&Type::Unknown));
    }

    #[test]
    fn test_incompatible_types() {
        assert!(!Type::String.is_compatible_with(&Type::Int));
        assert!(!Type::Int.is_compatible_with(&Type::String));
        assert!(!Type::String.is_compatible_with(&Type::Bool));
        assert!(!Type::None.is_compatible_with(&Type::Int));
    }
}
