use mamba_parser::lexer::Lexer;
use mamba_parser::token::TokenKind;

#[test]
fn test_no_indentation() {
    let mut lexer = Lexer::new("x = 1\ny = 2");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("x".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Assign);
    assert_eq!(tokens[2].kind, TokenKind::Integer(1));
    assert_eq!(tokens[3].kind, TokenKind::Newline);
    assert_eq!(tokens[4].kind, TokenKind::Identifier("y".to_string()));
}

#[test]
fn test_single_indent() {
    let mut lexer = Lexer::new("if x:\n    y = 1");
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // if x : NEWLINE INDENT y = 1
    assert!(matches!(token_kinds[0], TokenKind::If));
    assert!(matches!(token_kinds[1], TokenKind::Identifier(_)));
    assert!(matches!(token_kinds[2], TokenKind::Colon));
    assert!(matches!(token_kinds[3], TokenKind::Newline));
    assert!(matches!(token_kinds[4], TokenKind::Indent));
    assert!(matches!(token_kinds[5], TokenKind::Identifier(_)));
}

#[test]
fn test_indent_dedent() {
    let mut lexer = Lexer::new("if x:\n    y = 1\nz = 2");
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Find INDENT and DEDENT tokens
    let indent_pos = token_kinds.iter().position(|k| matches!(k, TokenKind::Indent));
    let dedent_pos = token_kinds.iter().position(|k| matches!(k, TokenKind::Dedent));
    
    assert!(indent_pos.is_some(), "Should have INDENT token");
    assert!(dedent_pos.is_some(), "Should have DEDENT token");
    assert!(indent_pos.unwrap() < dedent_pos.unwrap(), "INDENT should come before DEDENT");
}

#[test]
fn test_multiple_indent_levels() {
    let code = "if x:\n    if y:\n        z = 1";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let indent_count = tokens.iter().filter(|t| matches!(t.kind, TokenKind::Indent)).count();
    assert_eq!(indent_count, 2, "Should have 2 INDENT tokens");
}

#[test]
fn test_multiple_dedents() {
    let code = "if x:\n    if y:\n        z = 1\na = 2";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let dedent_count = tokens.iter().filter(|t| matches!(t.kind, TokenKind::Dedent)).count();
    assert_eq!(dedent_count, 2, "Should have 2 DEDENT tokens");
}

#[test]
fn test_empty_line_preserves_indentation() {
    let code = "if x:\n    y = 1\n\n    z = 2";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Should have 1 INDENT and 1 DEDENT at EOF (empty line doesn't affect indentation)
    let indent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Indent)).count();
    let dedent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Dedent)).count();
    
    assert_eq!(indent_count, 1);
    assert_eq!(dedent_count, 1); // DEDENT at EOF
}

#[test]
fn test_comment_line_preserves_indentation() {
    let code = "if x:\n    y = 1\n    # comment\n    z = 2";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Should have 1 INDENT and 1 DEDENT at EOF (comment line doesn't affect indentation)
    let indent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Indent)).count();
    let dedent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Dedent)).count();
    
    assert_eq!(indent_count, 1);
    assert_eq!(dedent_count, 1); // DEDENT at EOF
}

#[test]
fn test_mixed_tabs_and_spaces_error() {
    let code = "if x:\n\t    y = 1"; // Tab followed by spaces
    let mut lexer = Lexer::new(code);
    let result = lexer.tokenize();
    
    assert!(result.is_err(), "Mixed tabs and spaces should be an error");
}

#[test]
fn test_inconsistent_dedent_error() {
    let code = "if x:\n    if y:\n        z = 1\n   a = 2"; // 3 spaces (invalid)
    let mut lexer = Lexer::new(code);
    let result = lexer.tokenize();
    
    assert!(result.is_err(), "Inconsistent dedent should be an error");
}

#[test]
fn test_dedent_at_eof() {
    let code = "if x:\n    y = 1";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Should emit DEDENT before EOF
    let dedent_pos = token_kinds.iter().position(|k| matches!(k, TokenKind::Dedent));
    let eof_pos = token_kinds.iter().position(|k| matches!(k, TokenKind::Eof));
    
    assert!(dedent_pos.is_some(), "Should have DEDENT at EOF");
    assert!(eof_pos.is_some(), "Should have EOF");
    assert!(dedent_pos.unwrap() < eof_pos.unwrap(), "DEDENT should come before EOF");
}

#[test]
fn test_multiple_dedents_at_eof() {
    let code = "if x:\n    if y:\n        z = 1";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Should emit 2 DEDENTs before EOF
    let dedent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Dedent)).count();
    assert_eq!(dedent_count, 2, "Should have 2 DEDENT tokens at EOF");
}

#[test]
fn test_if_else_block() {
    let code = "if x:\n    y = 1\nelse:\n    z = 2";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Should have 2 INDENTs (one for if block, one for else block)
    let indent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Indent)).count();
    assert_eq!(indent_count, 2);
}

#[test]
fn test_function_definition() {
    let code = "def foo():\n    x = 1\n    y = 2\n    return x + y";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Should have 1 INDENT and 1 DEDENT
    let indent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Indent)).count();
    let dedent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Dedent)).count();
    
    assert_eq!(indent_count, 1);
    assert_eq!(dedent_count, 1);
}

#[test]
fn test_nested_function() {
    let code = "def outer():\n    def inner():\n        x = 1\n    y = 2";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Should have 2 INDENTs and 2 DEDENTs
    let indent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Indent)).count();
    let dedent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Dedent)).count();
    
    assert_eq!(indent_count, 2);
    assert_eq!(dedent_count, 2);
}

#[test]
fn test_for_loop() {
    let code = "for i in range(10):\n    print(i)";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Should have 1 INDENT and 1 DEDENT
    let indent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Indent)).count();
    let dedent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Dedent)).count();
    
    assert_eq!(indent_count, 1);
    assert_eq!(dedent_count, 1);
}

#[test]
fn test_while_loop() {
    let code = "while x > 0:\n    x = x - 1";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    let indent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Indent)).count();
    let dedent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Dedent)).count();
    
    assert_eq!(indent_count, 1);
    assert_eq!(dedent_count, 1);
}

#[test]
fn test_class_definition() {
    let code = "class Foo:\n    def bar(self):\n        pass";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Should have 2 INDENTs (class body, method body) and 2 DEDENTs
    let indent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Indent)).count();
    let dedent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Dedent)).count();
    
    assert_eq!(indent_count, 2);
    assert_eq!(dedent_count, 2);
}

#[test]
fn test_try_except() {
    let code = "try:\n    x = 1\nexcept:\n    y = 2";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Should have 2 INDENTs (try block, except block)
    let indent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Indent)).count();
    assert_eq!(indent_count, 2);
}

#[test]
fn test_complex_nesting() {
    let code = "if a:\n    if b:\n        if c:\n            x = 1\n        y = 2\n    z = 3";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // 3 indents, 3 dedents
    let indent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Indent)).count();
    let dedent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Dedent)).count();
    
    assert_eq!(indent_count, 3);
    assert_eq!(dedent_count, 3);
}

#[test]
fn test_only_spaces() {
    let code = "    ";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    // Empty line - should just have EOF
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].kind, TokenKind::Eof));
}

#[test]
fn test_tabs_only() {
    let code = "\t\t";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    // Empty line - should just have EOF
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].kind, TokenKind::Eof));
}

#[test]
fn test_indent_with_tabs() {
    let code = "if x:\n\ty = 1";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Should handle tabs consistently
    let indent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Indent)).count();
    assert_eq!(indent_count, 1);
}

#[test]
fn test_multiple_statements_same_indent() {
    let code = "if x:\n    y = 1\n    z = 2\n    a = 3";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    
    // Only 1 INDENT (all statements at same level)
    let indent_count = token_kinds.iter().filter(|k| matches!(k, TokenKind::Indent)).count();
    assert_eq!(indent_count, 1);
}
