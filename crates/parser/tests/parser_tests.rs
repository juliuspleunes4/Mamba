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
        Statement::Assignment { target, value, .. } => {
            // Check target is identifier
            match target {
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
