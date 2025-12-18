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
fn test_simple_class_decorator() {
    let source = r#"@decorator
class Foo:
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, bases, body, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 1);
            assert!(bases.is_empty());
            assert_eq!(body.len(), 1); // pass statement
            
            match &decorators[0] {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "decorator");
                }
                _ => panic!("Expected identifier decorator"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_class_decorator_with_call() {
    let source = r#"@decorator()
class Foo:
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 1);
            
            match &decorators[0] {
                Expression::Call { function, arguments, .. } => {
                    match &**function {
                        Expression::Identifier { name, .. } => {
                            assert_eq!(name, "decorator");
                        }
                        _ => panic!("Expected identifier in call"),
                    }
                    assert!(arguments.is_empty());
                }
                _ => panic!("Expected call decorator"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_class_decorator_with_arguments() {
    let source = r#"@decorator(arg1, arg2)
class Foo:
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 1);
            
            match &decorators[0] {
                Expression::Call { arguments, .. } => {
                    assert_eq!(arguments.len(), 2);
                }
                _ => panic!("Expected call decorator"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_multiple_class_decorators() {
    let source = r#"@decorator1
@decorator2
@decorator3
class Foo:
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 3);
            
            // Decorators should be in order from top to bottom
            match &decorators[0] {
                Expression::Identifier { name, .. } => assert_eq!(name, "decorator1"),
                _ => panic!("Expected identifier"),
            }
            match &decorators[1] {
                Expression::Identifier { name, .. } => assert_eq!(name, "decorator2"),
                _ => panic!("Expected identifier"),
            }
            match &decorators[2] {
                Expression::Identifier { name, .. } => assert_eq!(name, "decorator3"),
                _ => panic!("Expected identifier"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_class_decorator_with_attribute_access() {
    let source = r#"@package.module.decorator
class Foo:
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 1);
            
            // Should be Attribute
            match &decorators[0] {
                Expression::Attribute { .. } => {
                    // Correct type
                }
                _ => panic!("Expected attribute access decorator"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_class_decorator_with_single_inheritance() {
    let source = r#"@decorator
class Foo(Base):
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, bases, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 1);
            assert_eq!(bases.len(), 1);
            
            match &decorators[0] {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "decorator");
                }
                _ => panic!("Expected identifier decorator"),
            }
            
            match &bases[0] {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "Base");
                }
                _ => panic!("Expected identifier base"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_class_decorator_with_multiple_inheritance() {
    let source = r#"@decorator
class Foo(Base1, Base2, Base3):
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, bases, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 1);
            assert_eq!(bases.len(), 3);
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_decorated_class_with_methods() {
    let source = r#"@decorator
class Foo:
    def method1(self):
        pass
    
    def method2(self, arg):
        return arg
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, body, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 1);
            assert_eq!(body.len(), 2); // Two methods
            
            // Verify methods are present
            match &body[0] {
                Statement::FunctionDef { name, .. } => {
                    assert_eq!(name, "method1");
                }
                _ => panic!("Expected FunctionDef"),
            }
            
            match &body[1] {
                Statement::FunctionDef { name, .. } => {
                    assert_eq!(name, "method2");
                }
                _ => panic!("Expected FunctionDef"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_decorated_and_undecorated_classes() {
    let source = r#"@decorator
class Foo:
    pass

class Bar:
    pass

@another_decorator
class Baz:
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 3);
    
    // First class: decorated
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 1);
        }
        _ => panic!("Expected ClassDef"),
    }
    
    // Second class: not decorated
    match &module.statements[1] {
        Statement::ClassDef { name, decorators, .. } => {
            assert_eq!(name, "Bar");
            assert_eq!(decorators.len(), 0);
        }
        _ => panic!("Expected ClassDef"),
    }
    
    // Third class: decorated
    match &module.statements[2] {
        Statement::ClassDef { name, decorators, .. } => {
            assert_eq!(name, "Baz");
            assert_eq!(decorators.len(), 1);
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_class_decorator_with_nested_call() {
    let source = r#"@decorator(other_func(arg))
class Foo:
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 1);
            
            match &decorators[0] {
                Expression::Call { arguments, .. } => {
                    assert_eq!(arguments.len(), 1);
                    // The argument should be another call
                    match &arguments[0] {
                        Expression::Call { .. } => {
                            // Correct
                        }
                        _ => panic!("Expected nested call"),
                    }
                }
                _ => panic!("Expected call decorator"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_multiple_decorated_classes_with_blank_lines() {
    let source = r#"@decorator1
class Foo:
    pass


@decorator2
class Bar:
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 2);
    
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 1);
        }
        _ => panic!("Expected ClassDef"),
    }
    
    match &module.statements[1] {
        Statement::ClassDef { name, decorators, .. } => {
            assert_eq!(name, "Bar");
            assert_eq!(decorators.len(), 1);
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_class_decorator_mixed_with_function_decorator() {
    let source = r#"@class_decorator
class Foo:
    @method_decorator
    def method(self):
        pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, body, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 1);
            
            // Check class decorator
            match &decorators[0] {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "class_decorator");
                }
                _ => panic!("Expected identifier"),
            }
            
            // Check method decorator
            match &body[0] {
                Statement::FunctionDef { name, decorators, .. } => {
                    assert_eq!(name, "method");
                    assert_eq!(decorators.len(), 1);
                    
                    match &decorators[0] {
                        Expression::Identifier { name, .. } => {
                            assert_eq!(name, "method_decorator");
                        }
                        _ => panic!("Expected identifier"),
                    }
                }
                _ => panic!("Expected FunctionDef"),
            }
        }
        _ => panic!("Expected ClassDef"),
    }
}

#[test]
fn test_complex_class_decorator_expression() {
    let source = r#"@decorator.method().another()
class Foo:
    pass
"#;
    let module = parse(source).unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::ClassDef { name, decorators, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(decorators.len(), 1);
            // Complex expression - just verify it parsed
        }
        _ => panic!("Expected ClassDef"),
    }
}
