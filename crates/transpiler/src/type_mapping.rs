use mamba_parser::ast::Expression;
use mamba_parser::types::Type;
use thiserror::Error;

/// Controls how Python `int` is emitted in Rust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntWidth {
    I32,
    I64,
}

/// Rust-side type representation used by transpilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustType {
    I32,
    I64,
    F64,
    String,
    Bool,
    Unit,
    Option(Box<RustType>),
    Unknown,
}

impl RustType {
    /// Render this type as Rust source text.
    pub fn render(&self) -> String {
        match self {
            RustType::I32 => "i32".to_string(),
            RustType::I64 => "i64".to_string(),
            RustType::F64 => "f64".to_string(),
            RustType::String => "String".to_string(),
            RustType::Bool => "bool".to_string(),
            RustType::Unit => "()".to_string(),
            RustType::Option(inner) => format!("Option<{}>", inner.render()),
            RustType::Unknown => "_".to_string(),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TypeMappingError {
    #[error("unsupported type annotation expression")]
    UnsupportedAnnotation,
    #[error("unknown type annotation '{0}'")]
    UnknownAnnotation(String),
    #[error("expected identifier as generic base type")]
    InvalidGenericBase,
    #[error("unsupported generic type '{0}'")]
    UnsupportedGeneric(String),
}

/// Maps Mamba semantic/annotation types into Rust types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeMapper {
    int_width: IntWidth,
}

impl Default for TypeMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeMapper {
    /// Create a mapper using i64 for Python integers.
    pub fn new() -> Self {
        Self {
            int_width: IntWidth::I64,
        }
    }

    /// Create a mapper with an explicit integer width strategy.
    pub fn with_int_width(int_width: IntWidth) -> Self {
        Self { int_width }
    }

    /// Map semantic-analysis types into Rust types.
    pub fn map_semantic_type(&self, ty: &Type) -> RustType {
        match ty {
            Type::Int => self.int_type(),
            Type::Float => RustType::F64,
            Type::String => RustType::String,
            Type::Bool => RustType::Bool,
            // `None` maps to `Option<()>` until richer contextual typing is available.
            Type::None => RustType::Option(Box::new(RustType::Unit)),
            Type::Unknown => RustType::Unknown,
        }
    }

    /// Map a parsed type annotation expression into a Rust type.
    ///
    /// Supported forms:
    /// - `int`, `float`, `str`, `bool`, `None`
    /// - `Option[T]`
    /// - Parenthesized forms like `(int)`
    pub fn map_annotation_expr(&self, annotation: &Expression) -> Result<RustType, TypeMappingError> {
        match annotation {
            Expression::Identifier { name, .. } => self.map_annotation_name(name),
            Expression::Parenthesized { expr, .. } => self.map_annotation_expr(expr),
            Expression::Subscript { object, index, .. } => {
                let base_name = match object.as_ref() {
                    Expression::Identifier { name, .. } => name.as_str(),
                    _ => return Err(TypeMappingError::InvalidGenericBase),
                };

                if base_name.eq_ignore_ascii_case("option") {
                    let inner = self.map_annotation_expr(index)?;
                    Ok(RustType::Option(Box::new(inner)))
                } else {
                    Err(TypeMappingError::UnsupportedGeneric(base_name.to_string()))
                }
            }
            _ => Err(TypeMappingError::UnsupportedAnnotation),
        }
    }

    /// Render a mapped annotation as Rust type text.
    pub fn render_annotation(&self, annotation: &Expression) -> Result<String, TypeMappingError> {
        Ok(self.map_annotation_expr(annotation)?.render())
    }

    fn int_type(&self) -> RustType {
        match self.int_width {
            IntWidth::I32 => RustType::I32,
            IntWidth::I64 => RustType::I64,
        }
    }

    fn map_annotation_name(&self, name: &str) -> Result<RustType, TypeMappingError> {
        if name.eq_ignore_ascii_case("int") {
            return Ok(self.int_type());
        }
        if name.eq_ignore_ascii_case("float") {
            return Ok(RustType::F64);
        }
        if name.eq_ignore_ascii_case("str") {
            return Ok(RustType::String);
        }
        if name.eq_ignore_ascii_case("bool") {
            return Ok(RustType::Bool);
        }
        if name.eq_ignore_ascii_case("none") {
            return Ok(RustType::Option(Box::new(RustType::Unit)));
        }

        Err(TypeMappingError::UnknownAnnotation(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::{IntWidth, RustType, TypeMapper, TypeMappingError};
    use mamba_parser::ast::Expression;
    use mamba_parser::token::SourcePosition;
    use mamba_parser::types::Type;

    fn pos() -> SourcePosition {
        SourcePosition::new(1, 1, 0)
    }

    fn ident(name: &str) -> Expression {
        Expression::Identifier {
            name: name.to_string(),
            position: pos(),
        }
    }

    #[test]
    fn maps_semantic_types_default_width() {
        let mapper = TypeMapper::new();

        assert_eq!(mapper.map_semantic_type(&Type::Int), RustType::I64);
        assert_eq!(mapper.map_semantic_type(&Type::Float), RustType::F64);
        assert_eq!(mapper.map_semantic_type(&Type::String), RustType::String);
        assert_eq!(mapper.map_semantic_type(&Type::Bool), RustType::Bool);
        assert_eq!(
            mapper.map_semantic_type(&Type::None),
            RustType::Option(Box::new(RustType::Unit))
        );
        assert_eq!(mapper.map_semantic_type(&Type::Unknown), RustType::Unknown);
    }

    #[test]
    fn maps_semantic_int_to_i32_when_configured() {
        let mapper = TypeMapper::with_int_width(IntWidth::I32);
        assert_eq!(mapper.map_semantic_type(&Type::Int), RustType::I32);
    }

    #[test]
    fn maps_simple_annotation_identifiers() {
        let mapper = TypeMapper::new();

        assert_eq!(mapper.map_annotation_expr(&ident("int")), Ok(RustType::I64));
        assert_eq!(mapper.map_annotation_expr(&ident("float")), Ok(RustType::F64));
        assert_eq!(mapper.map_annotation_expr(&ident("str")), Ok(RustType::String));
        assert_eq!(mapper.map_annotation_expr(&ident("bool")), Ok(RustType::Bool));
        assert_eq!(
            mapper.map_annotation_expr(&ident("None")),
            Ok(RustType::Option(Box::new(RustType::Unit)))
        );
    }

    #[test]
    fn maps_parenthesized_annotation() {
        let mapper = TypeMapper::new();
        let annotation = Expression::Parenthesized {
            expr: Box::new(ident("int")),
            position: pos(),
        };

        assert_eq!(mapper.map_annotation_expr(&annotation), Ok(RustType::I64));
    }

    #[test]
    fn maps_option_generic_annotation() {
        let mapper = TypeMapper::new();
        let annotation = Expression::Subscript {
            object: Box::new(ident("Option")),
            index: Box::new(ident("str")),
            position: pos(),
        };

        assert_eq!(
            mapper.map_annotation_expr(&annotation),
            Ok(RustType::Option(Box::new(RustType::String)))
        );
    }

    #[test]
    fn renders_annotation_to_rust_string() {
        let mapper = TypeMapper::new();
        let annotation = Expression::Subscript {
            object: Box::new(ident("Option")),
            index: Box::new(ident("int")),
            position: pos(),
        };

        assert_eq!(mapper.render_annotation(&annotation), Ok("Option<i64>".to_string()));
    }

    #[test]
    fn rejects_unknown_identifier_annotation() {
        let mapper = TypeMapper::new();
        let result = mapper.map_annotation_expr(&ident("Decimal"));

        assert_eq!(
            result,
            Err(TypeMappingError::UnknownAnnotation("Decimal".to_string()))
        );
    }

    #[test]
    fn rejects_unsupported_generic_base() {
        let mapper = TypeMapper::new();
        let annotation = Expression::Subscript {
            object: Box::new(ident("List")),
            index: Box::new(ident("int")),
            position: pos(),
        };

        assert_eq!(
            mapper.map_annotation_expr(&annotation),
            Err(TypeMappingError::UnsupportedGeneric("List".to_string()))
        );
    }

    #[test]
    fn rejects_non_identifier_generic_base() {
        let mapper = TypeMapper::new();
        let annotation = Expression::Subscript {
            object: Box::new(Expression::Parenthesized {
                expr: Box::new(ident("Option")),
                position: pos(),
            }),
            index: Box::new(ident("int")),
            position: pos(),
        };

        assert_eq!(
            mapper.map_annotation_expr(&annotation),
            Err(TypeMappingError::InvalidGenericBase)
        );
    }

    #[test]
    fn rejects_unsupported_annotation_expression() {
        let mapper = TypeMapper::new();
        let annotation = Expression::Tuple {
            elements: vec![ident("int")],
            position: pos(),
        };

        assert_eq!(
            mapper.map_annotation_expr(&annotation),
            Err(TypeMappingError::UnsupportedAnnotation)
        );
    }

    #[test]
    fn renders_nested_option() {
        let mapper = TypeMapper::new();
        let annotation = Expression::Subscript {
            object: Box::new(ident("Option")),
            index: Box::new(Expression::Subscript {
                object: Box::new(ident("Option")),
                index: Box::new(ident("bool")),
                position: pos(),
            }),
            position: pos(),
        };

        assert_eq!(
            mapper.render_annotation(&annotation),
            Ok("Option<Option<bool>>".to_string())
        );
    }
}
