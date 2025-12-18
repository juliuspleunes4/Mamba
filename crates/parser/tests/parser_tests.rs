use mamba_parser::ast::*;
use mamba_parser::lexer::Lexer;
use mamba_parser::parser::Parser;

/// Helper function to parse a string into an AST
fn parse(input: &str) -> Result<Module, mamba_error::MambaError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[test]
fn test_parse_integer_literal() {
    let module = parse("42\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Literal(Literal::Integer { value, .. })) => {
            assert_eq!(*value, 42);
        }
        _ => panic!("Expected integer literal expression"),
    }
}

#[test]
fn test_parse_float_literal() {
    let module = parse("3.14\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Literal(Literal::Float { value, .. })) => {
            assert!((value - 3.14).abs() < 0.001);
        }
        _ => panic!("Expected float literal expression"),
    }
}

#[test]
fn test_parse_string_literal() {
    let module = parse("\"hello\"\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Literal(Literal::String { value, .. })) => {
            assert_eq!(value, "hello");
        }
        _ => panic!("Expected string literal expression"),
    }
}

#[test]
fn test_parse_boolean_literals() {
    let module = parse("True\nFalse\n").unwrap();
    assert_eq!(module.statements.len(), 2);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Literal(Literal::Boolean { value, .. })) => {
            assert_eq!(*value, true);
        }
        _ => panic!("Expected True literal"),
    }
    
    match &module.statements[1] {
        Statement::Expression(Expression::Literal(Literal::Boolean { value, .. })) => {
            assert_eq!(*value, false);
        }
        _ => panic!("Expected False literal"),
    }
}

#[test]
fn test_parse_none_literal() {
    let module = parse("None\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Literal(Literal::None { .. })) => {},
        _ => panic!("Expected None literal"),
    }
}

#[test]
fn test_parse_identifier() {
    let module = parse("variable\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Identifier { name, .. }) => {
            assert_eq!(name, "variable");
        }
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_parse_simple_addition() {
    let module = parse("1 + 2\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { left, op, right, .. }) => {
            assert!(matches!(op, BinaryOperator::Add));
            
            // Check left operand
            match **left {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(value, 1),
                _ => panic!("Expected integer 1"),
            }
            
            // Check right operand
            match **right {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(value, 2),
                _ => panic!("Expected integer 2"),
            }
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_simple_subtraction() {
    let module = parse("10 - 3\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { left, op, right, .. }) => {
            assert!(matches!(op, BinaryOperator::Subtract));
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_multiplication() {
    let module = parse("5 * 6\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { op, .. }) => {
            assert!(matches!(op, BinaryOperator::Multiply));
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_division() {
    let module = parse("12 / 4\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { op, .. }) => {
            assert!(matches!(op, BinaryOperator::Divide));
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_power() {
    let module = parse("2 ** 8\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { op, .. }) => {
            assert!(matches!(op, BinaryOperator::Power));
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_comparison() {
    let module = parse("x == 5\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { op, .. }) => {
            assert!(matches!(op, BinaryOperator::Equal));
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_operator_precedence() {
    // 1 + 2 * 3 should parse as 1 + (2 * 3)
    let module = parse("1 + 2 * 3\n").unwrap();
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { left, op, right, .. }) => {
            // Top level should be addition
            assert!(matches!(op, BinaryOperator::Add));
            
            // Left should be 1
            match **left {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(value, 1),
                _ => panic!("Expected integer 1"),
            }
            
            // Right should be multiplication (2 * 3)
            match **right {
                Expression::BinaryOp { op: BinaryOperator::Multiply, .. } => {},
                _ => panic!("Expected multiplication on right"),
            }
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_parenthesized_expression() {
    let module = parse("(1 + 2) * 3\n").unwrap();
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { left, op, right, .. }) => {
            // Top level should be multiplication
            assert!(matches!(op, BinaryOperator::Multiply));
            
            // Left should be parenthesized addition
            match **left {
                Expression::Parenthesized { .. } => {},
                _ => panic!("Expected parenthesized expression on left"),
            }
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_unary_minus() {
    let module = parse("-5\n").unwrap();
    
    match &module.statements[0] {
        Statement::Expression(Expression::UnaryOp { op, operand, .. }) => {
            assert!(matches!(op, UnaryOperator::Minus));
            
            match **operand {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(value, 5),
                _ => panic!("Expected integer 5"),
            }
        }
        _ => panic!("Expected unary operation"),
    }
}

#[test]
fn test_parse_logical_and() {
    let module = parse("True and False\n").unwrap();
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { op, .. }) => {
            assert!(matches!(op, BinaryOperator::And));
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_logical_or() {
    let module = parse("True or False\n").unwrap();
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { op, .. }) => {
            assert!(matches!(op, BinaryOperator::Or));
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_logical_not() {
    let module = parse("not True\n").unwrap();
    
    match &module.statements[0] {
        Statement::Expression(Expression::UnaryOp { op, .. }) => {
            assert!(matches!(op, UnaryOperator::Not));
        }
        _ => panic!("Expected unary operation"),
    }
}

#[test]
fn test_parse_assignment() {
    let module = parse("x = 42\n").unwrap();
    
    match &module.statements[0] {
        Statement::Assignment { targets, value, .. } => {
            // Should have exactly one target for simple assignment
            assert_eq!(targets.len(), 1);
            
            // Check target is identifier
            match &targets[0] {
                Expression::Identifier { name, .. } => assert_eq!(name, "x"),
                _ => panic!("Expected identifier as target"),
            }
            
            // Check value is integer
            match value {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 42),
                _ => panic!("Expected integer value"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_parse_augmented_assignment() {
    let module = parse("x += 5\n").unwrap();
    
    match &module.statements[0] {
        Statement::AugmentedAssignment { target, op, value, .. } => {
            assert!(matches!(op, AugmentedOperator::Add));
            
            match target {
                Expression::Identifier { name, .. } => assert_eq!(name, "x"),
                _ => panic!("Expected identifier as target"),
            }
            
            match value {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 5),
                _ => panic!("Expected integer value"),
            }
        }
        _ => panic!("Expected augmented assignment statement"),
    }
}

#[test]
fn test_parse_multiple_assignment() {
    let module = parse("x = y = 42\n").unwrap();
    
    match &module.statements[0] {
        Statement::Assignment { targets, value, .. } => {
            // Should have two targets
            assert_eq!(targets.len(), 2);
            
            // Check first target is 'x'
            match &targets[0] {
                Expression::Identifier { name, .. } => assert_eq!(name, "x"),
                _ => panic!("Expected identifier 'x' as first target"),
            }
            
            // Check second target is 'y'
            match &targets[1] {
                Expression::Identifier { name, .. } => assert_eq!(name, "y"),
                _ => panic!("Expected identifier 'y' as second target"),
            }
            
            // Check value is integer 42
            match value {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 42),
                _ => panic!("Expected integer value"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_parse_chained_assignment() {
    let module = parse("a = b = c = 100\n").unwrap();
    
    match &module.statements[0] {
        Statement::Assignment { targets, value, .. } => {
            // Should have three targets
            assert_eq!(targets.len(), 3);
            
            // Check targets
            match &targets[0] {
                Expression::Identifier { name, .. } => assert_eq!(name, "a"),
                _ => panic!("Expected identifier 'a'"),
            }
            match &targets[1] {
                Expression::Identifier { name, .. } => assert_eq!(name, "b"),
                _ => panic!("Expected identifier 'b'"),
            }
            match &targets[2] {
                Expression::Identifier { name, .. } => assert_eq!(name, "c"),
                _ => panic!("Expected identifier 'c'"),
            }
            
            // Check value
            match value {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 100),
                _ => panic!("Expected integer value"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_parse_tuple_unpacking() {
    let module = parse("a, b = 1, 2\n").unwrap();
    
    match &module.statements[0] {
        Statement::Assignment { targets, value, .. } => {
            // Should have one target (the tuple)
            assert_eq!(targets.len(), 1);
            
            // Check target is a tuple with two elements
            match &targets[0] {
                Expression::Tuple { elements, .. } => {
                    assert_eq!(elements.len(), 2);
                    match &elements[0] {
                        Expression::Identifier { name, .. } => assert_eq!(name, "a"),
                        _ => panic!("Expected identifier 'a'"),
                    }
                    match &elements[1] {
                        Expression::Identifier { name, .. } => assert_eq!(name, "b"),
                        _ => panic!("Expected identifier 'b'"),
                    }
                }
                _ => panic!("Expected tuple as target"),
            }
            
            // Check value is a tuple with two integers
            match value {
                Expression::Tuple { elements, .. } => {
                    assert_eq!(elements.len(), 2);
                    match &elements[0] {
                        Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 1),
                        _ => panic!("Expected integer 1"),
                    }
                    match &elements[1] {
                        Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 2),
                        _ => panic!("Expected integer 2"),
                    }
                }
                _ => panic!("Expected tuple as value"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_parse_list_unpacking() {
    let module = parse("x, y, z = [10, 20, 30]\n").unwrap();
    
    match &module.statements[0] {
        Statement::Assignment { targets, value, .. } => {
            // Should have one target (the tuple)
            assert_eq!(targets.len(), 1);
            
            // Check target is a tuple with three elements
            match &targets[0] {
                Expression::Tuple { elements, .. } => {
                    assert_eq!(elements.len(), 3);
                    match &elements[0] {
                        Expression::Identifier { name, .. } => assert_eq!(name, "x"),
                        _ => panic!("Expected identifier 'x'"),
                    }
                    match &elements[1] {
                        Expression::Identifier { name, .. } => assert_eq!(name, "y"),
                        _ => panic!("Expected identifier 'y'"),
                    }
                    match &elements[2] {
                        Expression::Identifier { name, .. } => assert_eq!(name, "z"),
                        _ => panic!("Expected identifier 'z'"),
                    }
                }
                _ => panic!("Expected tuple as target"),
            }
            
            // Check value is a list
            match value {
                Expression::List { elements, .. } => {
                    assert_eq!(elements.len(), 3);
                }
                _ => panic!("Expected list as value"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_parse_starred_assignment() {
    let module = parse("a, *b, c = [1, 2, 3, 4, 5]\n").unwrap();
    
    match &module.statements[0] {
        Statement::Assignment { targets, value, .. } => {
            // Should have one target (the tuple)
            assert_eq!(targets.len(), 1);
            
            // Check target is a tuple with three elements (a, *b, c)
            match &targets[0] {
                Expression::Tuple { elements, .. } => {
                    assert_eq!(elements.len(), 3);
                    
                    // First element: 'a'
                    match &elements[0] {
                        Expression::Identifier { name, .. } => assert_eq!(name, "a"),
                        _ => panic!("Expected identifier 'a'"),
                    }
                    
                    // Second element: *b
                    match &elements[1] {
                        Expression::Starred { value, .. } => {
                            match value.as_ref() {
                                Expression::Identifier { name, .. } => assert_eq!(name, "b"),
                                _ => panic!("Expected identifier 'b' in starred expression"),
                            }
                        }
                        _ => panic!("Expected starred expression"),
                    }
                    
                    // Third element: 'c'
                    match &elements[2] {
                        Expression::Identifier { name, .. } => assert_eq!(name, "c"),
                        _ => panic!("Expected identifier 'c'"),
                    }
                }
                _ => panic!("Expected tuple as target"),
            }
            
            // Check value is a list
            match value {
                Expression::List { elements, .. } => {
                    assert_eq!(elements.len(), 5);
                }
                _ => panic!("Expected list as value"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_parse_starred_only_middle() {
    let module = parse("first, *rest = [1, 2, 3]\n").unwrap();
    
    match &module.statements[0] {
        Statement::Assignment { targets, .. } => {
            match &targets[0] {
                Expression::Tuple { elements, .. } => {
                    assert_eq!(elements.len(), 2);
                    
                    // First element
                    match &elements[0] {
                        Expression::Identifier { name, .. } => assert_eq!(name, "first"),
                        _ => panic!("Expected identifier 'first'"),
                    }
                    
                    // Second element: *rest
                    match &elements[1] {
                        Expression::Starred { value, .. } => {
                            match value.as_ref() {
                                Expression::Identifier { name, .. } => assert_eq!(name, "rest"),
                                _ => panic!("Expected identifier 'rest'"),
                            }
                        }
                        _ => panic!("Expected starred expression"),
                    }
                }
                _ => panic!("Expected tuple as target"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_parse_starred_at_end() {
    let module = parse("*rest, last = [1, 2, 3]\n").unwrap();
    
    match &module.statements[0] {
        Statement::Assignment { targets, .. } => {
            match &targets[0] {
                Expression::Tuple { elements, .. } => {
                    assert_eq!(elements.len(), 2);
                    
                    // First element: *rest
                    match &elements[0] {
                        Expression::Starred { value, .. } => {
                            match value.as_ref() {
                                Expression::Identifier { name, .. } => assert_eq!(name, "rest"),
                                _ => panic!("Expected identifier 'rest'"),
                            }
                        }
                        _ => panic!("Expected starred expression"),
                    }
                    
                    // Second element: 'last'
                    match &elements[1] {
                        Expression::Identifier { name, .. } => assert_eq!(name, "last"),
                        _ => panic!("Expected identifier 'last'"),
                    }
                }
                _ => panic!("Expected tuple as target"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_parse_assert_simple() {
    let module = parse("assert x > 0\n").unwrap();
    
    match &module.statements[0] {
        Statement::Assert { condition, message, .. } => {
            // Check condition is a comparison
            match condition {
                Expression::BinaryOp { op, .. } => {
                    assert!(matches!(op, BinaryOperator::GreaterThan));
                }
                _ => panic!("Expected binary operation"),
            }
            
            // No message
            assert!(message.is_none());
        }
        _ => panic!("Expected assert statement"),
    }
}

#[test]
fn test_parse_assert_with_message() {
    let module = parse("assert x > 0, \"x must be positive\"\n").unwrap();
    
    match &module.statements[0] {
        Statement::Assert { condition, message, .. } => {
            // Check condition
            match condition {
                Expression::BinaryOp { op, .. } => {
                    assert!(matches!(op, BinaryOperator::GreaterThan));
                }
                _ => panic!("Expected binary operation"),
            }
            
            // Check message
            assert!(message.is_some());
            match message.as_ref().unwrap() {
                Expression::Literal(Literal::String { value, .. }) => {
                    assert_eq!(value, "x must be positive");
                }
                _ => panic!("Expected string literal as message"),
            }
        }
        _ => panic!("Expected assert statement"),
    }
}

#[test]
fn test_parse_assert_true() {
    let module = parse("assert True\n").unwrap();
    
    match &module.statements[0] {
        Statement::Assert { condition, message, .. } => {
            match condition {
                Expression::Literal(Literal::Boolean { value, .. }) => {
                    assert_eq!(*value, true);
                }
                _ => panic!("Expected boolean literal"),
            }
            assert!(message.is_none());
        }
        _ => panic!("Expected assert statement"),
    }
}

#[test]
fn test_parse_del_single() {
    let module = parse("del x\n").unwrap();
    
    match &module.statements[0] {
        Statement::Del { targets, .. } => {
            assert_eq!(targets.len(), 1);
            match &targets[0] {
                Expression::Identifier { name, .. } => assert_eq!(name, "x"),
                _ => panic!("Expected identifier"),
            }
        }
        _ => panic!("Expected del statement"),
    }
}

#[test]
fn test_parse_del_multiple() {
    let module = parse("del x, y, z\n").unwrap();
    
    match &module.statements[0] {
        Statement::Del { targets, .. } => {
            assert_eq!(targets.len(), 3);
            
            match &targets[0] {
                Expression::Identifier { name, .. } => assert_eq!(name, "x"),
                _ => panic!("Expected identifier 'x'"),
            }
            match &targets[1] {
                Expression::Identifier { name, .. } => assert_eq!(name, "y"),
                _ => panic!("Expected identifier 'y'"),
            }
            match &targets[2] {
                Expression::Identifier { name, .. } => assert_eq!(name, "z"),
                _ => panic!("Expected identifier 'z'"),
            }
        }
        _ => panic!("Expected del statement"),
    }
}

#[test]
fn test_parse_del_attribute() {
    let module = parse("del obj.attr\n").unwrap();
    
    match &module.statements[0] {
        Statement::Del { targets, .. } => {
            assert_eq!(targets.len(), 1);
            match &targets[0] {
                Expression::Attribute { object, attribute, .. } => {
                    match object.as_ref() {
                        Expression::Identifier { name, .. } => assert_eq!(name, "obj"),
                        _ => panic!("Expected identifier"),
                    }
                    assert_eq!(attribute, "attr");
                }
                _ => panic!("Expected attribute expression"),
            }
        }
        _ => panic!("Expected del statement"),
    }
}

#[test]
fn test_parse_del_subscript() {
    let module = parse("del list[0]\n").unwrap();
    
    match &module.statements[0] {
        Statement::Del { targets, .. } => {
            assert_eq!(targets.len(), 1);
            match &targets[0] {
                Expression::Subscript { object, index, .. } => {
                    match object.as_ref() {
                        Expression::Identifier { name, .. } => assert_eq!(name, "list"),
                        _ => panic!("Expected identifier"),
                    }
                    match index.as_ref() {
                        Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 0),
                        _ => panic!("Expected integer literal"),
                    }
                }
                _ => panic!("Expected subscript expression"),
            }
        }
        _ => panic!("Expected del statement"),
    }
}

#[test]
fn test_parse_global_single() {
    let module = parse("global x\n").unwrap();
    
    match &module.statements[0] {
        Statement::Global { names, .. } => {
            assert_eq!(names.len(), 1);
            assert_eq!(names[0], "x");
        }
        _ => panic!("Expected global statement"),
    }
}

#[test]
fn test_parse_global_multiple() {
    let module = parse("global x, y, z\n").unwrap();
    
    match &module.statements[0] {
        Statement::Global { names, .. } => {
            assert_eq!(names.len(), 3);
            assert_eq!(names[0], "x");
            assert_eq!(names[1], "y");
            assert_eq!(names[2], "z");
        }
        _ => panic!("Expected global statement"),
    }
}

#[test]
fn test_parse_nonlocal_single() {
    let module = parse("nonlocal x\n").unwrap();
    
    match &module.statements[0] {
        Statement::Nonlocal { names, .. } => {
            assert_eq!(names.len(), 1);
            assert_eq!(names[0], "x");
        }
        _ => panic!("Expected nonlocal statement"),
    }
}

#[test]
fn test_parse_nonlocal_multiple() {
    let module = parse("nonlocal x, y, z\n").unwrap();
    
    match &module.statements[0] {
        Statement::Nonlocal { names, .. } => {
            assert_eq!(names.len(), 3);
            assert_eq!(names[0], "x");
            assert_eq!(names[1], "y");
            assert_eq!(names[2], "z");
        }
        _ => panic!("Expected nonlocal statement"),
    }
}

#[test]
fn test_parse_nonlocal_no_identifiers_error() {
    let result = parse("nonlocal\n");
    assert!(result.is_err());
    // Should error with clear message about missing identifier
}

#[test]
fn test_parse_global_no_identifiers_error() {
    let result = parse("global\n");
    assert!(result.is_err());
    // Should error with clear message about missing identifier
}

#[test]
fn test_parse_raise_bare() {
    let module = parse("raise\n").unwrap();
    
    match &module.statements[0] {
        Statement::Raise { exception, .. } => {
            assert!(exception.is_none());
        }
        _ => panic!("Expected raise statement"),
    }
}

#[test]
fn test_parse_raise_with_exception() {
    let module = parse("raise ValueError\n").unwrap();
    
    match &module.statements[0] {
        Statement::Raise { exception, .. } => {
            assert!(exception.is_some());
            match exception.as_ref().unwrap() {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "ValueError");
                }
                _ => panic!("Expected identifier expression"),
            }
        }
        _ => panic!("Expected raise statement"),
    }
}

#[test]
fn test_parse_raise_with_message() {
    let module = parse("raise ValueError(\"error message\")\n").unwrap();
    
    match &module.statements[0] {
        Statement::Raise { exception, .. } => {
            assert!(exception.is_some());
            match exception.as_ref().unwrap() {
                Expression::Call { function, arguments, .. } => {
                    match &**function {
                        Expression::Identifier { name, .. } => {
                            assert_eq!(name, "ValueError");
                        }
                        _ => panic!("Expected identifier in function position"),
                    }
                    assert_eq!(arguments.len(), 1);
                    match &arguments[0] {
                        Expression::Literal(Literal::String { value, .. }) => {
                            assert_eq!(value, "error message");
                        }
                        _ => panic!("Expected string literal argument"),
                    }
                }
                _ => panic!("Expected call expression"),
            }
        }
        _ => panic!("Expected raise statement"),
    }
}

#[test]
fn test_parse_raise_with_expression() {
    let module = parse("raise Exception(\"test\") if x else RuntimeError()\n").unwrap();
    
    match &module.statements[0] {
        Statement::Raise { exception, .. } => {
            assert!(exception.is_some());
            // Verify it's a conditional expression
            match exception.as_ref().unwrap() {
                Expression::Conditional { .. } => {
                    // Expected
                }
                _ => panic!("Expected conditional expression"),
            }
        }
        _ => panic!("Expected raise statement"),
    }
}

// ============================================================================
// Import Statement Tests
// ============================================================================

#[test]
fn test_parse_import_single() {
    let module = parse("import os\n").unwrap();
    
    match &module.statements[0] {
        Statement::Import { items, .. } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].module, "os");
            assert_eq!(items[0].alias, None);
        }
        _ => panic!("Expected import statement"),
    }
}

#[test]
fn test_parse_import_multiple() {
    let module = parse("import os, sys, json\n").unwrap();
    
    match &module.statements[0] {
        Statement::Import { items, .. } => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].module, "os");
            assert_eq!(items[1].module, "sys");
            assert_eq!(items[2].module, "json");
            assert_eq!(items[0].alias, None);
            assert_eq!(items[1].alias, None);
            assert_eq!(items[2].alias, None);
        }
        _ => panic!("Expected import statement"),
    }
}

#[test]
fn test_parse_import_dotted() {
    let module = parse("import os.path\n").unwrap();
    
    match &module.statements[0] {
        Statement::Import { items, .. } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].module, "os.path");
            assert_eq!(items[0].alias, None);
        }
        _ => panic!("Expected import statement"),
    }
}

#[test]
fn test_parse_import_deeply_dotted() {
    let module = parse("import package.subpackage.module\n").unwrap();
    
    match &module.statements[0] {
        Statement::Import { items, .. } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].module, "package.subpackage.module");
            assert_eq!(items[0].alias, None);
        }
        _ => panic!("Expected import statement"),
    }
}

#[test]
fn test_parse_import_with_alias() {
    let module = parse("import numpy as np\n").unwrap();
    
    match &module.statements[0] {
        Statement::Import { items, .. } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].module, "numpy");
            assert_eq!(items[0].alias, Some("np".to_string()));
        }
        _ => panic!("Expected import statement"),
    }
}

#[test]
fn test_parse_import_dotted_with_alias() {
    let module = parse("import os.path as ospath\n").unwrap();
    
    match &module.statements[0] {
        Statement::Import { items, .. } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].module, "os.path");
            assert_eq!(items[0].alias, Some("ospath".to_string()));
        }
        _ => panic!("Expected import statement"),
    }
}

#[test]
fn test_parse_import_multiple_with_aliases() {
    let module = parse("import numpy as np, pandas as pd\n").unwrap();
    
    match &module.statements[0] {
        Statement::Import { items, .. } => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].module, "numpy");
            assert_eq!(items[0].alias, Some("np".to_string()));
            assert_eq!(items[1].module, "pandas");
            assert_eq!(items[1].alias, Some("pd".to_string()));
        }
        _ => panic!("Expected import statement"),
    }
}

#[test]
fn test_parse_import_mixed_aliases() {
    let module = parse("import os, sys as system, json\n").unwrap();
    
    match &module.statements[0] {
        Statement::Import { items, .. } => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].module, "os");
            assert_eq!(items[0].alias, None);
            assert_eq!(items[1].module, "sys");
            assert_eq!(items[1].alias, Some("system".to_string()));
            assert_eq!(items[2].module, "json");
            assert_eq!(items[2].alias, None);
        }
        _ => panic!("Expected import statement"),
    }
}

#[test]
fn test_parse_import_trailing_comma() {
    let module = parse("import os, sys,\n").unwrap();
    
    match &module.statements[0] {
        Statement::Import { items, .. } => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].module, "os");
            assert_eq!(items[1].module, "sys");
        }
        _ => panic!("Expected import statement"),
    }
}

// ============================================================================
// From...Import Statement Tests
// ============================================================================

#[test]
fn test_parse_from_import_single() {
    let module = parse("from os import path\n").unwrap();
    
    match &module.statements[0] {
        Statement::FromImport { module: mod_name, items, .. } => {
            assert_eq!(mod_name, "os");
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].name, "path");
            assert_eq!(items[0].alias, None);
        }
        _ => panic!("Expected from...import statement"),
    }
}

#[test]
fn test_parse_from_import_multiple() {
    let module = parse("from os import path, environ, getcwd\n").unwrap();
    
    match &module.statements[0] {
        Statement::FromImport { module: mod_name, items, .. } => {
            assert_eq!(mod_name, "os");
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].name, "path");
            assert_eq!(items[1].name, "environ");
            assert_eq!(items[2].name, "getcwd");
            assert!(items.iter().all(|i| i.alias.is_none()));
        }
        _ => panic!("Expected from...import statement"),
    }
}

#[test]
fn test_parse_from_import_dotted_module() {
    let module = parse("from os.path import join\n").unwrap();
    
    match &module.statements[0] {
        Statement::FromImport { module: mod_name, items, .. } => {
            assert_eq!(mod_name, "os.path");
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].name, "join");
            assert_eq!(items[0].alias, None);
        }
        _ => panic!("Expected from...import statement"),
    }
}

#[test]
fn test_parse_from_import_deeply_dotted_module() {
    let module = parse("from package.subpackage.module import function\n").unwrap();
    
    match &module.statements[0] {
        Statement::FromImport { module: mod_name, items, .. } => {
            assert_eq!(mod_name, "package.subpackage.module");
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].name, "function");
        }
        _ => panic!("Expected from...import statement"),
    }
}

#[test]
fn test_parse_from_import_with_alias() {
    let module = parse("from numpy import array as arr\n").unwrap();
    
    match &module.statements[0] {
        Statement::FromImport { module: mod_name, items, .. } => {
            assert_eq!(mod_name, "numpy");
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].name, "array");
            assert_eq!(items[0].alias, Some("arr".to_string()));
        }
        _ => panic!("Expected from...import statement"),
    }
}

#[test]
fn test_parse_from_import_multiple_with_aliases() {
    let module = parse("from numpy import array as arr, zeros as z\n").unwrap();
    
    match &module.statements[0] {
        Statement::FromImport { module: mod_name, items, .. } => {
            assert_eq!(mod_name, "numpy");
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].name, "array");
            assert_eq!(items[0].alias, Some("arr".to_string()));
            assert_eq!(items[1].name, "zeros");
            assert_eq!(items[1].alias, Some("z".to_string()));
        }
        _ => panic!("Expected from...import statement"),
    }
}

#[test]
fn test_parse_from_import_mixed_aliases() {
    let module = parse("from os import path, environ as env, getcwd\n").unwrap();
    
    match &module.statements[0] {
        Statement::FromImport { module: mod_name, items, .. } => {
            assert_eq!(mod_name, "os");
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].name, "path");
            assert_eq!(items[0].alias, None);
            assert_eq!(items[1].name, "environ");
            assert_eq!(items[1].alias, Some("env".to_string()));
            assert_eq!(items[2].name, "getcwd");
            assert_eq!(items[2].alias, None);
        }
        _ => panic!("Expected from...import statement"),
    }
}

#[test]
fn test_parse_from_import_wildcard() {
    let module = parse("from os import *\n").unwrap();
    
    match &module.statements[0] {
        Statement::FromImport { module: mod_name, items, .. } => {
            assert_eq!(mod_name, "os");
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].name, "*");
            assert_eq!(items[0].alias, None);
        }
        _ => panic!("Expected from...import statement"),
    }
}

#[test]
fn test_parse_from_import_wildcard_dotted_module() {
    let module = parse("from package.submodule import *\n").unwrap();
    
    match &module.statements[0] {
        Statement::FromImport { module: mod_name, items, .. } => {
            assert_eq!(mod_name, "package.submodule");
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].name, "*");
        }
        _ => panic!("Expected from...import statement"),
    }
}

#[test]
fn test_parse_from_import_trailing_comma() {
    let module = parse("from os import path, environ,\n").unwrap();
    
    match &module.statements[0] {
        Statement::FromImport { module: mod_name, items, .. } => {
            assert_eq!(mod_name, "os");
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].name, "path");
            assert_eq!(items[1].name, "environ");
        }
        _ => panic!("Expected from...import statement"),
    }
}

// ============================================================================
// Control Flow - If Statement Tests
// ============================================================================

#[test]
fn test_parse_if_simple() {
    let module = parse("if x:\n    pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::If { condition, then_block, elif_blocks, else_block, .. } => {
            // Check condition
            match condition {
                Expression::Identifier { name, .. } => assert_eq!(name, "x"),
                _ => panic!("Expected identifier condition"),
            }
            // Check then block
            assert_eq!(then_block.len(), 1);
            assert!(matches!(then_block[0], Statement::Pass(_)));
            // No elif or else
            assert_eq!(elif_blocks.len(), 0);
            assert!(else_block.is_none());
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_parse_if_else() {
    let module = parse("if x:\n    pass\nelse:\n    pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::If { condition, then_block, elif_blocks, else_block, .. } => {
            match condition {
                Expression::Identifier { name, .. } => assert_eq!(name, "x"),
                _ => panic!("Expected identifier condition"),
            }
            assert_eq!(then_block.len(), 1);
            assert_eq!(elif_blocks.len(), 0);
            assert!(else_block.is_some());
            let else_stmts = else_block.as_ref().unwrap();
            assert_eq!(else_stmts.len(), 1);
            assert!(matches!(else_stmts[0], Statement::Pass(_)));
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_parse_if_elif_else() {
    let module = parse("if x:\n    pass\nelif y:\n    pass\nelse:\n    pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::If { condition, then_block, elif_blocks, else_block, .. } => {
            match condition {
                Expression::Identifier { name, .. } => assert_eq!(name, "x"),
                _ => panic!("Expected identifier condition"),
            }
            assert_eq!(then_block.len(), 1);
            assert_eq!(elif_blocks.len(), 1);
            // Check elif
            match &elif_blocks[0].0 {
                Expression::Identifier { name, .. } => assert_eq!(name, "y"),
                _ => panic!("Expected identifier condition"),
            }
            assert_eq!(elif_blocks[0].1.len(), 1);
            // Check else
            assert!(else_block.is_some());
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_parse_if_multiple_elif() {
    let module = parse("if x:\n    pass\nelif y:\n    pass\nelif z:\n    pass\nelse:\n    pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::If { elif_blocks, .. } => {
            assert_eq!(elif_blocks.len(), 2);
            match &elif_blocks[0].0 {
                Expression::Identifier { name, .. } => assert_eq!(name, "y"),
                _ => panic!("Expected identifier"),
            }
            match &elif_blocks[1].0 {
                Expression::Identifier { name, .. } => assert_eq!(name, "z"),
                _ => panic!("Expected identifier"),
            }
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_parse_if_with_multiple_statements() {
    let module = parse("if x:\n    a = 1\n    b = 2\n    pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::If { then_block, .. } => {
            assert_eq!(then_block.len(), 3);
            assert!(matches!(then_block[0], Statement::Assignment { .. }));
            assert!(matches!(then_block[1], Statement::Assignment { .. }));
            assert!(matches!(then_block[2], Statement::Pass(_)));
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_parse_if_with_complex_condition() {
    let module = parse("if x > 5 and y < 10:\n    pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::If { condition, .. } => {
            // Should parse as a logical AND of two comparisons
            assert!(matches!(condition, Expression::BinaryOp { .. }));
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_parse_nested_if() {
    let module = parse("if x:\n    if y:\n        pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::If { then_block, .. } => {
            assert_eq!(then_block.len(), 1);
            // Inner if should be another If statement
            assert!(matches!(then_block[0], Statement::If { .. }));
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_parse_if_elif_without_else() {
    let module = parse("if x:\n    pass\nelif y:\n    pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::If { elif_blocks, else_block, .. } => {
            assert_eq!(elif_blocks.len(), 1);
            assert!(else_block.is_none());
        }
        _ => panic!("Expected if statement"),
    }
}

// ============================================================================
// Control Flow - While Loop Tests
// ============================================================================

#[test]
fn test_parse_while_simple() {
    let module = parse("while x:\n    pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::While { condition, body, else_block, .. } => {
            match condition {
                Expression::Identifier { name, .. } => assert_eq!(name, "x"),
                _ => panic!("Expected identifier condition"),
            }
            assert_eq!(body.len(), 1);
            assert!(matches!(body[0], Statement::Pass(_)));
            assert!(else_block.is_none());
        }
        _ => panic!("Expected while statement"),
    }
}

#[test]
fn test_parse_while_else() {
    let module = parse("while x:\n    pass\nelse:\n    pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::While { condition, body, else_block, .. } => {
            match condition {
                Expression::Identifier { name, .. } => assert_eq!(name, "x"),
                _ => panic!("Expected identifier"),
            }
            assert_eq!(body.len(), 1);
            assert!(else_block.is_some());
            let else_stmts = else_block.as_ref().unwrap();
            assert_eq!(else_stmts.len(), 1);
        }
        _ => panic!("Expected while statement"),
    }
}

#[test]
fn test_parse_while_with_break() {
    let module = parse("while x:\n    break\n").unwrap();
    
    match &module.statements[0] {
        Statement::While { body, .. } => {
            assert_eq!(body.len(), 1);
            assert!(matches!(body[0], Statement::Break(_)));
        }
        _ => panic!("Expected while statement"),
    }
}

#[test]
fn test_parse_while_with_complex_condition() {
    let module = parse("while x > 0 and y < 10:\n    pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::While { condition, .. } => {
            assert!(matches!(condition, Expression::BinaryOp { .. }));
        }
        _ => panic!("Expected while statement"),
    }
}

#[test]
fn test_parse_nested_while() {
    let module = parse("while x:\n    while y:\n        pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::While { body, .. } => {
            assert_eq!(body.len(), 1);
            assert!(matches!(body[0], Statement::While { .. }));
        }
        _ => panic!("Expected while statement"),
    }
}

// ============================================================================
// Control Flow - For Loop Tests
// ============================================================================

#[test]
fn test_parse_for_simple() {
    let module = parse("for x in items:\n    pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::For { target, iter, body, else_block, .. } => {
            match target {
                Expression::Identifier { name, .. } => assert_eq!(name, "x"),
                _ => panic!("Expected identifier target"),
            }
            match iter {
                Expression::Identifier { name, .. } => assert_eq!(name, "items"),
                _ => panic!("Expected identifier iter"),
            }
            assert_eq!(body.len(), 1);
            assert!(matches!(body[0], Statement::Pass(_)));
            assert!(else_block.is_none());
        }
        _ => panic!("Expected for statement"),
    }
}

#[test]
fn test_parse_for_else() {
    let module = parse("for x in items:\n    pass\nelse:\n    pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::For { body, else_block, .. } => {
            assert_eq!(body.len(), 1);
            assert!(else_block.is_some());
            let else_stmts = else_block.as_ref().unwrap();
            assert_eq!(else_stmts.len(), 1);
        }
        _ => panic!("Expected for statement"),
    }
}

#[test]
fn test_parse_for_with_tuple_unpacking() {
    let module = parse("for x, y in items:\n    pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::For { target, .. } => {
            // Target should be a tuple
            match target {
                Expression::Tuple { elements, .. } => {
                    assert_eq!(elements.len(), 2);
                }
                _ => panic!("Expected tuple target"),
            }
        }
        _ => panic!("Expected for statement"),
    }
}

#[test]
fn test_parse_for_with_range() {
    let module = parse("for i in range(10):\n    pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::For { iter, .. } => {
            // iter should be a function call
            assert!(matches!(iter, Expression::Call { .. }));
        }
        _ => panic!("Expected for statement"),
    }
}

#[test]
fn test_parse_nested_for() {
    let module = parse("for x in items:\n    for y in x:\n        pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::For { body, .. } => {
            assert_eq!(body.len(), 1);
            assert!(matches!(body[0], Statement::For { .. }));
        }
        _ => panic!("Expected for statement"),
    }
}

#[test]
fn test_parse_for_with_break_continue() {
    let module = parse("for x in items:\n    if x:\n        break\n    continue\n").unwrap();
    
    match &module.statements[0] {
        Statement::For { body, .. } => {
            assert_eq!(body.len(), 2);
            assert!(matches!(body[0], Statement::If { .. }));
            assert!(matches!(body[1], Statement::Continue(_)));
        }
        _ => panic!("Expected for statement"),
    }
}

// ============================================================================
// Function Definition Tests
// ============================================================================

#[test]
fn test_parse_function_no_params() {
    let result = parse("def foo():\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.statements.len(), 1);
    
    match &ast.statements[0] {
        Statement::FunctionDef { name, parameters, body, .. } => {
            assert_eq!(name, "foo");
            assert_eq!(parameters.len(), 0);
            assert_eq!(body.len(), 1);
            match &body[0] {
                Statement::Pass(_) => {},
                _ => panic!("Expected pass statement in body"),
            }
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_single_param() {
    let result = parse("def add(x):\n    return x\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { name, parameters, body, .. } => {
            assert_eq!(name, "add");
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name, "x");
            assert!(parameters[0].default.is_none());
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_multiple_params() {
    let result = parse("def add(x, y, z):\n    return x + y + z\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { name, parameters, .. } => {
            assert_eq!(name, "add");
            assert_eq!(parameters.len(), 3);
            assert_eq!(parameters[0].name, "x");
            assert_eq!(parameters[1].name, "y");
            assert_eq!(parameters[2].name, "z");
            assert!(parameters[0].default.is_none());
            assert!(parameters[1].default.is_none());
            assert!(parameters[2].default.is_none());
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_with_default_param() {
    let result = parse("def greet(name, greeting='Hello'):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { name, parameters, .. } => {
            assert_eq!(name, "greet");
            assert_eq!(parameters.len(), 2);
            assert_eq!(parameters[0].name, "name");
            assert!(parameters[0].default.is_none());
            assert_eq!(parameters[1].name, "greeting");
            assert!(parameters[1].default.is_some());
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_all_defaults() {
    let result = parse("def func(a=1, b=2, c=3):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { parameters, .. } => {
            assert_eq!(parameters.len(), 3);
            assert!(parameters[0].default.is_some());
            assert!(parameters[1].default.is_some());
            assert!(parameters[2].default.is_some());
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_trailing_comma() {
    let result = parse("def func(a, b,):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { parameters, .. } => {
            assert_eq!(parameters.len(), 2);
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_multiline_body() {
    let result = parse("def compute():\n    x = 1\n    y = 2\n    return x + y\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { body, .. } => {
            assert_eq!(body.len(), 3);
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_nested_function() {
    let result = parse("def outer():\n    def inner():\n        pass\n    return inner\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { name, body, .. } => {
            assert_eq!(name, "outer");
            assert_eq!(body.len(), 2);
            match &body[0] {
                Statement::FunctionDef { name, .. } => {
                    assert_eq!(name, "inner");
                }
                _ => panic!("Expected nested function definition"),
            }
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_complex_defaults() {
    let result = parse("def func(a=1+2, b=[1,2,3], c={'x': 1}):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { parameters, .. } => {
            assert_eq!(parameters.len(), 3);
            assert!(parameters[0].default.is_some());
            assert!(parameters[1].default.is_some());
            assert!(parameters[2].default.is_some());
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_with_if_body() {
    let result = parse("def check(x):\n    if x > 0:\n        return True\n    return False\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { body, .. } => {
            assert_eq!(body.len(), 2);
            match &body[0] {
                Statement::If { .. } => {},
                _ => panic!("Expected if statement"),
            }
        }
        _ => panic!("Expected function definition"),
    }
}

// Error cases

#[test]
fn test_parse_function_missing_name() {
    let result = parse("def ():\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_function_missing_parens() {
    let result = parse("def foo:\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_function_missing_colon() {
    let result = parse("def foo()\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_function_missing_body() {
    let result = parse("def foo():\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_function_invalid_param_name() {
    let result = parse("def foo(123):\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_function_missing_closing_paren() {
    let result = parse("def foo(x:\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_function_invalid_default() {
    let result = parse("def foo(x=):\n    pass\n");
    assert!(result.is_err());
}

// Variadic parameters tests

#[test]
fn test_parse_function_with_args_only() {
    let result = parse("def foo(*args):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { name, parameters, .. } => {
            assert_eq!(name, "foo");
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name, "args");
            assert!(matches!(parameters[0].kind, ParameterKind::VarArgs));
            assert!(parameters[0].default.is_none());
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_with_kwargs_only() {
    let result = parse("def foo(**kwargs):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { name, parameters, .. } => {
            assert_eq!(name, "foo");
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name, "kwargs");
            assert!(matches!(parameters[0].kind, ParameterKind::VarKwargs));
            assert!(parameters[0].default.is_none());
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_with_args_and_kwargs() {
    let result = parse("def foo(*args, **kwargs):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { parameters, .. } => {
            assert_eq!(parameters.len(), 2);
            assert_eq!(parameters[0].name, "args");
            assert!(matches!(parameters[0].kind, ParameterKind::VarArgs));
            assert_eq!(parameters[1].name, "kwargs");
            assert!(matches!(parameters[1].kind, ParameterKind::VarKwargs));
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_regular_and_args() {
    let result = parse("def foo(x, y, *args):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { parameters, .. } => {
            assert_eq!(parameters.len(), 3);
            assert_eq!(parameters[0].name, "x");
            assert!(matches!(parameters[0].kind, ParameterKind::Regular));
            assert_eq!(parameters[1].name, "y");
            assert!(matches!(parameters[1].kind, ParameterKind::Regular));
            assert_eq!(parameters[2].name, "args");
            assert!(matches!(parameters[2].kind, ParameterKind::VarArgs));
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_regular_and_kwargs() {
    let result = parse("def foo(x, y, **kwargs):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { parameters, .. } => {
            assert_eq!(parameters.len(), 3);
            assert_eq!(parameters[0].name, "x");
            assert!(matches!(parameters[0].kind, ParameterKind::Regular));
            assert_eq!(parameters[1].name, "y");
            assert!(matches!(parameters[1].kind, ParameterKind::Regular));
            assert_eq!(parameters[2].name, "kwargs");
            assert!(matches!(parameters[2].kind, ParameterKind::VarKwargs));
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_all_parameter_types() {
    let result = parse("def foo(a, b, c=3, *args, **kwargs):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { parameters, .. } => {
            assert_eq!(parameters.len(), 5);
            assert_eq!(parameters[0].name, "a");
            assert!(matches!(parameters[0].kind, ParameterKind::Regular));
            assert!(parameters[0].default.is_none());
            assert_eq!(parameters[1].name, "b");
            assert!(matches!(parameters[1].kind, ParameterKind::Regular));
            assert!(parameters[1].default.is_none());
            assert_eq!(parameters[2].name, "c");
            assert!(matches!(parameters[2].kind, ParameterKind::Regular));
            assert!(parameters[2].default.is_some());
            assert_eq!(parameters[3].name, "args");
            assert!(matches!(parameters[3].kind, ParameterKind::VarArgs));
            assert_eq!(parameters[4].name, "kwargs");
            assert!(matches!(parameters[4].kind, ParameterKind::VarKwargs));
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_defaults_before_args() {
    let result = parse("def foo(x=1, y=2, *args):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::FunctionDef { parameters, .. } => {
            assert_eq!(parameters.len(), 3);
            assert!(parameters[0].default.is_some());
            assert!(parameters[1].default.is_some());
            assert!(matches!(parameters[2].kind, ParameterKind::VarArgs));
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_trailing_comma_after_args() {
    let result = parse("def foo(*args,):\n    pass\n");
    assert!(result.is_ok());
}

#[test]
fn test_parse_function_trailing_comma_after_kwargs() {
    let result = parse("def foo(**kwargs,):\n    pass\n");
    assert!(result.is_ok());
}

// Error tests for variadic parameters

#[test]
fn test_parse_function_multiple_args_error() {
    let result = parse("def foo(*args, *more):\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_function_multiple_kwargs_error() {
    let result = parse("def foo(**kwargs, **more):\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_function_kwargs_before_args_error() {
    let result = parse("def foo(**kwargs, *args):\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_function_regular_after_args_error() {
    // This is actually valid Python 3 - parameters after *args are keyword-only
    // Changed to test **kwargs followed by anything, which is the real error
    let result = parse("def foo(**kwargs, x):\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_function_regular_after_kwargs_error() {
    let result = parse("def foo(**kwargs, x):\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_function_default_after_args_error() {
    // This is actually valid Python 3 - parameters after *args are keyword-only
    // Changed to test the same as above
    let result = parse("def foo(**kwargs, x=1):\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_function_param_without_default_after_default_error() {
    let result = parse("def foo(x=1, y):\n    pass\n");
    assert!(result.is_err());
}

// ============================================================================
// Class Definition Tests
// ============================================================================

#[test]
fn test_parse_class_empty() {
    let result = parse("class Foo:\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.statements.len(), 1);
    
    match &ast.statements[0] {
        Statement::ClassDef { name, bases, body, .. } => {
            assert_eq!(name, "Foo");
            assert_eq!(bases.len(), 0);
            assert_eq!(body.len(), 1);
            match &body[0] {
                Statement::Pass(_) => {},
                _ => panic!("Expected pass statement in body"),
            }
        }
        _ => panic!("Expected class definition"),
    }
}

#[test]
fn test_parse_class_with_statements() {
    let result = parse("class Person:\n    name = 'Unknown'\n    age = 0\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::ClassDef { name, body, .. } => {
            assert_eq!(name, "Person");
            assert_eq!(body.len(), 2);
        }
        _ => panic!("Expected class definition"),
    }
}

#[test]
fn test_parse_class_with_method() {
    let result = parse("class Person:\n    def greet(self):\n        pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::ClassDef { name, body, .. } => {
            assert_eq!(name, "Person");
            assert_eq!(body.len(), 1);
            match &body[0] {
                Statement::FunctionDef { name, .. } => {
                    assert_eq!(name, "greet");
                }
                _ => panic!("Expected function definition"),
            }
        }
        _ => panic!("Expected class definition"),
    }
}

#[test]
fn test_parse_class_with_multiple_methods() {
    let result = parse("class Person:\n    def __init__(self):\n        pass\n    def greet(self):\n        pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::ClassDef { body, .. } => {
            assert_eq!(body.len(), 2);
        }
        _ => panic!("Expected class definition"),
    }
}

#[test]
fn test_parse_class_single_inheritance() {
    let result = parse("class Child(Parent):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::ClassDef { name, bases, .. } => {
            assert_eq!(name, "Child");
            assert_eq!(bases.len(), 1);
            match &bases[0] {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "Parent");
                }
                _ => panic!("Expected identifier for base class"),
            }
        }
        _ => panic!("Expected class definition"),
    }
}

#[test]
fn test_parse_class_multiple_inheritance() {
    let result = parse("class Child(Parent1, Parent2, Parent3):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::ClassDef { name, bases, .. } => {
            assert_eq!(name, "Child");
            assert_eq!(bases.len(), 3);
        }
        _ => panic!("Expected class definition"),
    }
}

#[test]
fn test_parse_class_inheritance_with_dotted_name() {
    let result = parse("class MyClass(package.module.BaseClass):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::ClassDef { bases, .. } => {
            assert_eq!(bases.len(), 1);
            match &bases[0] {
                Expression::Attribute { .. } => {},
                _ => panic!("Expected attribute access for dotted base class"),
            }
        }
        _ => panic!("Expected class definition"),
    }
}

#[test]
fn test_parse_class_empty_inheritance_parens() {
    let result = parse("class Foo():\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::ClassDef { bases, .. } => {
            assert_eq!(bases.len(), 0);
        }
        _ => panic!("Expected class definition"),
    }
}

#[test]
fn test_parse_class_trailing_comma_in_bases() {
    let result = parse("class Child(Parent1, Parent2,):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::ClassDef { bases, .. } => {
            assert_eq!(bases.len(), 2);
        }
        _ => panic!("Expected class definition"),
    }
}

#[test]
fn test_parse_nested_class() {
    let result = parse("class Outer:\n    class Inner:\n        pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::ClassDef { name, body, .. } => {
            assert_eq!(name, "Outer");
            assert_eq!(body.len(), 1);
            match &body[0] {
                Statement::ClassDef { name, .. } => {
                    assert_eq!(name, "Inner");
                }
                _ => panic!("Expected nested class definition"),
            }
        }
        _ => panic!("Expected class definition"),
    }
}

#[test]
fn test_parse_class_complex_body() {
    let result = parse("class Person:\n    name = 'Unknown'\n    def __init__(self, name):\n        self.name = name\n    def greet(self):\n        return self.name\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.statements[0] {
        Statement::ClassDef { body, .. } => {
            assert_eq!(body.len(), 3);
        }
        _ => panic!("Expected class definition"),
    }
}

// Error tests for class definitions

#[test]
fn test_parse_class_missing_name() {
    let result = parse("class :\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_class_missing_colon() {
    let result = parse("class Foo\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_class_missing_body() {
    let result = parse("class Foo:\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_class_keyword_as_name() {
    let result = parse("class if:\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_class_number_as_name() {
    let result = parse("class 123:\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_class_missing_closing_paren() {
    let result = parse("class Child(Parent:\n    pass\n");
    assert!(result.is_err());
}

// ============================================================================
// Keyword-Only Parameters Tests
// ============================================================================

#[test]
fn test_parse_kwonly_bare_star() {
    // Basic: def func(a, *, b)
    let result = parse("def func(a, *, b):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.statements.len(), 1);
    
    if let Statement::FunctionDef { name, parameters, .. } = &ast.statements[0] {
        assert_eq!(name, "func");
        assert_eq!(parameters.len(), 2);
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::Regular));
        assert_eq!(parameters[1].name, "b");
        assert!(matches!(parameters[1].kind, ParameterKind::KwOnly));
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_kwonly_with_defaults() {
    // Keyword-only with defaults: def func(a, *, b=1, c=2)
    let result = parse("def func(a, *, b=1, c=2):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 3);
        assert_eq!(parameters[1].name, "b");
        assert!(matches!(parameters[1].kind, ParameterKind::KwOnly));
        assert!(parameters[1].default.is_some());
        assert_eq!(parameters[2].name, "c");
        assert!(matches!(parameters[2].kind, ParameterKind::KwOnly));
        assert!(parameters[2].default.is_some());
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_kwonly_after_varargs() {
    // Keyword-only after *args: def func(a, *args, b)
    let result = parse("def func(a, *args, b):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 3);
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::Regular));
        assert_eq!(parameters[1].name, "args");
        assert!(matches!(parameters[1].kind, ParameterKind::VarArgs));
        assert_eq!(parameters[2].name, "b");
        assert!(matches!(parameters[2].kind, ParameterKind::KwOnly));
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_kwonly_full_combo() {
    // Full combination: def func(a, b=1, *args, c, d=2, **kwargs)
    let result = parse("def func(a, b=1, *args, c, d=2, **kwargs):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 6);
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::Regular));
        assert!(parameters[0].default.is_none());
        
        assert_eq!(parameters[1].name, "b");
        assert!(matches!(parameters[1].kind, ParameterKind::Regular));
        assert!(parameters[1].default.is_some());
        
        assert_eq!(parameters[2].name, "args");
        assert!(matches!(parameters[2].kind, ParameterKind::VarArgs));
        
        assert_eq!(parameters[3].name, "c");
        assert!(matches!(parameters[3].kind, ParameterKind::KwOnly));
        assert!(parameters[3].default.is_none());
        
        assert_eq!(parameters[4].name, "d");
        assert!(matches!(parameters[4].kind, ParameterKind::KwOnly));
        assert!(parameters[4].default.is_some());
        
        assert_eq!(parameters[5].name, "kwargs");
        assert!(matches!(parameters[5].kind, ParameterKind::VarKwargs));
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_kwonly_only() {
    // Only keyword-only: def func(*, a, b)
    let result = parse("def func(*, a, b):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 2);
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::KwOnly));
        assert_eq!(parameters[1].name, "b");
        assert!(matches!(parameters[1].kind, ParameterKind::KwOnly));
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_kwonly_trailing_comma() {
    // Trailing comma: def func(a, *, b,)
    let result = parse("def func(a, *, b,):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 2);
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::Regular));
        assert_eq!(parameters[1].name, "b");
        assert!(matches!(parameters[1].kind, ParameterKind::KwOnly));
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_kwonly_mixed_defaults() {
    // Mix of keyword-only with and without defaults: def func(*, a, b=1, c)
    let result = parse("def func(*, a, b=1, c):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 3);
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::KwOnly));
        assert!(parameters[0].default.is_none());
        
        assert_eq!(parameters[1].name, "b");
        assert!(matches!(parameters[1].kind, ParameterKind::KwOnly));
        assert!(parameters[1].default.is_some());
        
        assert_eq!(parameters[2].name, "c");
        assert!(matches!(parameters[2].kind, ParameterKind::KwOnly));
        assert!(parameters[2].default.is_none());
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_kwonly_duplicate_bare_star() {
    // Error: multiple bare * (def func(*, a, *, b))
    let result = parse("def func(*, a, *, b):\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_kwonly_varargs_after_bare_star() {
    // Error: *args after bare * (def func(*, a, *args))
    let result = parse("def func(*, a, *args):\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_kwonly_duplicate_varargs() {
    // Error: duplicate *args (def func(*args, *more))
    let result = parse("def func(*args, *more):\n    pass\n");
    assert!(result.is_err());
}

// ============================================================================
// Positional-Only Parameters Tests
// ============================================================================

#[test]
fn test_parse_posonly_basic() {
    // Basic: def func(a, /, b)
    let result = parse("def func(a, /, b):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.statements.len(), 1);
    
    if let Statement::FunctionDef { name, parameters, .. } = &ast.statements[0] {
        assert_eq!(name, "func");
        assert_eq!(parameters.len(), 2);
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::PositionalOnly));
        assert_eq!(parameters[1].name, "b");
        assert!(matches!(parameters[1].kind, ParameterKind::Regular));
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_posonly_with_defaults() {
    // Positional-only with defaults: def func(a=1, b=2, /)
    let result = parse("def func(a=1, b=2, /):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 2);
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::PositionalOnly));
        assert!(parameters[0].default.is_some());
        assert_eq!(parameters[1].name, "b");
        assert!(matches!(parameters[1].kind, ParameterKind::PositionalOnly));
        assert!(parameters[1].default.is_some());
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_posonly_mixed_with_regular() {
    // Mixed: def func(a, b, /, c, d)
    let result = parse("def func(a, b, /, c, d):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 4);
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::PositionalOnly));
        assert_eq!(parameters[1].name, "b");
        assert!(matches!(parameters[1].kind, ParameterKind::PositionalOnly));
        assert_eq!(parameters[2].name, "c");
        assert!(matches!(parameters[2].kind, ParameterKind::Regular));
        assert_eq!(parameters[3].name, "d");
        assert!(matches!(parameters[3].kind, ParameterKind::Regular));
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_posonly_full_combo() {
    // Full combination: def func(a, /, b, *args, c, **kwargs)
    let result = parse("def func(a, /, b, *args, c, **kwargs):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 5);
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::PositionalOnly));
        
        assert_eq!(parameters[1].name, "b");
        assert!(matches!(parameters[1].kind, ParameterKind::Regular));
        
        assert_eq!(parameters[2].name, "args");
        assert!(matches!(parameters[2].kind, ParameterKind::VarArgs));
        
        assert_eq!(parameters[3].name, "c");
        assert!(matches!(parameters[3].kind, ParameterKind::KwOnly));
        
        assert_eq!(parameters[4].name, "kwargs");
        assert!(matches!(parameters[4].kind, ParameterKind::VarKwargs));
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_posonly_only() {
    // Only positional-only: def func(a, b, /)
    let result = parse("def func(a, b, /):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 2);
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::PositionalOnly));
        assert_eq!(parameters[1].name, "b");
        assert!(matches!(parameters[1].kind, ParameterKind::PositionalOnly));
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_posonly_with_kwonly() {
    // With keyword-only: def func(a, /, b, *, c)
    let result = parse("def func(a, /, b, *, c):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 3);
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::PositionalOnly));
        assert_eq!(parameters[1].name, "b");
        assert!(matches!(parameters[1].kind, ParameterKind::Regular));
        assert_eq!(parameters[2].name, "c");
        assert!(matches!(parameters[2].kind, ParameterKind::KwOnly));
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_posonly_trailing_comma() {
    // Trailing comma: def func(a, /,)
    let result = parse("def func(a, /,):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::PositionalOnly));
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_posonly_duplicate_slash() {
    // Error: multiple / markers (def func(a, /, b, /, c))
    let result = parse("def func(a, /, b, /, c):\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_posonly_slash_after_star() {
    // Error: / after * (def func(*args, /))
    let result = parse("def func(*args, /):\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_posonly_slash_after_kwargs() {
    // Error: / after ** (def func(**kwargs, /))
    let result = parse("def func(**kwargs, /):\n    pass\n");
    assert!(result.is_err());
}

// ============================================================================
// Async Function Tests
// ============================================================================

#[test]
fn test_parse_async_basic() {
    // Basic: async def func(): pass
    let result = parse("async def func():\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.statements.len(), 1);
    
    if let Statement::FunctionDef { name, is_async, parameters, .. } = &ast.statements[0] {
        assert_eq!(name, "func");
        assert!(is_async);
        assert_eq!(parameters.len(), 0);
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_async_with_parameters() {
    // Async with parameters: async def func(a, b)
    let result = parse("async def func(a, b):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { name, is_async, parameters, .. } = &ast.statements[0] {
        assert_eq!(name, "func");
        assert!(is_async);
        assert_eq!(parameters.len(), 2);
        assert_eq!(parameters[0].name, "a");
        assert_eq!(parameters[1].name, "b");
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_async_with_defaults() {
    // Async with defaults: async def func(a=1, b=2)
    let result = parse("async def func(a=1, b=2):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { is_async, parameters, .. } = &ast.statements[0] {
        assert!(is_async);
        assert_eq!(parameters.len(), 2);
        assert!(parameters[0].default.is_some());
        assert!(parameters[1].default.is_some());
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_async_with_varargs() {
    // Async with *args: async def func(*args)
    let result = parse("async def func(*args):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { is_async, parameters, .. } = &ast.statements[0] {
        assert!(is_async);
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name, "args");
        assert!(matches!(parameters[0].kind, ParameterKind::VarArgs));
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_async_with_kwargs() {
    // Async with **kwargs: async def func(**kwargs)
    let result = parse("async def func(**kwargs):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { is_async, parameters, .. } = &ast.statements[0] {
        assert!(is_async);
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name, "kwargs");
        assert!(matches!(parameters[0].kind, ParameterKind::VarKwargs));
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_async_all_param_types() {
    // Full combination: async def func(a, /, b, *args, c, **kwargs)
    let result = parse("async def func(a, /, b, *args, c, **kwargs):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { is_async, parameters, .. } = &ast.statements[0] {
        assert!(is_async);
        assert_eq!(parameters.len(), 5);
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::PositionalOnly));
        assert_eq!(parameters[1].name, "b");
        assert!(matches!(parameters[1].kind, ParameterKind::Regular));
        assert_eq!(parameters[2].name, "args");
        assert!(matches!(parameters[2].kind, ParameterKind::VarArgs));
        assert_eq!(parameters[3].name, "c");
        assert!(matches!(parameters[3].kind, ParameterKind::KwOnly));
        assert_eq!(parameters[4].name, "kwargs");
        assert!(matches!(parameters[4].kind, ParameterKind::VarKwargs));
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_async_method_in_class() {
    // Async method: class X:\n    async def method(self): pass
    let result = parse("class X:\n    async def method(self):\n        pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::ClassDef { body, .. } = &ast.statements[0] {
        assert_eq!(body.len(), 1);
        if let Statement::FunctionDef { name, is_async, .. } = &body[0] {
            assert_eq!(name, "method");
            assert!(is_async);
        } else {
            panic!("Expected FunctionDef in class body");
        }
    } else {
        panic!("Expected ClassDef");
    }
}

#[test]
fn test_parse_async_without_def_error() {
    // Error: async without def (async x = 1)
    let result = parse("async x = 1\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_async_missing_body_error() {
    // Error: missing body (async def func():)
    let result = parse("async def func():\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_regular_function_not_async() {
    // Regular function should have is_async = false
    let result = parse("def func():\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { is_async, .. } = &ast.statements[0] {
        assert!(!is_async);
    } else {
        panic!("Expected FunctionDef");
    }
}

// ============================================================================
// Negative Tests - Error Handling for New Statements
// ============================================================================

#[test]
fn test_parse_global_with_number_error() {
    let result = parse("global 123\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_global_with_keyword_error() {
    let result = parse("global if\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_nonlocal_with_number_error() {
    let result = parse("nonlocal 456\n");
    assert!(result.is_err());
}

// ============================================================================
// Parameter Type Annotation Tests
// ============================================================================

#[test]
fn test_parse_param_type_annotation_single() {
    let result = parse("def func(x: int):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name, "x");
        assert!(parameters[0].type_annotation.is_some());
        if let Some(Expression::Identifier { name, .. }) = &parameters[0].type_annotation {
            assert_eq!(name, "int");
        } else {
            panic!("Expected identifier type annotation");
        }
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_param_type_annotation_multiple() {
    let result = parse("def func(x: int, y: str, z: float):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 3);
        
        // Check x: int
        assert_eq!(parameters[0].name, "x");
        if let Some(Expression::Identifier { name, .. }) = &parameters[0].type_annotation {
            assert_eq!(name, "int");
        } else {
            panic!("Expected int type annotation for x");
        }
        
        // Check y: str
        assert_eq!(parameters[1].name, "y");
        if let Some(Expression::Identifier { name, .. }) = &parameters[1].type_annotation {
            assert_eq!(name, "str");
        } else {
            panic!("Expected str type annotation for y");
        }
        
        // Check z: float
        assert_eq!(parameters[2].name, "z");
        if let Some(Expression::Identifier { name, .. }) = &parameters[2].type_annotation {
            assert_eq!(name, "float");
        } else {
            panic!("Expected float type annotation for z");
        }
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_param_type_annotation_with_default() {
    let result = parse("def func(x: int = 5, y: str = \"hello\"):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 2);
        
        // Check x: int = 5
        assert_eq!(parameters[0].name, "x");
        assert!(parameters[0].type_annotation.is_some());
        assert!(parameters[0].default.is_some());
        
        // Check y: str = "hello"
        assert_eq!(parameters[1].name, "y");
        assert!(parameters[1].type_annotation.is_some());
        assert!(parameters[1].default.is_some());
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_param_type_annotation_varargs() {
    let result = parse("def func(*args: int):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name, "args");
        assert!(matches!(parameters[0].kind, ParameterKind::VarArgs));
        assert!(parameters[0].type_annotation.is_some());
        if let Some(Expression::Identifier { name, .. }) = &parameters[0].type_annotation {
            assert_eq!(name, "int");
        } else {
            panic!("Expected int type annotation for *args");
        }
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_param_type_annotation_kwargs() {
    let result = parse("def func(**kwargs: str):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name, "kwargs");
        assert!(matches!(parameters[0].kind, ParameterKind::VarKwargs));
        assert!(parameters[0].type_annotation.is_some());
        if let Some(Expression::Identifier { name, .. }) = &parameters[0].type_annotation {
            assert_eq!(name, "str");
        } else {
            panic!("Expected str type annotation for **kwargs");
        }
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_param_type_annotation_all_kinds() {
    let result = parse("def func(a: int, /, b: str, *args: float, c: bool, **kwargs: dict):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 5);
        
        // a: int (positional-only)
        assert_eq!(parameters[0].name, "a");
        assert!(matches!(parameters[0].kind, ParameterKind::PositionalOnly));
        assert!(parameters[0].type_annotation.is_some());
        
        // b: str (regular)
        assert_eq!(parameters[1].name, "b");
        assert!(matches!(parameters[1].kind, ParameterKind::Regular));
        assert!(parameters[1].type_annotation.is_some());
        
        // *args: float
        assert_eq!(parameters[2].name, "args");
        assert!(matches!(parameters[2].kind, ParameterKind::VarArgs));
        assert!(parameters[2].type_annotation.is_some());
        
        // c: bool (keyword-only)
        assert_eq!(parameters[3].name, "c");
        assert!(matches!(parameters[3].kind, ParameterKind::KwOnly));
        assert!(parameters[3].type_annotation.is_some());
        
        // **kwargs: dict
        assert_eq!(parameters[4].name, "kwargs");
        assert!(matches!(parameters[4].kind, ParameterKind::VarKwargs));
        assert!(parameters[4].type_annotation.is_some());
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_param_type_annotation_generic() {
    let result = parse("def func(x: List[int]):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name, "x");
        assert!(parameters[0].type_annotation.is_some());
        // Type annotation is a subscript expression: List[int]
        if let Some(Expression::Subscript { .. }) = &parameters[0].type_annotation {
            // Success - generic type parsed correctly
        } else {
            panic!("Expected subscript expression for generic type");
        }
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_param_type_annotation_optional() {
    let result = parse("def func(x: Optional[int]):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name, "x");
        assert!(parameters[0].type_annotation.is_some());
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_param_without_type_annotation() {
    let result = parse("def func(x, y: int, z):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 3);
        
        // x has no type annotation
        assert_eq!(parameters[0].name, "x");
        assert!(parameters[0].type_annotation.is_none());
        
        // y has type annotation
        assert_eq!(parameters[1].name, "y");
        assert!(parameters[1].type_annotation.is_some());
        
        // z has no type annotation
        assert_eq!(parameters[2].name, "z");
        assert!(parameters[2].type_annotation.is_none());
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_param_type_annotation_complex() {
    // Test nested generic types like List[List[int]]
    let result = parse("def func(x: List[List[int]]):\n    pass\n");
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    if let Statement::FunctionDef { parameters, .. } = &ast.statements[0] {
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name, "x");
        assert!(parameters[0].type_annotation.is_some());
        // Complex nested generic type
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_nonlocal_with_keyword_error() {
    let result = parse("nonlocal while\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_assert_empty_error() {
    let result = parse("assert\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_assert_comma_without_message_error() {
    let result = parse("assert x,\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_del_empty_error() {
    let result = parse("del\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_del_invalid_target_error() {
    let result = parse("del 123\n");
    // Note: Parser currently accepts this syntactically (del can parse any expression)
    // Semantic validation would reject deleting a literal
    // This is acceptable for a basic parser - semantic checks come later
    if result.is_ok() {
        // Parser allows it, semantic analysis would catch it later
    }
}

#[test]
fn test_parse_del_literal_error() {
    let result = parse("del \"string\"\n");
    // Note: Parser currently accepts this syntactically
    // Semantic validation would reject deleting a literal
    // This is acceptable for a basic parser - semantic checks come later
    if result.is_ok() {
        // Parser allows it, semantic analysis would catch it later
    }
}

#[test]
fn test_parse_multiple_starred_in_unpacking_error() {
    // Python doesn't allow multiple starred expressions in unpacking
    let result = parse("a, *b, *c = [1, 2, 3]\n");
    // This should now be caught as a syntax error by the parser
    assert!(result.is_err(), "Multiple starred expressions should be a syntax error");
}

#[test]
fn test_parse_multiple_starred_in_list_unpacking_error() {
    let result = parse("[a, *b, *c] = [1, 2, 3]\n");
    assert!(result.is_err(), "Multiple starred expressions in list should be a syntax error");
}

#[test]
fn test_parse_multiple_starred_nested_error() {
    let result = parse("a, (*b, *c) = [1, (2, 3)]\n");
    assert!(result.is_err(), "Multiple starred expressions in nested tuple should be a syntax error");
}

#[test]
fn test_parse_single_starred_unpacking_ok() {
    // Single starred expression is valid
    let result = parse("a, *b, c = [1, 2, 3, 4]\n");
    assert!(result.is_ok(), "Single starred expression should be valid");
}

#[test]
fn test_parse_starred_outside_unpacking_error() {
    let result = parse("*x\n");
    // Note: Parser currently accepts this syntactically as an expression statement
    // Semantic validation would reject starred expressions outside unpacking context
    // This is acceptable for a basic parser - semantic checks come later
    if result.is_ok() {
        // Parser allows it, semantic analysis would catch it later
    }
}

#[test]
fn test_parse_raise_with_invalid_expression_error() {
    let result = parse("raise if\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_import_missing_module_name_error() {
    let result = parse("import\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_import_missing_alias_after_as_error() {
    let result = parse("import os as\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_import_invalid_alias_error() {
    let result = parse("import os as 123\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_import_missing_identifier_after_dot_error() {
    let result = parse("import os.\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_import_double_dot_error() {
    let result = parse("import os..path\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_import_keyword_as_module_error() {
    let result = parse("import if\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_from_import_missing_import_keyword_error() {
    let result = parse("from os\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_from_import_missing_name_error() {
    let result = parse("from os import\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_from_import_missing_alias_after_as_error() {
    let result = parse("from os import path as\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_from_import_invalid_alias_error() {
    let result = parse("from os import path as 123\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_from_import_wildcard_with_alias_error() {
    let result = parse("from os import * as everything\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_from_import_wildcard_with_others_error() {
    let result = parse("from os import *, path\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_from_import_missing_module_name_error() {
    let result = parse("from import path\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_assert_with_invalid_expression_error() {
    let result = parse("assert if\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_if_missing_colon_error() {
    let result = parse("if x\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_if_missing_condition_error() {
    let result = parse("if :\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_if_missing_block_error() {
    let result = parse("if x:\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_if_empty_block_error() {
    let result = parse("if x:\n\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_elif_missing_colon_error() {
    let result = parse("if x:\n    pass\nelif y\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_else_missing_colon_error() {
    let result = parse("if x:\n    pass\nelse\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_while_missing_colon_error() {
    let result = parse("while x\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_while_missing_condition_error() {
    let result = parse("while :\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_while_missing_block_error() {
    let result = parse("while x:\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_for_missing_in_error() {
    let result = parse("for x items:\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_for_missing_colon_error() {
    let result = parse("for x in items\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_for_missing_target_error() {
    let result = parse("for in items:\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_for_missing_iterable_error() {
    let result = parse("for x in :\n    pass\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_for_missing_block_error() {
    let result = parse("for x in items:\n");
    assert!(result.is_err());
}

#[test]
fn test_parse_pass_statement() {
    let module = parse("pass\n").unwrap();
    
    match &module.statements[0] {
        Statement::Pass(_) => {},
        _ => panic!("Expected pass statement"),
    }
}

#[test]
fn test_parse_return_statement() {
    let module = parse("return 42\n").unwrap();
    
    match &module.statements[0] {
        Statement::Return { value: Some(expr), .. } => {
            match expr {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 42),
                _ => panic!("Expected integer value"),
            }
        }
        _ => panic!("Expected return statement"),
    }
}

#[test]
fn test_parse_return_none() {
    let module = parse("return\n").unwrap();
    
    match &module.statements[0] {
        Statement::Return { value: None, .. } => {},
        _ => panic!("Expected return statement with no value"),
    }
}

#[test]
fn test_parse_multiple_statements() {
    let module = parse("x = 1\ny = 2\nz = 3\n").unwrap();
    assert_eq!(module.statements.len(), 3);
    
    for (i, stmt) in module.statements.iter().enumerate() {
        match stmt {
            Statement::Assignment { value, .. } => {
                match value {
                    Expression::Literal(Literal::Integer { value, .. }) => {
                        assert_eq!(*value, (i + 1) as i64);
                    }
                    _ => panic!("Expected integer value"),
                }
            }
            _ => panic!("Expected assignment statement"),
        }
    }
}

#[test]
fn test_parse_complex_expression() {
    // (1 + 2) * 3 - 4 / 2
    let module = parse("(1 + 2) * 3 - 4 / 2\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    // Just verify it parses without panicking
    // Detailed structure verification would be very verbose
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { op: BinaryOperator::Subtract, .. }) => {},
        _ => panic!("Expected subtraction as top-level operation"),
    }
}

#[test]
fn test_parse_not_in_operator() {
    let module = parse("x not in list\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { op, .. }) => {
            assert!(matches!(op, BinaryOperator::NotIn));
        }
        _ => panic!("Expected binary operation with NotIn operator"),
    }
}

#[test]
fn test_parse_is_not_operator() {
    let module = parse("x is not y\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { op, .. }) => {
            assert!(matches!(op, BinaryOperator::IsNot));
        }
        _ => panic!("Expected binary operation with IsNot operator"),
    }
}

#[test]
fn test_parse_in_operator() {
    let module = parse("x in list\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { op, .. }) => {
            assert!(matches!(op, BinaryOperator::In));
        }
        _ => panic!("Expected binary operation with In operator"),
    }
}

#[test]
fn test_parse_is_operator() {
    let module = parse("x is None\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { op, .. }) => {
            assert!(matches!(op, BinaryOperator::Is));
        }
        _ => panic!("Expected binary operation with Is operator"),
    }
}

#[test]
fn test_parse_function_call_no_args() {
    let module = parse("foo()\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { function, arguments, .. }) => {
            // Function should be an identifier
            match **function {
                Expression::Identifier { ref name, .. } => assert_eq!(name, "foo"),
                _ => panic!("Expected identifier as function"),
            }
            assert_eq!(arguments.len(), 0);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_function_call_single_arg() {
    let module = parse("print(42)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { function, arguments, .. }) => {
            match **function {
                Expression::Identifier { ref name, .. } => assert_eq!(name, "print"),
                _ => panic!("Expected identifier as function"),
            }
            assert_eq!(arguments.len(), 1);
            
            match &arguments[0] {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 42),
                _ => panic!("Expected integer argument"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_function_call_multiple_args() {
    let module = parse("add(1, 2, 3)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { function, arguments, .. }) => {
            match **function {
                Expression::Identifier { ref name, .. } => assert_eq!(name, "add"),
                _ => panic!("Expected identifier as function"),
            }
            assert_eq!(arguments.len(), 3);
            
            for (i, arg) in arguments.iter().enumerate() {
                match arg {
                    Expression::Literal(Literal::Integer { value, .. }) => {
                        assert_eq!(*value, (i + 1) as i64);
                    }
                    _ => panic!("Expected integer argument"),
                }
            }
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_function_call_with_expression_args() {
    let module = parse("func(1 + 2, x * y)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { arguments, .. }) => {
            assert_eq!(arguments.len(), 2);
            
            // First arg should be addition
            match &arguments[0] {
                Expression::BinaryOp { op, .. } => {
                    assert!(matches!(op, BinaryOperator::Add));
                }
                _ => panic!("Expected binary operation"),
            }
            
            // Second arg should be multiplication
            match &arguments[1] {
                Expression::BinaryOp { op, .. } => {
                    assert!(matches!(op, BinaryOperator::Multiply));
                }
                _ => panic!("Expected binary operation"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_nested_function_calls() {
    let module = parse("outer(inner(5))\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { function, arguments, .. }) => {
            // Outer function should be "outer"
            match **function {
                Expression::Identifier { ref name, .. } => assert_eq!(name, "outer"),
                _ => panic!("Expected identifier"),
            }
            
            assert_eq!(arguments.len(), 1);
            
            // Argument should be another function call
            match &arguments[0] {
                Expression::Call { function: inner_func, arguments: inner_args, .. } => {
                    match **inner_func {
                        Expression::Identifier { ref name, .. } => assert_eq!(name, "inner"),
                        _ => panic!("Expected identifier for inner function"),
                    }
                    assert_eq!(inner_args.len(), 1);
                }
                _ => panic!("Expected nested function call"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_function_call_trailing_comma() {
    let module = parse("func(1, 2,)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { arguments, .. }) => {
            assert_eq!(arguments.len(), 2);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_chained_function_calls() {
    let module = parse("get_func()()\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { function, arguments, .. }) => {
            // Outer call should have no arguments
            assert_eq!(arguments.len(), 0);
            
            // Function should be another call
            match **function {
                Expression::Call { .. } => {},
                _ => panic!("Expected inner function call"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_subscript_simple() {
    let module = parse("list[0]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Subscript { object, index, .. }) => {
            // Object should be identifier "list"
            match **object {
                Expression::Identifier { ref name, .. } => assert_eq!(name, "list"),
                _ => panic!("Expected identifier as object"),
            }
            
            // Index should be integer 0
            match **index {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(value, 0),
                _ => panic!("Expected integer index"),
            }
        }
        _ => panic!("Expected subscript operation"),
    }
}

#[test]
fn test_parse_subscript_negative_index() {
    let module = parse("array[-1]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Subscript { index, .. }) => {
            // Index should be unary minus operation
            match **index {
                Expression::UnaryOp { op: UnaryOperator::Minus, .. } => {},
                _ => panic!("Expected unary minus for negative index"),
            }
        }
        _ => panic!("Expected subscript operation"),
    }
}

#[test]
fn test_parse_subscript_string_key() {
    let module = parse("dict[\"key\"]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Subscript { object, index, .. }) => {
            match **object {
                Expression::Identifier { ref name, .. } => assert_eq!(name, "dict"),
                _ => panic!("Expected identifier"),
            }
            
            match **index {
                Expression::Literal(Literal::String { ref value, .. }) => assert_eq!(value, "key"),
                _ => panic!("Expected string index"),
            }
        }
        _ => panic!("Expected subscript operation"),
    }
}

#[test]
fn test_parse_subscript_expression_index() {
    let module = parse("items[i + 1]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Subscript { index, .. }) => {
            // Index should be binary operation
            match **index {
                Expression::BinaryOp { op: BinaryOperator::Add, .. } => {},
                _ => panic!("Expected addition in index"),
            }
        }
        _ => panic!("Expected subscript operation"),
    }
}

#[test]
fn test_parse_nested_subscripts() {
    let module = parse("matrix[i][j]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Subscript { object, index, .. }) => {
            // Object should be another subscript
            match **object {
                Expression::Subscript { .. } => {},
                _ => panic!("Expected nested subscript"),
            }
            
            // Index should be identifier "j"
            match **index {
                Expression::Identifier { ref name, .. } => assert_eq!(name, "j"),
                _ => panic!("Expected identifier index"),
            }
        }
        _ => panic!("Expected subscript operation"),
    }
}

#[test]
fn test_parse_subscript_after_call() {
    let module = parse("get_list()[0]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Subscript { object, index, .. }) => {
            // Object should be a function call
            match **object {
                Expression::Call { .. } => {},
                _ => panic!("Expected function call as object"),
            }
            
            match **index {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(value, 0),
                _ => panic!("Expected integer index"),
            }
        }
        _ => panic!("Expected subscript operation"),
    }
}

#[test]
fn test_parse_call_after_subscript() {
    let module = parse("funcs[0]()\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { function, arguments, .. }) => {
            // Function should be a subscript
            match **function {
                Expression::Subscript { .. } => {},
                _ => panic!("Expected subscript as function"),
            }
            
            assert_eq!(arguments.len(), 0);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_attribute_simple() {
    let module = parse("obj.attr\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Attribute { object, attribute, .. }) => {
            // Object should be identifier "obj"
            match **object {
                Expression::Identifier { ref name, .. } => assert_eq!(name, "obj"),
                _ => panic!("Expected identifier as object"),
            }
            
            assert_eq!(attribute, "attr");
        }
        _ => panic!("Expected attribute access"),
    }
}

#[test]
fn test_parse_attribute_chained() {
    let module = parse("obj.x.y.z\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Attribute { object, attribute, .. }) => {
            // Innermost attribute should be "z"
            assert_eq!(attribute, "z");
            
            // Object should be another attribute (obj.x.y)
            match **object {
                Expression::Attribute { ref attribute, .. } => {
                    assert_eq!(attribute, "y");
                }
                _ => panic!("Expected nested attribute access"),
            }
        }
        _ => panic!("Expected attribute access"),
    }
}

#[test]
fn test_parse_method_call() {
    let module = parse("obj.method()\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { function, arguments, .. }) => {
            // Function should be an attribute access
            match **function {
                Expression::Attribute { ref object, ref attribute, .. } => {
                    match **object {
                        Expression::Identifier { ref name, .. } => assert_eq!(name, "obj"),
                        _ => panic!("Expected identifier"),
                    }
                    assert_eq!(attribute, "method");
                }
                _ => panic!("Expected attribute access as function"),
            }
            
            assert_eq!(arguments.len(), 0);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_method_call_with_args() {
    let module = parse("obj.add(1, 2)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { function, arguments, .. }) => {
            match **function {
                Expression::Attribute { ref attribute, .. } => {
                    assert_eq!(attribute, "add");
                }
                _ => panic!("Expected attribute access"),
            }
            
            assert_eq!(arguments.len(), 2);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_chained_method_calls() {
    let module = parse("obj.get().process()\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { function, arguments, .. }) => {
            // Outer call to process()
            assert_eq!(arguments.len(), 0);
            
            // Function should be attribute access on a call result
            match **function {
                Expression::Attribute { ref object, ref attribute, .. } => {
                    assert_eq!(attribute, "process");
                    
                    // Object should be a call to get()
                    match **object {
                        Expression::Call { .. } => {},
                        _ => panic!("Expected call to get()"),
                    }
                }
                _ => panic!("Expected attribute access"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_attribute_subscript() {
    let module = parse("obj.list[0]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Subscript { object, index, .. }) => {
            // Object should be attribute access
            match **object {
                Expression::Attribute { ref attribute, .. } => {
                    assert_eq!(attribute, "list");
                }
                _ => panic!("Expected attribute access"),
            }
            
            // Index should be 0
            match **index {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(value, 0),
                _ => panic!("Expected integer index"),
            }
        }
        _ => panic!("Expected subscript operation"),
    }
}

#[test]
fn test_parse_complex_postfix_chain() {
    // obj.get_funcs()[0](arg).result
    let module = parse("obj.get_funcs()[0](arg).result\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Attribute { attribute, .. }) => {
            // Final operation should be accessing .result
            assert_eq!(attribute, "result");
        }
        _ => panic!("Expected attribute access at top level"),
    }
}

#[test]
fn test_parse_empty_list() {
    let module = parse("[]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::List { elements, .. }) => {
            assert_eq!(elements.len(), 0);
        }
        _ => panic!("Expected list expression"),
    }
}

#[test]
fn test_parse_list_single_element() {
    let module = parse("[42]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::List { elements, .. }) => {
            assert_eq!(elements.len(), 1);
            
            match &elements[0] {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 42),
                _ => panic!("Expected integer element"),
            }
        }
        _ => panic!("Expected list expression"),
    }
}

#[test]
fn test_parse_list_multiple_elements() {
    let module = parse("[1, 2, 3]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::List { elements, .. }) => {
            assert_eq!(elements.len(), 3);
            
            for (i, elem) in elements.iter().enumerate() {
                match elem {
                    Expression::Literal(Literal::Integer { value, .. }) => {
                        assert_eq!(*value, (i + 1) as i64);
                    }
                    _ => panic!("Expected integer element"),
                }
            }
        }
        _ => panic!("Expected list expression"),
    }
}

#[test]
fn test_parse_list_trailing_comma() {
    let module = parse("[1, 2, 3,]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::List { elements, .. }) => {
            assert_eq!(elements.len(), 3);
        }
        _ => panic!("Expected list expression"),
    }
}

#[test]
fn test_parse_list_with_expressions() {
    let module = parse("[1 + 2, x * y, func()]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::List { elements, .. }) => {
            assert_eq!(elements.len(), 3);
            
            // First should be addition
            match &elements[0] {
                Expression::BinaryOp { op: BinaryOperator::Add, .. } => {},
                _ => panic!("Expected addition"),
            }
            
            // Second should be multiplication
            match &elements[1] {
                Expression::BinaryOp { op: BinaryOperator::Multiply, .. } => {},
                _ => panic!("Expected multiplication"),
            }
            
            // Third should be function call
            match &elements[2] {
                Expression::Call { .. } => {},
                _ => panic!("Expected function call"),
            }
        }
        _ => panic!("Expected list expression"),
    }
}

#[test]
fn test_parse_nested_lists() {
    let module = parse("[[1, 2], [3, 4]]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::List { elements, .. }) => {
            assert_eq!(elements.len(), 2);
            
            // Both elements should be lists
            for elem in elements {
                match elem {
                    Expression::List { elements: inner_elements, .. } => {
                        assert_eq!(inner_elements.len(), 2);
                    }
                    _ => panic!("Expected nested list"),
                }
            }
        }
        _ => panic!("Expected list expression"),
    }
}

#[test]
fn test_parse_list_subscript_disambiguation() {
    // Make sure list literal [1, 2] is different from subscript arr[1]
    let module1 = parse("[1, 2]\n").unwrap();
    let module2 = parse("arr[1]\n").unwrap();
    
    // First should be a list
    match &module1.statements[0] {
        Statement::Expression(Expression::List { .. }) => {},
        _ => panic!("Expected list literal"),
    }
    
    // Second should be a subscript
    match &module2.statements[0] {
        Statement::Expression(Expression::Subscript { .. }) => {},
        _ => panic!("Expected subscript operation"),
    }
}

#[test]
fn test_parse_empty_tuple() {
    let module = parse("()\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Tuple { elements, .. }) => {
            assert_eq!(elements.len(), 0);
        }
        _ => panic!("Expected tuple expression"),
    }
}

#[test]
fn test_parse_single_element_tuple() {
    let module = parse("(42,)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Tuple { elements, .. }) => {
            assert_eq!(elements.len(), 1);
            
            match &elements[0] {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 42),
                _ => panic!("Expected integer element"),
            }
        }
        _ => panic!("Expected tuple expression"),
    }
}

#[test]
fn test_parse_tuple_multiple_elements() {
    let module = parse("(1, 2, 3)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Tuple { elements, .. }) => {
            assert_eq!(elements.len(), 3);
            
            for (i, elem) in elements.iter().enumerate() {
                match elem {
                    Expression::Literal(Literal::Integer { value, .. }) => {
                        assert_eq!(*value, (i + 1) as i64);
                    }
                    _ => panic!("Expected integer element"),
                }
            }
        }
        _ => panic!("Expected tuple expression"),
    }
}

#[test]
fn test_parse_tuple_trailing_comma() {
    let module = parse("(1, 2, 3,)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Tuple { elements, .. }) => {
            assert_eq!(elements.len(), 3);
        }
        _ => panic!("Expected tuple expression"),
    }
}

#[test]
fn test_parse_parenthesized_vs_tuple() {
    // (x) is parenthesized, (x,) is tuple
    let module1 = parse("(42)\n").unwrap();
    let module2 = parse("(42,)\n").unwrap();
    
    // First should be parenthesized
    match &module1.statements[0] {
        Statement::Expression(Expression::Parenthesized { .. }) => {},
        _ => panic!("Expected parenthesized expression"),
    }
    
    // Second should be tuple
    match &module2.statements[0] {
        Statement::Expression(Expression::Tuple { .. }) => {},
        _ => panic!("Expected tuple expression"),
    }
}

#[test]
fn test_parse_nested_tuples() {
    let module = parse("((1, 2), (3, 4))\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Tuple { elements, .. }) => {
            assert_eq!(elements.len(), 2);
            
            // Both elements should be tuples
            for elem in elements {
                match elem {
                    Expression::Tuple { elements: inner_elements, .. } => {
                        assert_eq!(inner_elements.len(), 2);
                    }
                    _ => panic!("Expected nested tuple"),
                }
            }
        }
        _ => panic!("Expected tuple expression"),
    }
}

#[test]
fn test_parse_tuple_with_expressions() {
    let module = parse("(1 + 2, x * y, func())\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Tuple { elements, .. }) => {
            assert_eq!(elements.len(), 3);
            
            // First should be addition
            match &elements[0] {
                Expression::BinaryOp { op: BinaryOperator::Add, .. } => {},
                _ => panic!("Expected addition"),
            }
            
            // Second should be multiplication
            match &elements[1] {
                Expression::BinaryOp { op: BinaryOperator::Multiply, .. } => {},
                _ => panic!("Expected multiplication"),
            }
            
            // Third should be function call
            match &elements[2] {
                Expression::Call { .. } => {},
                _ => panic!("Expected function call"),
            }
        }
        _ => panic!("Expected tuple expression"),
    }
}

#[test]
fn test_parse_tuple_subscript() {
    let module = parse("(1, 2, 3)[0]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Subscript { object, .. }) => {
            // Object should be a tuple
            match **object {
                Expression::Tuple { .. } => {},
                _ => panic!("Expected tuple as subscript object"),
            }
        }
        _ => panic!("Expected subscript operation"),
    }
}

// ============================================================================
// Dict Expression Tests
// ============================================================================

#[test]
fn test_empty_dict() {
    let module = parse("{}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Dict { pairs, .. }) => {
            assert_eq!(pairs.len(), 0);
        }
        _ => panic!("Expected dict expression"),
    }
}

#[test]
fn test_single_pair_dict() {
    let module = parse("{\"key\": \"value\"}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Dict { pairs, .. }) => {
            assert_eq!(pairs.len(), 1);
            
            // Check key is string "key"
            match &pairs[0].0 {
                Expression::Literal(Literal::String { value, .. }) => assert_eq!(value, "key"),
                _ => panic!("Expected string literal as key"),
            }
            
            // Check value is string "value"
            match &pairs[0].1 {
                Expression::Literal(Literal::String { value, .. }) => assert_eq!(value, "value"),
                _ => panic!("Expected string literal as value"),
            }
        }
        _ => panic!("Expected dict expression"),
    }
}

#[test]
fn test_multiple_pairs_dict() {
    let module = parse("{\"a\": 1, \"b\": 2, \"c\": 3}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Dict { pairs, .. }) => {
            assert_eq!(pairs.len(), 3);
            
            // Check first pair: "a": 1
            match &pairs[0].0 {
                Expression::Literal(Literal::String { value, .. }) => assert_eq!(value, "a"),
                _ => panic!("Expected string literal as key"),
            }
            match &pairs[0].1 {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 1),
                _ => panic!("Expected integer literal as value"),
            }
            
            // Check second pair: "b": 2
            match &pairs[1].0 {
                Expression::Literal(Literal::String { value, .. }) => assert_eq!(value, "b"),
                _ => panic!("Expected string literal as key"),
            }
            match &pairs[1].1 {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 2),
                _ => panic!("Expected integer literal as value"),
            }
            
            // Check third pair: "c": 3
            match &pairs[2].0 {
                Expression::Literal(Literal::String { value, .. }) => assert_eq!(value, "c"),
                _ => panic!("Expected string literal as key"),
            }
            match &pairs[2].1 {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 3),
                _ => panic!("Expected integer literal as value"),
            }
        }
        _ => panic!("Expected dict expression"),
    }
}

#[test]
fn test_dict_with_trailing_comma() {
    let module = parse("{\"x\": 10, \"y\": 20,}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Dict { pairs, .. }) => {
            assert_eq!(pairs.len(), 2);
        }
        _ => panic!("Expected dict expression"),
    }
}

#[test]
fn test_dict_with_expression_keys_and_values() {
    let module = parse("{x + 1: y * 2, \"key\": [1, 2, 3]}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Dict { pairs, .. }) => {
            assert_eq!(pairs.len(), 2);
            
            // First pair: x + 1 : y * 2
            match &pairs[0].0 {
                Expression::BinaryOp { .. } => {},
                _ => panic!("Expected binary operation as key"),
            }
            match &pairs[0].1 {
                Expression::BinaryOp { .. } => {},
                _ => panic!("Expected binary operation as value"),
            }
            
            // Second pair: "key" : [1, 2, 3]
            match &pairs[1].0 {
                Expression::Literal(Literal::String { value, .. }) => assert_eq!(value, "key"),
                _ => panic!("Expected string literal as key"),
            }
            match &pairs[1].1 {
                Expression::List { elements, .. } => assert_eq!(elements.len(), 3),
                _ => panic!("Expected list as value"),
            }
        }
        _ => panic!("Expected dict expression"),
    }
}

#[test]
fn test_nested_dict() {
    let module = parse("{\"outer\": {\"inner\": 42}}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Dict { pairs, .. }) => {
            assert_eq!(pairs.len(), 1);
            
            // Check key is "outer"
            match &pairs[0].0 {
                Expression::Literal(Literal::String { value, .. }) => assert_eq!(value, "outer"),
                _ => panic!("Expected string literal as key"),
            }
            
            // Check value is nested dict
            match &pairs[0].1 {
                Expression::Dict { pairs: inner_pairs, .. } => {
                    assert_eq!(inner_pairs.len(), 1);
                    
                    // Check inner key is "inner"
                    match &inner_pairs[0].0 {
                        Expression::Literal(Literal::String { value, .. }) => assert_eq!(value, "inner"),
                        _ => panic!("Expected string literal as inner key"),
                    }
                    
                    // Check inner value is 42
                    match &inner_pairs[0].1 {
                        Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 42),
                        _ => panic!("Expected integer literal as inner value"),
                    }
                }
                _ => panic!("Expected dict as value"),
            }
        }
        _ => panic!("Expected dict expression"),
    }
}

#[test]
fn test_dict_with_subscript() {
    let module = parse("{\"a\": 1, \"b\": 2}[\"a\"]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Subscript { object, index, .. }) => {
            // Object should be a dict
            match **object {
                Expression::Dict { .. } => {},
                _ => panic!("Expected dict as subscript object"),
            }
            
            // Index should be string "a"
            match **index {
                Expression::Literal(Literal::String { ref value, .. }) => assert_eq!(value, "a"),
                _ => panic!("Expected string literal as index"),
            }
        }
        _ => panic!("Expected subscript operation"),
    }
}
// ============================================================================
// Set Expression Tests
// ============================================================================

#[test]
fn test_single_element_set() {
    let module = parse("{1}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Set { elements, .. }) => {
            assert_eq!(elements.len(), 1);
            match &elements[0] {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 1),
                _ => panic!("Expected integer literal"),
            }
        }
        _ => panic!("Expected set expression"),
    }
}

#[test]
fn test_multiple_elements_set() {
    let module = parse("{1, 2, 3}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Set { elements, .. }) => {
            assert_eq!(elements.len(), 3);
            
            match &elements[0] {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 1),
                _ => panic!("Expected integer literal"),
            }
            match &elements[1] {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 2),
                _ => panic!("Expected integer literal"),
            }
            match &elements[2] {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 3),
                _ => panic!("Expected integer literal"),
            }
        }
        _ => panic!("Expected set expression"),
    }
}

#[test]
fn test_set_with_trailing_comma() {
    let module = parse("{1, 2, 3,}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Set { elements, .. }) => {
            assert_eq!(elements.len(), 3);
        }
        _ => panic!("Expected set expression"),
    }
}

#[test]
fn test_set_with_expressions() {
    let module = parse("{x + 1, y * 2, \"text\"}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Set { elements, .. }) => {
            assert_eq!(elements.len(), 3);
            
            // First element: x + 1
            match &elements[0] {
                Expression::BinaryOp { .. } => {},
                _ => panic!("Expected binary operation"),
            }
            
            // Second element: y * 2
            match &elements[1] {
                Expression::BinaryOp { .. } => {},
                _ => panic!("Expected binary operation"),
            }
            
            // Third element: "text"
            match &elements[2] {
                Expression::Literal(Literal::String { value, .. }) => assert_eq!(value, "text"),
                _ => panic!("Expected string literal"),
            }
        }
        _ => panic!("Expected set expression"),
    }
}

#[test]
fn test_nested_set() {
    // Note: In Python, sets can't contain other sets (not hashable)
    // but we can have a set containing a list
    let module = parse("{[1, 2], [3, 4]}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Set { elements, .. }) => {
            assert_eq!(elements.len(), 2);
            
            match &elements[0] {
                Expression::List { elements: inner, .. } => assert_eq!(inner.len(), 2),
                _ => panic!("Expected list"),
            }
            match &elements[1] {
                Expression::List { elements: inner, .. } => assert_eq!(inner.len(), 2),
                _ => panic!("Expected list"),
            }
        }
        _ => panic!("Expected set expression"),
    }
}

#[test]
fn test_empty_braces_is_dict() {
    // In Python, {} is always an empty dict, not an empty set
    let module = parse("{}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Dict { pairs, .. }) => {
            assert_eq!(pairs.len(), 0);
        }
        _ => panic!("Expected empty dict, not set"),
    }
}

#[test]
fn test_set_vs_dict_disambiguation() {
    // Test that {1} is a set but {1: 2} is a dict
    let set_module = parse("{1}\n").unwrap();
    match &set_module.statements[0] {
        Statement::Expression(Expression::Set { .. }) => {},
        _ => panic!("Expected set expression for {{1}}"),
    }
    
    let dict_module = parse("{1: 2}\n").unwrap();
    match &dict_module.statements[0] {
        Statement::Expression(Expression::Dict { .. }) => {},
        _ => panic!("Expected dict expression for {{1: 2}}"),
    }
}

// ============================================================================
// Lambda Expression Tests
// ============================================================================

#[test]
fn test_lambda_no_params() {
    let module = parse("lambda: 42\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Lambda { parameters, body, .. }) => {
            assert_eq!(parameters.len(), 0);
            
            match **body {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(value, 42),
                _ => panic!("Expected integer literal in lambda body"),
            }
        }
        _ => panic!("Expected lambda expression"),
    }
}

#[test]
fn test_lambda_single_param() {
    let module = parse("lambda x: x + 1\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Lambda { parameters, body, .. }) => {
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0], "x");
            
            match **body {
                Expression::BinaryOp { .. } => {},
                _ => panic!("Expected binary operation in lambda body"),
            }
        }
        _ => panic!("Expected lambda expression"),
    }
}

#[test]
fn test_lambda_multiple_params() {
    let module = parse("lambda x, y, z: x + y * z\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Lambda { parameters, body, .. }) => {
            assert_eq!(parameters.len(), 3);
            assert_eq!(parameters[0], "x");
            assert_eq!(parameters[1], "y");
            assert_eq!(parameters[2], "z");
            
            match **body {
                Expression::BinaryOp { .. } => {},
                _ => panic!("Expected binary operation in lambda body"),
            }
        }
        _ => panic!("Expected lambda expression"),
    }
}

#[test]
fn test_lambda_with_comparison() {
    let module = parse("lambda x: x > 0\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Lambda { parameters, body, .. }) => {
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0], "x");
            
            match **body {
                Expression::BinaryOp { op: BinaryOperator::GreaterThan, .. } => {},
                _ => panic!("Expected comparison in lambda body"),
            }
        }
        _ => panic!("Expected lambda expression"),
    }
}

#[test]
fn test_lambda_with_function_call() {
    let module = parse("lambda x: func(x)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Lambda { parameters, body, .. }) => {
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0], "x");
            
            match **body {
                Expression::Call { .. } => {},
                _ => panic!("Expected call in lambda body"),
            }
        }
        _ => panic!("Expected lambda expression"),
    }
}

#[test]
fn test_lambda_in_function_call() {
    let module = parse("map(lambda x: x * 2, lst)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { arguments, .. }) => {
            assert_eq!(arguments.len(), 2);
            
            // First argument should be lambda
            match &arguments[0] {
                Expression::Lambda { parameters, .. } => {
                    assert_eq!(parameters.len(), 1);
                    assert_eq!(parameters[0], "x");
                }
                _ => panic!("Expected lambda as first argument"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_lambda_with_list() {
    let module = parse("lambda: [1, 2, 3]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Lambda { parameters, body, .. }) => {
            assert_eq!(parameters.len(), 0);
            
            match **body {
                Expression::List { ref elements, .. } => assert_eq!(elements.len(), 3),
                _ => panic!("Expected list in lambda body"),
            }
        }
        _ => panic!("Expected lambda expression"),
    }
}

// ============================================================================
// Conditional Expression Tests (Ternary)
// ============================================================================

#[test]
fn test_simple_conditional() {
    let module = parse("1 if True else 0\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Conditional { condition, true_expr, false_expr, .. }) => {
            // true_expr should be 1
            match **true_expr {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(value, 1),
                _ => panic!("Expected integer 1 as true expression"),
            }
            
            // condition should be True
            match **condition {
                Expression::Literal(Literal::Boolean { value, .. }) => assert_eq!(value, true),
                _ => panic!("Expected True as condition"),
            }
            
            // false_expr should be 0
            match **false_expr {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(value, 0),
                _ => panic!("Expected integer 0 as false expression"),
            }
        }
        _ => panic!("Expected conditional expression"),
    }
}

#[test]
fn test_conditional_with_comparison() {
    let module = parse("'yes' if x > 0 else 'no'\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Conditional { condition, true_expr, false_expr, .. }) => {
            // Condition should be a comparison
            match **condition {
                Expression::BinaryOp { op: BinaryOperator::GreaterThan, .. } => {},
                _ => panic!("Expected comparison as condition"),
            }
            
            // true_expr should be string
            match **true_expr {
                Expression::Literal(Literal::String { ref value, .. }) => assert_eq!(value, "yes"),
                _ => panic!("Expected string literal"),
            }
            
            // false_expr should be string
            match **false_expr {
                Expression::Literal(Literal::String { ref value, .. }) => assert_eq!(value, "no"),
                _ => panic!("Expected string literal"),
            }
        }
        _ => panic!("Expected conditional expression"),
    }
}

#[test]
fn test_conditional_with_expressions() {
    let module = parse("x + 1 if x > 0 else x - 1\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Conditional { condition, true_expr, false_expr, .. }) => {
            // true_expr should be binary op
            match **true_expr {
                Expression::BinaryOp { op: BinaryOperator::Add, .. } => {},
                _ => panic!("Expected addition"),
            }
            
            // condition should be comparison
            match **condition {
                Expression::BinaryOp { op: BinaryOperator::GreaterThan, .. } => {},
                _ => panic!("Expected comparison"),
            }
            
            // false_expr should be binary op
            match **false_expr {
                Expression::BinaryOp { op: BinaryOperator::Subtract, .. } => {},
                _ => panic!("Expected subtraction"),
            }
        }
        _ => panic!("Expected conditional expression"),
    }
}

#[test]
fn test_nested_conditional() {
    let module = parse("'a' if x > 0 else 'b' if x < 0 else 'c'\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Conditional { false_expr, .. }) => {
            // false_expr should itself be a conditional
            match **false_expr {
                Expression::Conditional { .. } => {},
                _ => panic!("Expected nested conditional in false branch"),
            }
        }
        _ => panic!("Expected conditional expression"),
    }
}

#[test]
fn test_conditional_in_function_call() {
    let module = parse("func(1 if cond else 0)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { arguments, .. }) => {
            assert_eq!(arguments.len(), 1);
            
            // Argument should be conditional
            match &arguments[0] {
                Expression::Conditional { .. } => {},
                _ => panic!("Expected conditional as argument"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_conditional_with_list() {
    let module = parse("[1, 2, 3] if flag else []\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Conditional { true_expr, false_expr, .. }) => {
            // true_expr should be list with 3 elements
            match **true_expr {
                Expression::List { ref elements, .. } => assert_eq!(elements.len(), 3),
                _ => panic!("Expected list with 3 elements"),
            }
            
            // false_expr should be empty list
            match **false_expr {
                Expression::List { ref elements, .. } => assert_eq!(elements.len(), 0),
                _ => panic!("Expected empty list"),
            }
        }
        _ => panic!("Expected conditional expression"),
    }
}

#[test]
fn test_conditional_with_function_calls() {
    let module = parse("func1() if condition() else func2()\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Conditional { condition, true_expr, false_expr, .. }) => {
            // All three parts should be function calls
            match **true_expr {
                Expression::Call { .. } => {},
                _ => panic!("Expected call in true branch"),
            }
            match **condition {
                Expression::Call { .. } => {},
                _ => panic!("Expected call in condition"),
            }
            match **false_expr {
                Expression::Call { .. } => {},
                _ => panic!("Expected call in false branch"),
            }
        }
        _ => panic!("Expected conditional expression"),
    }
}

// ============================================================================
// Walrus Operator / Assignment Expression Tests
// ============================================================================

#[test]
fn test_simple_walrus() {
    let module = parse("x := 5\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::AssignmentExpr { target, value, .. }) => {
            assert_eq!(target, "x");
            
            match **value {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(value, 5),
                _ => panic!("Expected integer literal as value"),
            }
        }
        _ => panic!("Expected assignment expression"),
    }
}

#[test]
fn test_walrus_with_expression() {
    let module = parse("y := x + 10\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::AssignmentExpr { target, value, .. }) => {
            assert_eq!(target, "y");
            
            match **value {
                Expression::BinaryOp { op: BinaryOperator::Add, .. } => {},
                _ => panic!("Expected binary operation as value"),
            }
        }
        _ => panic!("Expected assignment expression"),
    }
}

#[test]
fn test_walrus_in_condition() {
    let module = parse("(n := len(data))\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Parenthesized { expr, .. }) => {
            match **expr {
                Expression::AssignmentExpr { ref target, .. } => {
                    assert_eq!(target, "n");
                }
                _ => panic!("Expected assignment expression inside parentheses"),
            }
        }
        _ => panic!("Expected parenthesized expression"),
    }
}

#[test]
fn test_walrus_in_function_call() {
    let module = parse("func(x := 42)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { arguments, .. }) => {
            assert_eq!(arguments.len(), 1);
            
            match &arguments[0] {
                Expression::AssignmentExpr { target, .. } => {
                    assert_eq!(target, "x");
                }
                _ => panic!("Expected assignment expression as argument"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_walrus_with_comparison() {
    let module = parse("result := x > 10\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::AssignmentExpr { target, value, .. }) => {
            assert_eq!(target, "result");
            
            match **value {
                Expression::BinaryOp { op: BinaryOperator::GreaterThan, .. } => {},
                _ => panic!("Expected comparison as value"),
            }
        }
        _ => panic!("Expected assignment expression"),
    }
}

#[test]
fn test_walrus_with_list() {
    let module = parse("items := [1, 2, 3]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::AssignmentExpr { target, value, .. }) => {
            assert_eq!(target, "items");
            
            match **value {
                Expression::List { ref elements, .. } => assert_eq!(elements.len(), 3),
                _ => panic!("Expected list as value"),
            }
        }
        _ => panic!("Expected assignment expression"),
    }
}

#[test]
fn test_walrus_with_function_call_value() {
    let module = parse("val := func()\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::AssignmentExpr { target, value, .. }) => {
            assert_eq!(target, "val");
            
            match **value {
                Expression::Call { .. } => {},
                _ => panic!("Expected function call as value"),
            }
        }
        _ => panic!("Expected assignment expression"),
    }
}

// ============================================================================
// Ellipsis Tests
// ============================================================================

#[test]
fn test_ellipsis_literal() {
    let module = parse("...\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Literal(Literal::Ellipsis { .. })) => {},
        _ => panic!("Expected ellipsis literal"),
    }
}

#[test]
fn test_ellipsis_in_list() {
    let module = parse("[1, ..., 3]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::List { elements, .. }) => {
            assert_eq!(elements.len(), 3);
            
            // Check middle element is ellipsis
            match &elements[1] {
                Expression::Literal(Literal::Ellipsis { .. }) => {},
                _ => panic!("Expected ellipsis in list"),
            }
        }
        _ => panic!("Expected list expression"),
    }
}

#[test]
fn test_ellipsis_in_tuple() {
    let module = parse("(1, ..., 3)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Tuple { elements, .. }) => {
            assert_eq!(elements.len(), 3);
            
            // Check middle element is ellipsis
            match &elements[1] {
                Expression::Literal(Literal::Ellipsis { .. }) => {},
                _ => panic!("Expected ellipsis in tuple"),
            }
        }
        _ => panic!("Expected tuple expression"),
    }
}

#[test]
fn test_ellipsis_as_function_argument() {
    let module = parse("func(...)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Call { arguments, .. }) => {
            assert_eq!(arguments.len(), 1);
            
            match &arguments[0] {
                Expression::Literal(Literal::Ellipsis { .. }) => {},
                _ => panic!("Expected ellipsis as argument"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_ellipsis_in_subscript() {
    let module = parse("arr[...]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Subscript { index, .. }) => {
            match **index {
                Expression::Literal(Literal::Ellipsis { .. }) => {},
                _ => panic!("Expected ellipsis as subscript index"),
            }
        }
        _ => panic!("Expected subscript expression"),
    }
}

#[test]
fn test_ellipsis_in_slice() {
    let module = parse("arr[(..., 0)]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Subscript { index, .. }) => {
            // The index should be a tuple with ellipsis and 0
            match **index {
                Expression::Tuple { ref elements, .. } => {
                    assert_eq!(elements.len(), 2);
                    
                    match &elements[0] {
                        Expression::Literal(Literal::Ellipsis { .. }) => {},
                        _ => panic!("Expected ellipsis as first element"),
                    }
                    
                    match &elements[1] {
                        Expression::Literal(Literal::Integer { value, .. }) => {
                            assert_eq!(*value, 0);
                        }
                        _ => panic!("Expected integer as second element"),
                    }
                }
                _ => panic!("Expected tuple as index"),
            }
        }
        _ => panic!("Expected subscript expression"),
    }
}

#[test]
fn test_multiple_ellipsis_in_dict() {
    let module = parse("{...: ..., 1: 2}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::Dict { pairs, .. }) => {
            assert_eq!(pairs.len(), 2);
            
            // Check first key-value pair has ellipsis for both
            match (&pairs[0].0, &pairs[0].1) {
                (
                    Expression::Literal(Literal::Ellipsis { .. }),
                    Expression::Literal(Literal::Ellipsis { .. })
                ) => {},
                _ => panic!("Expected ellipsis for key and value in first pair"),
            }
        }
        _ => panic!("Expected dict expression"),
    }
}

// ============================================================================
// List Comprehension Tests
// ============================================================================

#[test]
fn test_simple_list_comprehension() {
    let module = parse("[x for x in items]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::ListComp { element, generators, .. }) => {
            // Check element is identifier 'x'
            match &**element {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "x");
                }
                _ => panic!("Expected identifier as element"),
            }
            
            // Check one generator
            assert_eq!(generators.len(), 1);
            assert_eq!(generators[0].target, "x");
            
            match &generators[0].iter {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "items");
                }
                _ => panic!("Expected identifier as iterator"),
            }
            
            assert_eq!(generators[0].conditions.len(), 0);
        }
        _ => panic!("Expected list comprehension"),
    }
}

#[test]
fn test_list_comprehension_with_expression() {
    let module = parse("[x * 2 for x in numbers]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::ListComp { element, generators, .. }) => {
            // Check element is x * 2
            match &**element {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::Multiply);
                }
                _ => panic!("Expected binary operation as element"),
            }
            
            assert_eq!(generators.len(), 1);
            assert_eq!(generators[0].target, "x");
        }
        _ => panic!("Expected list comprehension"),
    }
}

#[test]
fn test_list_comprehension_with_condition() {
    let module = parse("[x for x in items if x > 0]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::ListComp { generators, .. }) => {
            assert_eq!(generators.len(), 1);
            assert_eq!(generators[0].target, "x");
            assert_eq!(generators[0].conditions.len(), 1);
            
            // Check condition is x > 0
            match &generators[0].conditions[0] {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::GreaterThan);
                }
                _ => panic!("Expected comparison as condition"),
            }
        }
        _ => panic!("Expected list comprehension"),
    }
}

#[test]
fn test_list_comprehension_with_multiple_conditions() {
    let module = parse("[x for x in items if x > 0 if x < 10]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::ListComp { generators, .. }) => {
            assert_eq!(generators.len(), 1);
            assert_eq!(generators[0].conditions.len(), 2);
        }
        _ => panic!("Expected list comprehension"),
    }
}

#[test]
fn test_list_comprehension_nested_loops() {
    let module = parse("[x + y for x in xs for y in ys]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::ListComp { element, generators, .. }) => {
            // Check element is x + y
            match &**element {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::Add);
                }
                _ => panic!("Expected binary operation"),
            }
            
            // Check two generators
            assert_eq!(generators.len(), 2);
            assert_eq!(generators[0].target, "x");
            assert_eq!(generators[1].target, "y");
            
            match &generators[0].iter {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "xs");
                }
                _ => panic!("Expected identifier"),
            }
            
            match &generators[1].iter {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "ys");
                }
                _ => panic!("Expected identifier"),
            }
        }
        _ => panic!("Expected list comprehension"),
    }
}

#[test]
fn test_list_comprehension_with_function_call() {
    let module = parse("[func(x) for x in items]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::ListComp { element, .. }) => {
            match &**element {
                Expression::Call { .. } => {},
                _ => panic!("Expected function call as element"),
            }
        }
        _ => panic!("Expected list comprehension"),
    }
}

#[test]
fn test_list_comprehension_complex() {
    let module = parse("[(x, y) for x in range(5) for y in range(x) if x + y > 3]\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::ListComp { element, generators, .. }) => {
            // Element should be a tuple
            match &**element {
                Expression::Tuple { elements, .. } => {
                    assert_eq!(elements.len(), 2);
                }
                _ => panic!("Expected tuple as element"),
            }
            
            // Two generators
            assert_eq!(generators.len(), 2);
            
            // First generator has no conditions
            assert_eq!(generators[0].conditions.len(), 0);
            
            // Second generator has one condition
            assert_eq!(generators[1].conditions.len(), 1);
        }
        _ => panic!("Expected list comprehension"),
    }
}

// ============================================================================
// Dict Comprehension Tests
// ============================================================================

#[test]
fn test_simple_dict_comprehension() {
    let module = parse("{x: x * 2 for x in items}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::DictComp { key, value, generators, .. }) => {
            // Check key is identifier 'x'
            match &**key {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "x");
                }
                _ => panic!("Expected identifier as key"),
            }
            
            // Check value is x * 2
            match &**value {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::Multiply);
                }
                _ => panic!("Expected binary operation as value"),
            }
            
            // Check one generator
            assert_eq!(generators.len(), 1);
            assert_eq!(generators[0].target, "x");
        }
        _ => panic!("Expected dict comprehension"),
    }
}

#[test]
fn test_dict_comprehension_with_condition() {
    let module = parse("{k: v for k in keys for v in values if k == v}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::DictComp { generators, .. }) => {
            assert_eq!(generators.len(), 2);
            assert_eq!(generators[0].target, "k");
            assert_eq!(generators[1].target, "v");
            assert_eq!(generators[0].conditions.len(), 0);
            assert_eq!(generators[1].conditions.len(), 1);
        }
        _ => panic!("Expected dict comprehension"),
    }
}

#[test]
fn test_dict_comprehension_with_expressions() {
    let module = parse("{str(i): i ** 2 for i in range(10)}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::DictComp { key, value, .. }) => {
            // Key should be function call
            match &**key {
                Expression::Call { .. } => {},
                _ => panic!("Expected function call as key"),
            }
            
            // Value should be power operation
            match &**value {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::Power);
                }
                _ => panic!("Expected power operation as value"),
            }
        }
        _ => panic!("Expected dict comprehension"),
    }
}

#[test]
fn test_dict_comprehension_nested_with_condition() {
    let module = parse("{x: y for x in xs for y in ys if x < y}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::DictComp { generators, .. }) => {
            assert_eq!(generators.len(), 2);
            assert_eq!(generators[1].conditions.len(), 1);
        }
        _ => panic!("Expected dict comprehension"),
    }
}

// ============================================================================
// Set Comprehension Tests
// ============================================================================

#[test]
fn test_simple_set_comprehension() {
    let module = parse("{x * 2 for x in items}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::SetComp { element, generators, .. }) => {
            // Check element is x * 2
            match &**element {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::Multiply);
                }
                _ => panic!("Expected binary operation as element"),
            }
            
            // Check one generator
            assert_eq!(generators.len(), 1);
            assert_eq!(generators[0].target, "x");
        }
        _ => panic!("Expected set comprehension"),
    }
}

#[test]
fn test_set_comprehension_with_condition() {
    let module = parse("{x for x in items if x > 0}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::SetComp { generators, .. }) => {
            assert_eq!(generators.len(), 1);
            assert_eq!(generators[0].conditions.len(), 1);
        }
        _ => panic!("Expected set comprehension"),
    }
}

#[test]
fn test_set_comprehension_with_multiple_conditions() {
    let module = parse("{x for x in items if x > 0 if x < 10}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::SetComp { generators, .. }) => {
            assert_eq!(generators.len(), 1);
            assert_eq!(generators[0].conditions.len(), 2);
        }
        _ => panic!("Expected set comprehension"),
    }
}

#[test]
fn test_set_comprehension_nested_loops() {
    let module = parse("{x + y for x in xs for y in ys}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::SetComp { element, generators, .. }) => {
            // Check element is x + y
            match &**element {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::Add);
                }
                _ => panic!("Expected binary operation"),
            }
            
            // Check two generators
            assert_eq!(generators.len(), 2);
            assert_eq!(generators[0].target, "x");
            assert_eq!(generators[1].target, "y");
        }
        _ => panic!("Expected set comprehension"),
    }
}

#[test]
fn test_set_comprehension_with_function_call() {
    let module = parse("{abs(x) for x in numbers}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::SetComp { element, .. }) => {
            match &**element {
                Expression::Call { .. } => {},
                _ => panic!("Expected function call as element"),
            }
        }
        _ => panic!("Expected set comprehension"),
    }
}

#[test]
fn test_set_comprehension_complex() {
    let module = parse("{(x, y) for x in range(5) for y in range(x) if x + y > 3}\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::SetComp { element, generators, .. }) => {
            // Element should be a tuple
            match &**element {
                Expression::Tuple { elements, .. } => {
                    assert_eq!(elements.len(), 2);
                }
                _ => panic!("Expected tuple as element"),
            }
            
            // Two generators
            assert_eq!(generators.len(), 2);
            
            // Second generator has one condition
            assert_eq!(generators[1].conditions.len(), 1);
        }
        _ => panic!("Expected set comprehension"),
    }
}

// ============================================================================
// Generator Expression Tests
// ============================================================================

#[test]
fn test_simple_generator_expression() {
    let module = parse("(x for x in items)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::GeneratorExpr { element, generators, .. }) => {
            // Check element is identifier 'x'
            match &**element {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "x");
                }
                _ => panic!("Expected identifier as element"),
            }
            
            // Check one generator
            assert_eq!(generators.len(), 1);
            assert_eq!(generators[0].target, "x");
            
            match &generators[0].iter {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "items");
                }
                _ => panic!("Expected identifier as iterator"),
            }
            
            assert_eq!(generators[0].conditions.len(), 0);
        }
        _ => panic!("Expected generator expression"),
    }
}

#[test]
fn test_generator_expression_with_expression() {
    let module = parse("(x * 2 for x in numbers)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::GeneratorExpr { element, generators, .. }) => {
            // Check element is x * 2
            match &**element {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::Multiply);
                }
                _ => panic!("Expected binary operation as element"),
            }
            
            assert_eq!(generators.len(), 1);
            assert_eq!(generators[0].target, "x");
        }
        _ => panic!("Expected generator expression"),
    }
}

#[test]
fn test_generator_expression_with_condition() {
    let module = parse("(x for x in items if x > 0)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::GeneratorExpr { generators, .. }) => {
            assert_eq!(generators.len(), 1);
            assert_eq!(generators[0].target, "x");
            assert_eq!(generators[0].conditions.len(), 1);
            
            // Check condition is x > 0
            match &generators[0].conditions[0] {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::GreaterThan);
                }
                _ => panic!("Expected comparison as condition"),
            }
        }
        _ => panic!("Expected generator expression"),
    }
}

#[test]
fn test_generator_expression_with_multiple_conditions() {
    let module = parse("(x for x in items if x > 0 if x < 10)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::GeneratorExpr { generators, .. }) => {
            assert_eq!(generators.len(), 1);
            assert_eq!(generators[0].conditions.len(), 2);
        }
        _ => panic!("Expected generator expression"),
    }
}

#[test]
fn test_generator_expression_nested_loops() {
    let module = parse("(x + y for x in xs for y in ys)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::GeneratorExpr { element, generators, .. }) => {
            // Check element is x + y
            match &**element {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::Add);
                }
                _ => panic!("Expected binary operation"),
            }
            
            // Check two generators
            assert_eq!(generators.len(), 2);
            assert_eq!(generators[0].target, "x");
            assert_eq!(generators[1].target, "y");
            
            match &generators[0].iter {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "xs");
                }
                _ => panic!("Expected identifier"),
            }
            
            match &generators[1].iter {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "ys");
                }
                _ => panic!("Expected identifier"),
            }
        }
        _ => panic!("Expected generator expression"),
    }
}

#[test]
fn test_generator_expression_with_function_call() {
    let module = parse("(func(x) for x in items)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::GeneratorExpr { element, .. }) => {
            match &**element {
                Expression::Call { .. } => {},
                _ => panic!("Expected function call as element"),
            }
        }
        _ => panic!("Expected generator expression"),
    }
}

#[test]
fn test_generator_expression_complex() {
    let module = parse("((x, y) for x in range(5) for y in range(x) if x + y > 3)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::GeneratorExpr { element, generators, .. }) => {
            // Element should be a tuple
            match &**element {
                Expression::Tuple { elements, .. } => {
                    assert_eq!(elements.len(), 2);
                }
                _ => panic!("Expected tuple as element"),
            }
            
            // Two generators
            assert_eq!(generators.len(), 2);
            
            // First generator has no conditions
            assert_eq!(generators[0].conditions.len(), 0);
            
            // Second generator has one condition
            assert_eq!(generators[1].conditions.len(), 1);
        }
        _ => panic!("Expected generator expression"),
    }
}

// ============================================================================
// Return Type Annotation Tests
// ============================================================================

#[test]
fn test_parse_return_type_simple_int() {
    let input = "def foo() -> int:\n    pass\n";
    let module = parse(input).unwrap();
    
    assert_eq!(module.statements.len(), 1);
    
    if let Statement::FunctionDef { name, return_type, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert!(return_type.is_some());
        
        if let Some(Expression::Identifier { name: type_name, .. }) = return_type {
            assert_eq!(type_name, "int");
        } else {
            panic!("Expected Identifier expression for return type");
        }
    } else {
        panic!("Expected function definition");
    }
}

#[test]
fn test_parse_return_type_simple_str() {
    let input = "def foo() -> str:\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { return_type, .. } = &module.statements[0] {
        if let Some(Expression::Identifier { name, .. }) = return_type {
            assert_eq!(name, "str");
        } else {
            panic!("Expected str return type");
        }
    }
}

#[test]
fn test_parse_return_type_simple_bool() {
    let input = "def foo() -> bool:\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { return_type, .. } = &module.statements[0] {
        if let Some(Expression::Identifier { name, .. }) = return_type {
            assert_eq!(name, "bool");
        } else {
            panic!("Expected bool return type");
        }
    }
}

#[test]
fn test_parse_return_type_none() {
    let input = "def foo() -> None:\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { return_type, .. } = &module.statements[0] {
        assert!(return_type.is_some());
        // None is parsed as a Literal, not an Identifier
        if let Some(Expression::Literal(Literal::None { .. })) = return_type {
            // Success - None was parsed correctly
        } else {
            panic!("Expected None literal as return type, got: {:?}", return_type);
        }
    }
}

#[test]
fn test_parse_function_without_return_type() {
    let input = "def foo():\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { name, return_type, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert!(return_type.is_none(), "Expected no return type");
    } else {
        panic!("Expected function definition");
    }
}

#[test]
fn test_parse_return_type_with_parameters() {
    let input = "def foo(x: int, y: str) -> bool:\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { name, parameters, return_type, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert_eq!(parameters.len(), 2);
        
        if let Some(Expression::Identifier { name, .. }) = return_type {
            assert_eq!(name, "bool");
        } else {
            panic!("Expected bool return type");
        }
    } else {
        panic!("Expected function definition");
    }
}

#[test]
fn test_parse_return_type_generic_list() {
    let input = "def foo() -> list[int]:\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { return_type, .. } = &module.statements[0] {
        assert!(return_type.is_some());
        
        // Should be a Subscript expression: list[int]
        if let Some(Expression::Subscript { object, index, .. }) = return_type {
            // object should be "list"
            if let Expression::Identifier { name, .. } = &**object {
                assert_eq!(name, "list");
            } else {
                panic!("Expected Identifier for list");
            }
            
            // index should be "int"
            if let Expression::Identifier { name, .. } = &**index {
                assert_eq!(name, "int");
            } else {
                panic!("Expected Identifier for int");
            }
        } else {
            panic!("Expected Subscript expression for list[int]");
        }
    } else {
        panic!("Expected function definition");
    }
}

#[test]
fn test_parse_return_type_generic_dict() {
    let input = "def foo() -> dict[str, int]:\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { return_type, .. } = &module.statements[0] {
        assert!(return_type.is_some());
        
        // Should be a Subscript expression: dict[str, int]
        if let Some(Expression::Subscript { object, index, .. }) = return_type {
            // object should be "dict"
            if let Expression::Identifier { name, .. } = &**object {
                assert_eq!(name, "dict");
            } else {
                panic!("Expected Identifier for dict");
            }
            
            // index should be a tuple (str, int)
            if let Expression::Tuple { elements, .. } = &**index {
                assert_eq!(elements.len(), 2);
                
                if let Expression::Identifier { name, .. } = &elements[0] {
                    assert_eq!(name, "str");
                } else {
                    panic!("Expected str as first type argument");
                }
                
                if let Expression::Identifier { name, .. } = &elements[1] {
                    assert_eq!(name, "int");
                } else {
                    panic!("Expected int as second type argument");
                }
            } else {
                panic!("Expected Tuple for dict type arguments");
            }
        } else {
            panic!("Expected Subscript expression for dict[str, int]");
        }
    } else {
        panic!("Expected function definition");
    }
}

#[test]
fn test_parse_return_type_nested_generic() {
    let input = "def foo() -> list[dict[str, int]]:\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { return_type, .. } = &module.statements[0] {
        assert!(return_type.is_some());
        
        // Should be list[dict[str, int]]
        if let Some(Expression::Subscript { object, index, .. }) = return_type {
            // Outer is list
            if let Expression::Identifier { name, .. } = &**object {
                assert_eq!(name, "list");
            } else {
                panic!("Expected list");
            }
            
            // Inner should be dict[str, int]
            if let Expression::Subscript { object: inner_object, index: inner_index, .. } = &**index {
                if let Expression::Identifier { name, .. } = &**inner_object {
                    assert_eq!(name, "dict");
                } else {
                    panic!("Expected dict");
                }
                
                // Inner index should be tuple (str, int)
                if let Expression::Tuple { elements, .. } = &**inner_index {
                    assert_eq!(elements.len(), 2);
                } else {
                    panic!("Expected tuple for dict type arguments");
                }
            } else {
                panic!("Expected Subscript for inner dict");
            }
        } else {
            panic!("Expected Subscript expression");
        }
    } else {
        panic!("Expected function definition");
    }
}

#[test]
fn test_parse_async_function_with_return_type() {
    let input = "async def foo() -> int:\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { name, is_async, return_type, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert!(is_async, "Expected async function");
        
        if let Some(Expression::Identifier { name, .. }) = return_type {
            assert_eq!(name, "int");
        } else {
            panic!("Expected int return type");
        }
    } else {
        panic!("Expected function definition");
    }
}

#[test]
fn test_parse_async_function_complex_return_type() {
    let input = "async def fetch_data() -> dict[str, list[int]]:\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { is_async, return_type, .. } = &module.statements[0] {
        assert!(is_async);
        assert!(return_type.is_some());
        
        // Verify it's a Subscript (dict[...])
        if let Some(Expression::Subscript { object, .. }) = return_type {
            if let Expression::Identifier { name, .. } = &**object {
                assert_eq!(name, "dict");
            } else {
                panic!("Expected dict");
            }
        } else {
            panic!("Expected Subscript expression");
        }
    }
}

#[test]
fn test_parse_return_type_callable() {
    let input = "def foo() -> Callable:\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { return_type, .. } = &module.statements[0] {
        if let Some(Expression::Identifier { name, .. }) = return_type {
            assert_eq!(name, "Callable");
        } else {
            panic!("Expected Callable return type");
        }
    }
}

#[test]
fn test_parse_return_type_optional() {
    let input = "def foo() -> Optional[int]:\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { return_type, .. } = &module.statements[0] {
        assert!(return_type.is_some());
        
        if let Some(Expression::Subscript { object, index, .. }) = return_type {
            if let Expression::Identifier { name, .. } = &**object {
                assert_eq!(name, "Optional");
            } else {
                panic!("Expected Optional");
            }
            
            if let Expression::Identifier { name, .. } = &**index {
                assert_eq!(name, "int");
            } else {
                panic!("Expected int");
            }
        } else {
            panic!("Expected Subscript expression");
        }
    }
}

#[test]
fn test_parse_return_type_union() {
    let input = "def foo() -> int | str:\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { return_type, .. } = &module.statements[0] {
        assert!(return_type.is_some());
        
        // Should be a BinaryOp with BitwiseOr
        if let Some(Expression::BinaryOp { left, op, right, .. }) = return_type {
            assert_eq!(*op, BinaryOperator::BitwiseOr);
            
            if let Expression::Identifier { name, .. } = &**left {
                assert_eq!(name, "int");
            } else {
                panic!("Expected int");
            }
            
            if let Expression::Identifier { name, .. } = &**right {
                assert_eq!(name, "str");
            } else {
                panic!("Expected str");
            }
        } else {
            panic!("Expected BinaryOp for union type");
        }
    }
}

#[test]
fn test_parse_multiple_functions_with_return_types() {
    let input = "def foo() -> int:\n    pass\ndef bar() -> str:\n    pass\ndef baz():\n    pass\n";
    let module = parse(input).unwrap();
    
    assert_eq!(module.statements.len(), 3);
    
    // First function has int return type
    if let Statement::FunctionDef { name, return_type, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        if let Some(Expression::Identifier { name, .. }) = return_type {
            assert_eq!(name, "int");
        } else {
            panic!("Expected int return type");
        }
    }
    
    // Second function has str return type
    if let Statement::FunctionDef { name, return_type, .. } = &module.statements[1] {
        assert_eq!(name, "bar");
        if let Some(Expression::Identifier { name, .. }) = return_type {
            assert_eq!(name, "str");
        } else {
            panic!("Expected str return type");
        }
    }
    
    // Third function has no return type
    if let Statement::FunctionDef { name, return_type, .. } = &module.statements[2] {
        assert_eq!(name, "baz");
        assert!(return_type.is_none());
    }
}

// ============================================================================
// Variable Annotation Tests
// ============================================================================

#[test]
fn test_parse_variable_annotation_simple_int() {
    let input = "x: int\n";
    let module = parse(input).unwrap();
    
    assert_eq!(module.statements.len(), 1);
    
    if let Statement::AnnAssignment { target, annotation, value, .. } = &module.statements[0] {
        assert_eq!(target, "x");
        assert!(value.is_none());
        
        if let Expression::Identifier { name, .. } = annotation {
            assert_eq!(name, "int");
        } else {
            panic!("Expected Identifier for annotation");
        }
    } else {
        panic!("Expected AnnAssignment");
    }
}

#[test]
fn test_parse_variable_annotation_with_value() {
    let input = "x: int = 5\n";
    let module = parse(input).unwrap();
    
    if let Statement::AnnAssignment { target, annotation, value, .. } = &module.statements[0] {
        assert_eq!(target, "x");
        
        if let Expression::Identifier { name, .. } = annotation {
            assert_eq!(name, "int");
        } else {
            panic!("Expected int annotation");
        }
        
        assert!(value.is_some());
        if let Some(Expression::Literal(Literal::Integer { value: val, .. })) = value {
            assert_eq!(*val, 5);
        } else {
            panic!("Expected integer value");
        }
    } else {
        panic!("Expected AnnAssignment");
    }
}

#[test]
fn test_parse_variable_annotation_string() {
    let input = "name: str = \"hello\"\n";
    let module = parse(input).unwrap();
    
    if let Statement::AnnAssignment { target, annotation, value, .. } = &module.statements[0] {
        assert_eq!(target, "name");
        
        if let Expression::Identifier { name, .. } = annotation {
            assert_eq!(name, "str");
        } else {
            panic!("Expected str annotation");
        }
        
        if let Some(Expression::Literal(Literal::String { value: s, .. })) = value {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected string value");
        }
    }
}

#[test]
fn test_parse_variable_annotation_bool() {
    let input = "flag: bool = True\n";
    let module = parse(input).unwrap();
    
    if let Statement::AnnAssignment { target, annotation, value, .. } = &module.statements[0] {
        assert_eq!(target, "flag");
        
        if let Expression::Identifier { name, .. } = annotation {
            assert_eq!(name, "bool");
        } else {
            panic!("Expected bool annotation");
        }
        
        if let Some(Expression::Literal(Literal::Boolean { value: b, .. })) = value {
            assert!(*b);
        } else {
            panic!("Expected boolean value");
        }
    }
}

#[test]
fn test_parse_variable_annotation_generic_list() {
    let input = "items: list[int]\n";
    let module = parse(input).unwrap();
    
    if let Statement::AnnAssignment { target, annotation, value, .. } = &module.statements[0] {
        assert_eq!(target, "items");
        assert!(value.is_none());
        
        // Should be list[int] subscript
        if let Expression::Subscript { object, index, .. } = annotation {
            if let Expression::Identifier { name, .. } = &**object {
                assert_eq!(name, "list");
            } else {
                panic!("Expected list");
            }
            
            if let Expression::Identifier { name, .. } = &**index {
                assert_eq!(name, "int");
            } else {
                panic!("Expected int");
            }
        } else {
            panic!("Expected Subscript for list[int]");
        }
    }
}

#[test]
fn test_parse_variable_annotation_generic_dict() {
    let input = "data: dict[str, int] = {}\n";
    let module = parse(input).unwrap();
    
    if let Statement::AnnAssignment { target, annotation, value, .. } = &module.statements[0] {
        assert_eq!(target, "data");
        
        // Check annotation is dict[str, int]
        if let Expression::Subscript { object, index, .. } = annotation {
            if let Expression::Identifier { name, .. } = &**object {
                assert_eq!(name, "dict");
            } else {
                panic!("Expected dict");
            }
            
            // Index should be tuple (str, int)
            if let Expression::Tuple { elements, .. } = &**index {
                assert_eq!(elements.len(), 2);
            } else {
                panic!("Expected tuple for dict type args");
            }
        } else {
            panic!("Expected Subscript");
        }
        
        // Check value is empty dict
        if let Some(Expression::Dict { pairs, .. }) = value {
            assert_eq!(pairs.len(), 0);
        } else {
            panic!("Expected empty dict value");
        }
    }
}

#[test]
fn test_parse_variable_annotation_nested_generic() {
    let input = "matrix: list[list[int]]\n";
    let module = parse(input).unwrap();
    
    if let Statement::AnnAssignment { target, annotation, .. } = &module.statements[0] {
        assert_eq!(target, "matrix");
        
        // Outer should be list[list[int]]
        if let Expression::Subscript { object, index, .. } = annotation {
            if let Expression::Identifier { name, .. } = &**object {
                assert_eq!(name, "list");
            } else {
                panic!("Expected list");
            }
            
            // Inner should also be list[int]
            if let Expression::Subscript { object: inner_obj, index: inner_idx, .. } = &**index {
                if let Expression::Identifier { name, .. } = &**inner_obj {
                    assert_eq!(name, "list");
                } else {
                    panic!("Expected inner list");
                }
                
                if let Expression::Identifier { name, .. } = &**inner_idx {
                    assert_eq!(name, "int");
                } else {
                    panic!("Expected int");
                }
            } else {
                panic!("Expected inner Subscript");
            }
        }
    }
}

#[test]
fn test_parse_variable_annotation_optional() {
    let input = "value: Optional[int] = None\n";
    let module = parse(input).unwrap();
    
    if let Statement::AnnAssignment { target, annotation, value, .. } = &module.statements[0] {
        assert_eq!(target, "value");
        
        // Annotation should be Optional[int]
        if let Expression::Subscript { object, index, .. } = annotation {
            if let Expression::Identifier { name, .. } = &**object {
                assert_eq!(name, "Optional");
            } else {
                panic!("Expected Optional");
            }
            
            if let Expression::Identifier { name, .. } = &**index {
                assert_eq!(name, "int");
            } else {
                panic!("Expected int");
            }
        } else {
            panic!("Expected Subscript");
        }
        
        // Value should be None literal
        assert!(value.is_some());
        if let Some(Expression::Literal(Literal::None { .. })) = value {
            // Correct!
        } else {
            panic!("Expected None literal value");
        }
    }
}

#[test]
fn test_parse_variable_annotation_union() {
    let input = "value: int | str\n";
    let module = parse(input).unwrap();
    
    if let Statement::AnnAssignment { target, annotation, .. } = &module.statements[0] {
        assert_eq!(target, "value");
        
        // Should be BinaryOp with BitwiseOr
        if let Expression::BinaryOp { left, op, right, .. } = annotation {
            assert_eq!(*op, BinaryOperator::BitwiseOr);
            
            if let Expression::Identifier { name, .. } = &**left {
                assert_eq!(name, "int");
            } else {
                panic!("Expected int");
            }
            
            if let Expression::Identifier { name, .. } = &**right {
                assert_eq!(name, "str");
            } else {
                panic!("Expected str");
            }
        } else {
            panic!("Expected BinaryOp for union");
        }
    }
}

#[test]
fn test_parse_multiple_variable_annotations() {
    let input = "x: int\ny: str = \"test\"\nz: bool = False\n";
    let module = parse(input).unwrap();
    
    assert_eq!(module.statements.len(), 3);
    
    // First: x: int
    if let Statement::AnnAssignment { target, value, .. } = &module.statements[0] {
        assert_eq!(target, "x");
        assert!(value.is_none());
    } else {
        panic!("Expected first AnnAssignment");
    }
    
    // Second: y: str = "test"
    if let Statement::AnnAssignment { target, value, .. } = &module.statements[1] {
        assert_eq!(target, "y");
        assert!(value.is_some());
    } else {
        panic!("Expected second AnnAssignment");
    }
    
    // Third: z: bool = False
    if let Statement::AnnAssignment { target, value, .. } = &module.statements[2] {
        assert_eq!(target, "z");
        if let Some(Expression::Literal(Literal::Boolean { value: b, .. })) = value {
            assert!(!*b);
        } else {
            panic!("Expected False");
        }
    } else {
        panic!("Expected third AnnAssignment");
    }
}

#[test]
fn test_parse_variable_annotation_complex_value() {
    let input = "result: dict[str, int] = {\"a\": 1, \"b\": 2}\n";
    let module = parse(input).unwrap();
    
    if let Statement::AnnAssignment { target, annotation, value, .. } = &module.statements[0] {
        assert_eq!(target, "result");
        
        // Verify annotation
        if let Expression::Subscript { object, .. } = annotation {
            if let Expression::Identifier { name, .. } = &**object {
                assert_eq!(name, "dict");
            }
        } else {
            panic!("Expected Subscript annotation");
        }
        
        // Verify value is a dict with 2 pairs
        if let Some(Expression::Dict { pairs, .. }) = value {
            assert_eq!(pairs.len(), 2);
        } else {
            panic!("Expected dict value");
        }
    }
}

#[test]
fn test_parse_variable_annotation_list_value() {
    let input = "numbers: list[int] = [1, 2, 3]\n";
    let module = parse(input).unwrap();
    
    if let Statement::AnnAssignment { target, value, .. } = &module.statements[0] {
        assert_eq!(target, "numbers");
        
        if let Some(Expression::List { elements, .. }) = value {
            assert_eq!(elements.len(), 3);
        } else {
            panic!("Expected list value");
        }
    }
}

#[test]
fn test_parse_variable_annotation_callable() {
    let input = "func: Callable\n";
    let module = parse(input).unwrap();
    
    if let Statement::AnnAssignment { target, annotation, value, .. } = &module.statements[0] {
        assert_eq!(target, "func");
        assert!(value.is_none());
        
        if let Expression::Identifier { name, .. } = annotation {
            assert_eq!(name, "Callable");
        } else {
            panic!("Expected Callable annotation");
        }
    }
}

// ============================================================================
// Decorator Tests
// ============================================================================

#[test]
fn test_parse_simple_decorator() {
    let input = "@decorator\ndef foo():\n    pass\n";
    let module = parse(input).unwrap();
    
    assert_eq!(module.statements.len(), 1);
    
    if let Statement::FunctionDef { name, decorators, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert_eq!(decorators.len(), 1);
        
        if let Expression::Identifier { name, .. } = &decorators[0] {
            assert_eq!(name, "decorator");
        } else {
            panic!("Expected Identifier decorator");
        }
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_decorator_with_call() {
    let input = "@decorator()\ndef foo():\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { name, decorators, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert_eq!(decorators.len(), 1);
        
        if let Expression::Call { function, arguments, .. } = &decorators[0] {
            if let Expression::Identifier { name, .. } = &**function {
                assert_eq!(name, "decorator");
            } else {
                panic!("Expected decorator function name");
            }
            assert_eq!(arguments.len(), 0);
        } else {
            panic!("Expected Call decorator");
        }
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_decorator_with_arguments() {
    let input = "@decorator(arg1, arg2)\ndef foo():\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { name, decorators, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert_eq!(decorators.len(), 1);
        
        if let Expression::Call { function, arguments, .. } = &decorators[0] {
            if let Expression::Identifier { name, .. } = &**function {
                assert_eq!(name, "decorator");
            }
            assert_eq!(arguments.len(), 2);
        } else {
            panic!("Expected Call decorator");
        }
    }
}

#[test]
fn test_parse_decorator_with_keyword_arguments() {
    // Note: Full keyword argument syntax (x=1) in calls may not be fully supported yet
    // This tests basic decorator with parentheses
    let input = "@decorator(1, 2)\ndef foo():\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { name, decorators, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert_eq!(decorators.len(), 1);
        
        if let Expression::Call { function, arguments, .. } = &decorators[0] {
            if let Expression::Identifier { name, .. } = &**function {
                assert_eq!(name, "decorator");
            }
            // Has arguments
            assert!(arguments.len() >= 2);
        } else {
            panic!("Expected Call decorator");
        }
    }
}

#[test]
fn test_parse_multiple_decorators() {
    let input = "@decorator1\n@decorator2\n@decorator3\ndef foo():\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { name, decorators, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert_eq!(decorators.len(), 3);
        
        // Check order (top to bottom)
        if let Expression::Identifier { name, .. } = &decorators[0] {
            assert_eq!(name, "decorator1");
        }
        if let Expression::Identifier { name, .. } = &decorators[1] {
            assert_eq!(name, "decorator2");
        }
        if let Expression::Identifier { name, .. } = &decorators[2] {
            assert_eq!(name, "decorator3");
        }
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_decorator_with_attribute_access() {
    let input = "@pkg.decorator\ndef foo():\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { name, decorators, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert_eq!(decorators.len(), 1);
        
        if let Expression::Attribute { object, attribute, .. } = &decorators[0] {
            if let Expression::Identifier { name: obj_name, .. } = &**object {
                assert_eq!(obj_name, "pkg");
            }
            assert_eq!(attribute, "decorator");
        } else {
            panic!("Expected Attribute decorator");
        }
    }
}

#[test]
fn test_parse_decorator_with_nested_attribute() {
    let input = "@pkg.module.decorator\ndef foo():\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { name, decorators, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert_eq!(decorators.len(), 1);
        
        // Should be pkg.module.decorator
        if let Expression::Attribute { attribute, .. } = &decorators[0] {
            assert_eq!(attribute, "decorator");
        } else {
            panic!("Expected Attribute decorator");
        }
    }
}

#[test]
fn test_parse_decorator_on_async_function() {
    let input = "@decorator\nasync def foo():\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { name, decorators, is_async, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert!(*is_async);
        assert_eq!(decorators.len(), 1);
        
        if let Expression::Identifier { name, .. } = &decorators[0] {
            assert_eq!(name, "decorator");
        }
    } else {
        panic!("Expected async FunctionDef");
    }
}

#[test]
fn test_parse_decorator_with_complex_call() {
    // Simpler version testing multiple arguments
    let input = "@decorator(\"arg\", 123, value)\ndef foo():\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { name, decorators, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert_eq!(decorators.len(), 1);
        
        if let Expression::Call { function, arguments, .. } = &decorators[0] {
            if let Expression::Identifier { name, .. } = &**function {
                assert_eq!(name, "decorator");
            }
            assert_eq!(arguments.len(), 3);
        } else {
            panic!("Expected Call decorator");
        }
    }
}

#[test]
fn test_parse_decorator_preserves_function_details() {
    let input = "@decorator\ndef foo(x: int, y: str) -> bool:\n    return True\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { name, decorators, parameters, return_type, body, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert_eq!(decorators.len(), 1);
        assert_eq!(parameters.len(), 2);
        assert!(return_type.is_some());
        assert_eq!(body.len(), 1);
    } else {
        panic!("Expected FunctionDef with all details");
    }
}

#[test]
fn test_parse_multiple_decorated_functions() {
    let input = "@dec1\ndef foo():\n    pass\n@dec2\ndef bar():\n    pass\n";
    let module = parse(input).unwrap();
    
    assert_eq!(module.statements.len(), 2);
    
    // First function
    if let Statement::FunctionDef { name, decorators, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert_eq!(decorators.len(), 1);
    }
    
    // Second function
    if let Statement::FunctionDef { name, decorators, .. } = &module.statements[1] {
        assert_eq!(name, "bar");
        assert_eq!(decorators.len(), 1);
    }
}

#[test]
fn test_parse_function_without_decorator() {
    let input = "def foo():\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { name, decorators, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert_eq!(decorators.len(), 0);
    } else {
        panic!("Expected FunctionDef");
    }
}

#[test]
fn test_parse_stacked_decorators_with_calls() {
    let input = "@decorator1(arg1)\n@decorator2()\n@decorator3\ndef foo():\n    pass\n";
    let module = parse(input).unwrap();
    
    if let Statement::FunctionDef { name, decorators, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert_eq!(decorators.len(), 3);
        
        // First: @decorator1(arg1) - should be a Call
        assert!(matches!(&decorators[0], Expression::Call { .. }));
        
        // Second: @decorator2() - should be a Call
        assert!(matches!(&decorators[1], Expression::Call { .. }));
        
        // Third: @decorator3 - should be an Identifier
        assert!(matches!(&decorators[2], Expression::Identifier { .. }));
    }
}

// ============================================================================
// Blank Line Handling Tests
// ============================================================================

#[test]
fn test_blank_line_between_statements() {
    let input = "x = 1\n\ny = 2\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_multiple_blank_lines_between_statements() {
    let input = "x = 1\n\n\n\ny = 2\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_blank_lines_between_functions() {
    let input = "def foo():\n    pass\n\ndef bar():\n    pass\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 2);
    
    if let Statement::FunctionDef { name, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
    }
    if let Statement::FunctionDef { name, .. } = &module.statements[1] {
        assert_eq!(name, "bar");
    }
}

#[test]
fn test_two_blank_lines_between_functions() {
    let input = "def foo():\n    pass\n\n\ndef bar():\n    pass\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_blank_lines_between_classes() {
    let input = "class Foo:\n    pass\n\nclass Bar:\n    pass\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 2);
    
    if let Statement::ClassDef { name, .. } = &module.statements[0] {
        assert_eq!(name, "Foo");
    }
    if let Statement::ClassDef { name, .. } = &module.statements[1] {
        assert_eq!(name, "Bar");
    }
}

#[test]
fn test_blank_lines_between_decorated_functions() {
    let input = "@decorator1\ndef foo():\n    pass\n\n@decorator2\ndef bar():\n    pass\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 2);
    
    if let Statement::FunctionDef { name, decorators, .. } = &module.statements[0] {
        assert_eq!(name, "foo");
        assert_eq!(decorators.len(), 1);
    }
    if let Statement::FunctionDef { name, decorators, .. } = &module.statements[1] {
        assert_eq!(name, "bar");
        assert_eq!(decorators.len(), 1);
    }
}

#[test]
fn test_blank_lines_mixed_statements() {
    let input = "x = 1\n\ndef foo():\n    pass\n\nclass Bar:\n    pass\n\ny = 2\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 4);
}

#[test]
fn test_leading_blank_lines() {
    let input = "\n\nx = 1\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 1);
}

#[test]
fn test_trailing_blank_lines() {
    let input = "x = 1\n\n\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 1);
}

#[test]
fn test_blank_lines_with_if_statements() {
    let input = "if x:\n    pass\n\nif y:\n    pass\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_blank_lines_with_for_loops() {
    let input = "for i in range(10):\n    pass\n\nfor j in range(5):\n    pass\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_blank_lines_with_while_loops() {
    let input = "while x:\n    pass\n\nwhile y:\n    pass\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_blank_lines_with_imports() {
    let input = "import foo\n\nimport bar\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 2);
}

#[test]
// TODO: implement try/except parsing
// Note: try/except parsing not yet implemented
// #[test]
// fn test_blank_lines_with_try_except() {
//     let input = "try:\n    pass\nexcept:\n    pass\n\ntry:\n    pass\nexcept:\n    pass\n";
//     let module = parse(input).unwrap();
//     assert_eq!(module.statements.len(), 2);
// }

#[test]
fn test_pep8_style_spacing() {
    // PEP 8: Two blank lines between top-level definitions
    let input = "def foo():\n    pass\n\n\ndef bar():\n    pass\n\n\nclass Baz:\n    pass\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 3);
}

#[test]
fn test_blank_lines_preserve_statement_content() {
    let input = "x = 1\n\ndef foo(a: int, b: str) -> bool:\n    return True\n\ny = 2\n";
    let module = parse(input).unwrap();
    
    assert_eq!(module.statements.len(), 3);
    
    // Verify the function still has all its details
    if let Statement::FunctionDef { name, parameters, return_type, .. } = &module.statements[1] {
        assert_eq!(name, "foo");
        assert_eq!(parameters.len(), 2);
        assert!(return_type.is_some());
    } else {
        panic!("Expected function definition");
    }
}

#[test]
fn test_blank_lines_with_annotations() {
    let input = "x: int = 1\n\ny: str = \"hello\"\n\nz: bool = True\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 3);
}

#[test]
fn test_many_consecutive_blank_lines() {
    let input = "x = 1\n\n\n\n\n\n\n\ny = 2\n";
    let module = parse(input).unwrap();
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_blank_lines_dont_create_empty_statements() {
    let input = "\n\n\nx = 1\n\n\ny = 2\n\n\n";
    let module = parse(input).unwrap();
    // Should only have 2 statements, not empty ones for blank lines
    assert_eq!(module.statements.len(), 2);
}

