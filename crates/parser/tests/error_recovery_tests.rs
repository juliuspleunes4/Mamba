use mamba_parser::ast::*;
use mamba_parser::lexer::Lexer;
use mamba_parser::parser::Parser;

/// Helper to parse and get all errors
fn parse_with_errors(source: &str) -> Result<Module, Vec<String>> {
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => return Err(vec![e.to_string()]),
    };
    let mut parser = Parser::new(tokens);
    parser.parse().map_err(|errors| {
        errors.into_iter().map(|e| e.to_string()).collect()
    })
}

/// Helper to check that we got exactly one error
fn expect_single_error(source: &str) -> String {
    match parse_with_errors(source) {
        Ok(_) => panic!("Expected error but parsing succeeded"),
        Err(errors) => {
            assert_eq!(errors.len(), 1, "Expected exactly 1 error, got {}: {:?}", errors.len(), errors);
            errors[0].clone()
        }
    }
}

/// Helper to check that we got multiple errors
fn expect_multiple_errors(source: &str, expected_count: usize) -> Vec<String> {
    match parse_with_errors(source) {
        Ok(_) => panic!("Expected errors but parsing succeeded"),
        Err(errors) => {
            assert_eq!(errors.len(), expected_count, 
                "Expected {} errors, got {}: {:?}", expected_count, errors.len(), errors);
            errors
        }
    }
}

#[test]
fn test_single_syntax_error_recovery() {
    // Missing colon on if statement
    let err = expect_single_error("if x == 5\n    print(x)\n");
    assert!(err.contains("Expected ':'"));
}

#[test]
fn test_recover_to_next_statement() {
    // Error in first statement, but second statement should parse
    let source = "x = if\ny = 5\n";
    let errors = expect_multiple_errors(source, 1);
    assert!(errors[0].contains("Expected") || errors[0].contains("Unexpected"));
}

#[test]
fn test_multiple_statement_errors() {
    // Multiple statements with errors, but cascading errors are suppressed
    // Until we successfully parse something
    let source = "x = if\ny = while\nz = 5\n";
    let errors = expect_multiple_errors(source, 1);
    // First error reported, others suppressed until successful parse at z = 5
    assert!(errors.len() == 1);
}

#[test]
fn test_recover_after_missing_colon() {
    // Missing colon, but continue parsing next statement
    let source = "if x == 5\n    print('error')\nprint('ok')\n";
    let errors = expect_multiple_errors(source, 1);
    assert!(errors[0].contains("Expected ':'"));
}

#[test]
fn test_recover_in_function_definitions() {
    // Error in first function, but second function should parse
    let source = "def foo(\n    pass\n\ndef bar():\n    pass\n";
    let errors = expect_multiple_errors(source, 1);
    assert!(errors[0].contains("Expected"));
}

#[test]
fn test_recover_in_class_definitions() {
    // Error in first class, but second class should parse
    let source = "class Foo\n    pass\n\nclass Bar:\n    pass\n";
    let errors = expect_multiple_errors(source, 1);
    assert!(errors[0].contains("Expected ':'"));
}

#[test]
fn test_error_at_start_then_valid_code() {
    // Error at the very beginning, then valid code
    let source = "if\nx = 5\ny = 10\n";
    let errors = expect_multiple_errors(source, 1);
    assert!(errors[0].contains("Expected"));
}

#[test]
fn test_valid_code_then_error_then_valid() {
    // Valid, error, valid pattern
    let source = "x = 5\nif\ny = 10\n";
    let errors = expect_multiple_errors(source, 1);
    assert!(errors[0].contains("Expected"));
}

#[test]
fn test_multiple_errors_in_sequence() {
    // Multiple errors one after another - cascading suppressed
    let source = "if\nwhile\nfor\n";
    let errors = expect_multiple_errors(source, 1);
    // Only first error reported
    assert_eq!(errors.len(), 1);
}

#[test]
fn test_recover_from_invalid_assignment_target() {
    // Invalid assignment target followed by valid statement
    let source = "5 = x\ny = 10\n";
    let errors = expect_multiple_errors(source, 1);
    assert!(errors[0].contains("assign"));
}

#[test]
fn test_error_recovery_preserves_good_code() {
    // Error in middle shouldn't affect parsing before or after
    let source = "x = 1\ny = 2\nif\nz = 3\nw = 4\n";
    
    // Should report the error but continue
    match parse_with_errors(source) {
        Ok(_) => panic!("Expected error"),
        Err(errors) => {
            assert_eq!(errors.len(), 1);
            assert!(errors[0].contains("Expected"));
        }
    }
}

#[test]
fn test_recover_after_bad_expression() {
    // Bad expression followed by valid statement
    let source = "x = (5 +\ny = 10\n";
    let errors = expect_multiple_errors(source, 1);
    assert!(!errors.is_empty());
}

#[test]
fn test_no_cascading_errors() {
    // One error shouldn't cause multiple error reports for same issue
    let source = "if x == 5\n    print(x)\n";
    let errors = expect_multiple_errors(source, 1);
    // Should only report the missing colon, not cascading errors
    assert_eq!(errors.len(), 1);
}

#[test]
fn test_recover_from_incomplete_function() {
    // Incomplete function definition followed by valid code
    let source = "def foo\nx = 5\n";
    let errors = expect_multiple_errors(source, 1);
    assert!(errors[0].contains("Expected '('") || errors[0].contains("Expected"));
}

#[test]
fn test_error_in_nested_block() {
    // Error inside a block shouldn't prevent parsing outer code
    let source = "if x:\n    if\n    y = 5\nz = 10\n";
    let errors = expect_multiple_errors(source, 1);
    assert!(errors[0].contains("Expected"));
}

#[test]
fn test_multiple_errors_different_contexts() {
    // Errors in different syntactic contexts - cascading suppressed
    let source = "def foo(\nx = if\nclass Bar\n";
    let errors = expect_multiple_errors(source, 1);
    // Only first error reported
    assert_eq!(errors.len(), 1);
}

#[test]
fn test_recover_to_dedent() {
    // Error inside block, recover at dedent
    let source = "if x:\n    y = if\nz = 5\n";
    let errors = expect_multiple_errors(source, 1);
    assert!(errors[0].contains("if") || errors[0].contains("Expected"));
}

#[test]
fn test_successful_parse_no_errors() {
    // Valid code should return Ok, not Err with empty vec
    let source = "x = 5\ny = 10\n";
    match parse_with_errors(source) {
        Ok(module) => {
            assert_eq!(module.statements.len(), 2);
        }
        Err(errors) => {
            panic!("Expected successful parse, got errors: {:?}", errors);
        }
    }
}

#[test]
fn test_recover_from_bad_decorator() {
    // Bad decorator followed by valid code
    let source = "@\ndef foo():\n    pass\n";
    let errors = expect_multiple_errors(source, 1);
    assert!(!errors.is_empty());
}

#[test]
fn test_error_on_last_statement() {
    // Error on the last statement
    let source = "x = 5\ny = 10\nz = if\n";
    let errors = expect_multiple_errors(source, 1);
    assert!(errors[0].contains("if") || errors[0].contains("Expected"));
}

#[test]
fn test_multiple_distinct_errors_with_valid_code_between() {
    // Error, valid code, error - both errors should be reported
    let source = "x = if\ny = 5\nz = while\n";
    let errors = expect_multiple_errors(source, 2);
    assert_eq!(errors.len(), 2);
    // First error about 'if'
    assert!(errors[0].contains("if"));
    // Second error about 'while' (after successful parse of y = 5)
    assert!(errors[1].contains("while"));
}
