use mamba_parser::lexer::Lexer;
use mamba_parser::parser::Parser;

/// Helper to parse and get error message (catches both lexer and parser errors)
fn parse_error(source: &str) -> String {
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => return e.to_string(), // Lexer error
    };
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(_) => panic!("Expected error but got success"),
        Err(e) => e.to_string(), // Parser error
    }
}

// Tests for improved error messages

#[test]
fn test_missing_colon_after_if() {
    let source = "if x == 5\n    print(x)\n";
    let err = parse_error(source);
    assert!(err.contains("Expected ':' after if"));
    // Could also contain "found newline" or similar
}

#[test]
fn test_missing_colon_after_def() {
    let source = "def foo()\n    pass\n";
    let err = parse_error(source);
    assert!(err.contains("Expected ':' after function"));
}

#[test]
fn test_missing_colon_after_class() {
    let source = "class Foo\n    pass\n";
    let err = parse_error(source);
    assert!(err.contains("Expected ':' after class"));
}

#[test]
fn test_unexpected_token_in_expression() {
    let source = "x = 5 @\n";
    let err = parse_error(source);
    // Should mention unexpected token
    assert!(err.contains("'@'") || err.contains("unexpected") || err.contains("Expected"));
}

#[test]
fn test_missing_closing_paren() {
    let source = "print(5\n";
    let err = parse_error(source);
    assert!(err.contains("Expected ')'") || err.contains("closing"));
}

#[test]
fn test_missing_closing_bracket() {
    let source = "x = [1, 2, 3\n";
    let err = parse_error(source);
    assert!(err.contains("Expected ']'") || err.contains("closing"));
}

#[test]
fn test_invalid_assignment_target() {
    let source = "5 = x\n";
    let err = parse_error(source);
    // Should indicate invalid assignment target
    assert!(err.contains("Cannot assign") || err.contains("Invalid"));
}

#[test]
fn test_unexpected_dedent() {
    let source = r#"if True:
    x = 1
  y = 2
"#;
    let err = parse_error(source);
    // Indentation error (lexer detects inconsistent indentation)
    assert!(err.contains("indent") || err.contains("Dedent") || err.contains("Unexpected"));
}

#[test]
fn test_expected_identifier_after_def() {
    let source = "def 123():\n    pass\n";
    let err = parse_error(source);
    assert!(err.contains("Expected") && err.contains("identifier"));
}

#[test]
fn test_expected_identifier_after_class() {
    let source = "class 123:\n    pass\n";
    let err = parse_error(source);
    assert!(err.contains("Expected") && err.contains("class name"));
}

#[test]
fn test_incomplete_expression() {
    let source = "x = 5 +\n";
    let err = parse_error(source);
    // Should indicate incomplete expression
    assert!(err.to_lowercase().contains("expected") || err.to_lowercase().contains("incomplete"));
}

#[test]
fn test_missing_import_module() {
    let source = "import\n";
    let err = parse_error(source);
    assert!(err.contains("Expected") && err.contains("module"));
}

#[test]
fn test_invalid_parameter_order() {
    // Non-default parameter after default parameter (in same section) is invalid
    let source = "def foo(x=1, y):\n    pass\n";
    let err = parse_error(source);
    assert!(err.contains("default") || err.contains("after"));
}

#[test]
fn test_duplicate_star_parameter() {
    let source = "def foo(*args, *more):\n    pass\n";
    let err = parse_error(source);
    assert!(err.contains("Duplicate") || err.contains("multiple"));
}

#[test]
fn test_expected_expression() {
    let source = "if :\n    pass\n";
    let err = parse_error(source);
    assert!(err.contains("Expected"));
}

#[test]
fn test_clear_position_in_error() {
    let source = "def foo():\n    x = 5 +\n";
    let err = parse_error(source);
    // Error should mention line 2
    assert!(err.contains("2:") || err.contains("at 2"));
}
