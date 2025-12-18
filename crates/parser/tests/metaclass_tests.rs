use mamba_parser::ast::*;
use mamba_parser::lexer::Lexer;
use mamba_parser::parser::Parser;

/// Helper function to parse a string into an AST
fn parse(input: &str) -> Result<Module, mamba_error::MambaError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse().map_err(|errors| errors.into_iter().next().unwrap())
}

#[test]
fn test_simple_metaclass_no_bases() {
    let source = r#"class Foo(metaclass=Meta):
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, bases, metaclass, .. } => {
            assert_eq!(name, "Foo");
            assert!(bases.is_empty());
            assert!(metaclass.is_some());
            
            match metaclass.as_ref().unwrap() {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "Meta");
                }
                _ => panic!("Expected identifier metaclass"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_metaclass_with_single_base() {
    let source = r#"class Foo(Base, metaclass=Meta):
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, bases, metaclass, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(bases.len(), 1);
            assert!(metaclass.is_some());
            
            match &bases[0] {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "Base");
                }
                _ => panic!("Expected identifier base"),
            }
            
            match metaclass.as_ref().unwrap() {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "Meta");
                }
                _ => panic!("Expected identifier metaclass"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_metaclass_with_multiple_bases() {
    let source = r#"class Foo(Base1, Base2, Base3, metaclass=Meta):
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, bases, metaclass, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(bases.len(), 3);
            assert!(metaclass.is_some());
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_metaclass_with_attribute_access() {
    let source = r#"class Foo(metaclass=pkg.module.Meta):
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, metaclass, .. } => {
            assert_eq!(name, "Foo");
            assert!(metaclass.is_some());
            
            // Should be attribute access
            match metaclass.as_ref().unwrap() {
                Expression::Attribute { .. } => {
                    // Correct
                }
                _ => panic!("Expected attribute access metaclass"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_metaclass_with_call() {
    let source = r#"class Foo(metaclass=type()):
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, metaclass, .. } => {
            assert_eq!(name, "Foo");
            assert!(metaclass.is_some());
            
            // Should be a call expression
            match metaclass.as_ref().unwrap() {
                Expression::Call { .. } => {
                    // Correct
                }
                _ => panic!("Expected call expression metaclass"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_metaclass_with_trailing_comma() {
    let source = r#"class Foo(Base, metaclass=Meta,):
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, bases, metaclass, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(bases.len(), 1);
            assert!(metaclass.is_some());
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_class_without_metaclass() {
    let source = r#"class Foo(Base):
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, bases, metaclass, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(bases.len(), 1);
            assert!(metaclass.is_none());
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_metaclass_with_decorators() {
    let source = r#"@decorator
class Foo(metaclass=Meta):
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, metaclass, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 1);
            assert!(metaclass.is_some());
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_metaclass_with_bases_and_decorators() {
    let source = r#"@decorator1
@decorator2
class Foo(Base1, Base2, metaclass=Meta):
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, bases, metaclass, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 2);
            assert_eq!(bases.len(), 2);
            assert!(metaclass.is_some());
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_metaclass_with_methods() {
    let source = r#"class Foo(metaclass=Meta):
    def method(self):
        pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, metaclass, body, .. } => {
            assert_eq!(name, "Foo");
            assert!(metaclass.is_some());
            assert_eq!(body.len(), 1);
            
            match &body[0] {
                Statement::FunctionDef { name, .. } => {
                    assert_eq!(name, "method");
                }
                _ => panic!("Expected FunctionDef"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_multiple_classes_with_and_without_metaclass() {
    let source = r#"class Foo(metaclass=Meta):
    pass

class Bar:
    pass

class Baz(Base, metaclass=Meta):
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 3);
    
    // First class: has metaclass
    match &module.statements[0] {
        Statement::ClassDef { name, metaclass, .. } => {
            assert_eq!(name, "Foo");
            assert!(metaclass.is_some());
        }
        _ => panic!("Expected ClassDef"),
    }
    
    // Second class: no metaclass
    match &module.statements[1] {
        Statement::ClassDef { name, metaclass, .. } => {
            assert_eq!(name, "Bar");
            assert!(metaclass.is_none());
        }
        _ => panic!("Expected ClassDef"),
    }
    
    // Third class: has metaclass and base
    match &module.statements[2] {
        Statement::ClassDef { name, bases, metaclass, .. } => {
            assert_eq!(name, "Baz");
            assert_eq!(bases.len(), 1);
            assert!(metaclass.is_some());
        }
        _ => panic!("Expected ClassDef"),
    }
}

// Negative tests

#[test]
fn test_invalid_keyword_argument() {
    let source = r#"class Foo(invalid=value):
    pass
"#;
    let result = parse(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid keyword argument"));
    assert!(err.to_string().contains("invalid"));
}

#[test]
fn test_duplicate_metaclass() {
    let source = r#"class Foo(metaclass=Meta1, metaclass=Meta2):
    pass
"#;
    let result = parse(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Duplicate metaclass"));
}

#[test]
fn test_base_after_metaclass() {
    let source = r#"class Foo(metaclass=Meta, Base):
    pass
"#;
    let result = parse(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    // Either error message is acceptable for this case
    assert!(err.to_string().contains("Base classes must come before metaclass"));
}

#[test]
fn test_metaclass_only_no_parens_is_regular_class() {
    let source = r#"class Foo:
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, bases, metaclass, .. } => {
            assert_eq!(name, "Foo");
            assert!(bases.is_empty());
            assert!(metaclass.is_none());
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_metaclass_with_type_builtin() {
    let source = r#"class Foo(metaclass=type):
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, metaclass, .. } => {
            assert_eq!(name, "Foo");
            assert!(metaclass.is_some());
            
            match metaclass.as_ref().unwrap() {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "type");
                }
                _ => panic!("Expected identifier"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}
