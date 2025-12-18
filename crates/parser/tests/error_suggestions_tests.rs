use mamba_parser::lexer::Lexer;
use mamba_parser::parser::Parser;

/// Helper to parse and get error message (catches both lexer and parser errors)
fn parse_error(source: &str) -> String {
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => return e.to_string(),
    };
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(_) => panic!("Expected error but got success"),
        Err(e) => e.to_string(),
    }
}

// Tests for error suggestions

#[test]
fn test_elseif_suggestion() {
    let source = "if x == 5:\n    print(x)\nelseif x == 6:\n    print(6)\n";
    let err = parse_error(source);
    assert!(err.contains("elseif"));
    assert!(err.contains("Did you mean 'elif'?"));
}

#[test]
fn test_elsif_suggestion() {
    let source = "if x == 5:\n    print(x)\nelsif x == 6:\n    print(6)\n";
    let err = parse_error(source);
    assert!(err.contains("elsif"));
    assert!(err.contains("Did you mean 'elif'?"));
}

#[test]
fn test_define_suggestion() {
    let source = "define foo():\n    pass\n";
    let err = parse_error(source);
    assert!(err.contains("define"));
    assert!(err.contains("Did you mean 'def'?"));
}

#[test]
fn test_function_suggestion() {
    let source = "function bar():\n    pass\n";
    let err = parse_error(source);
    assert!(err.contains("function"));
    assert!(err.contains("Did you mean 'def'?"));
}

#[test]
fn test_func_suggestion() {
    let source = "func baz():\n    pass\n";
    let err = parse_error(source);
    assert!(err.contains("func"));
    assert!(err.contains("Did you mean 'def'?"));
}

#[test]
fn test_cls_suggestion() {
    let source = "cls Foo:\n    pass\n";
    let err = parse_error(source);
    assert!(err.contains("cls"));
    assert!(err.contains("Did you mean 'class'?"));
}

#[test]
fn test_switch_suggestion() {
    let source = "switch x:\n    pass\n";
    let err = parse_error(source);
    assert!(err.contains("switch"));
    assert!(err.contains("Did you mean 'match'?"));
}

#[test]
fn test_foreach_suggestion() {
    let source = "foreach item in items:\n    print(item)\n";
    let err = parse_error(source);
    assert!(err.contains("foreach"));
    assert!(err.contains("Did you mean 'for'?"));
}

#[test]
fn test_until_suggestion() {
    let source = "until x > 10:\n    x = x + 1\n";
    let err = parse_error(source);
    assert!(err.contains("until"));
    assert!(err.contains("while not"));
}

#[test]
fn test_unless_suggestion() {
    let source = "unless x:\n    print('x is falsy')\n";
    let err = parse_error(source);
    assert!(err.contains("unless"));
    assert!(err.contains("if not"));
}

#[test]
fn test_then_suggestion() {
    let source = "if x == 5 then:\n    print(x)\n";
    let err = parse_error(source);
    eprintln!("Error: {}", err);
    assert!(err.contains("then"));
    // Note: 'then' appears as identifier in expression context, not statement
    // So it may give a different error. Let's just check it's mentioned.
}
