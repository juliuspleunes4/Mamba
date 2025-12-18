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
fn test_not_in_simple() {
    let module = parse("5 not in numbers\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { left, op, right, .. }) => {
            assert!(matches!(op, BinaryOperator::NotIn));
            
            // Left should be 5
            match **left {
                Expression::Literal(Literal::Integer { value, .. }) => assert_eq!(value, 5),
                _ => panic!("Expected integer 5 on left"),
            }
            
            // Right should be identifier "numbers"
            match **right {
                Expression::Identifier { ref name, .. } => assert_eq!(name, "numbers"),
                _ => panic!("Expected identifier 'numbers' on right"),
            }
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_is_not_simple() {
    let module = parse("x is not None\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { left, op, right, .. }) => {
            assert!(matches!(op, BinaryOperator::IsNot));
            
            // Left should be identifier "x"
            match **left {
                Expression::Identifier { ref name, .. } => assert_eq!(name, "x"),
                _ => panic!("Expected identifier 'x' on left"),
            }
            
            // Right should be None literal
            match **right {
                Expression::Literal(Literal::None { .. }) => {},
                _ => panic!("Expected None on right"),
            }
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_complex_membership_expression() {
    // Test: "apple" in fruits and "banana" not in vegetables
    let module = parse("\"apple\" in fruits and \"banana\" not in vegetables\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    // Top level should be "and"
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { op: BinaryOperator::And, left, right, .. }) => {
            // Left side: "apple" in fruits
            match **left {
                Expression::BinaryOp { op: BinaryOperator::In, .. } => {},
                _ => panic!("Expected 'in' operator on left side of 'and'"),
            }
            
            // Right side: "banana" not in vegetables
            match **right {
                Expression::BinaryOp { op: BinaryOperator::NotIn, .. } => {},
                _ => panic!("Expected 'not in' operator on right side of 'and'"),
            }
        }
        _ => panic!("Expected 'and' at top level"),
    }
}

#[test]
fn test_complex_identity_expression() {
    // Test: result is not None and result is valid
    let module = parse("result is not None and result is valid\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    // Top level should be "and"
    match &module.statements[0] {
        Statement::Expression(Expression::BinaryOp { op: BinaryOperator::And, left, right, .. }) => {
            // Left side: result is not None
            match **left {
                Expression::BinaryOp { op: BinaryOperator::IsNot, .. } => {},
                _ => panic!("Expected 'is not' operator on left side of 'and'"),
            }
            
            // Right side: result is valid
            match **right {
                Expression::BinaryOp { op: BinaryOperator::Is, .. } => {},
                _ => panic!("Expected 'is' operator on right side of 'and'"),
            }
        }
        _ => panic!("Expected 'and' at top level"),
    }
}

#[test]
fn test_not_operator_vs_not_in() {
    // Test that standalone "not" still works as unary operator
    // not (x in list) should parse as NOT (x IN list)
    let module = parse("not (x in list)\n").unwrap();
    assert_eq!(module.statements.len(), 1);
    
    match &module.statements[0] {
        Statement::Expression(Expression::UnaryOp { op, operand, .. }) => {
            assert!(matches!(op, UnaryOperator::Not));
            
            // Operand should be parenthesized (x in list)
            match **operand {
                Expression::Parenthesized { ref expr, .. } => {
                    match **expr {
                        Expression::BinaryOp { op: BinaryOperator::In, .. } => {},
                        _ => panic!("Expected 'in' operator inside parentheses"),
                    }
                }
                _ => panic!("Expected parenthesized expression"),
            }
        }
        _ => panic!("Expected unary not operation"),
    }
}
