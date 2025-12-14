use mamba_parser::token::{SourcePosition, TokenKind};

#[test]
fn test_source_position() {
    let pos = SourcePosition::new(10, 5, 100);
    assert_eq!(pos.line, 10);
    assert_eq!(pos.column, 5);
    assert_eq!(pos.offset, 100);
}

#[test]
fn test_source_position_start() {
    let pos = SourcePosition::start();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);
    assert_eq!(pos.offset, 0);
}

#[test]
fn test_keyword_detection() {
    assert!(TokenKind::If.is_keyword());
    assert!(TokenKind::For.is_keyword());
    assert!(TokenKind::True.is_keyword());
    assert!(TokenKind::False.is_keyword());
    assert!(TokenKind::None.is_keyword());
    assert!(!matches!(
        TokenKind::Identifier("foo".to_string()).is_keyword(),
        true
    ));
}

#[test]
fn test_keyword_from_str() {
    assert_eq!(TokenKind::keyword_from_str("if"), Some(TokenKind::If));
    assert_eq!(TokenKind::keyword_from_str("for"), Some(TokenKind::For));
    assert_eq!(
        TokenKind::keyword_from_str("while"),
        Some(TokenKind::While)
    );
    assert_eq!(TokenKind::keyword_from_str("def"), Some(TokenKind::Def));
    assert_eq!(
        TokenKind::keyword_from_str("class"),
        Some(TokenKind::Class)
    );
    assert_eq!(TokenKind::keyword_from_str("True"), Some(TokenKind::True));
    assert_eq!(
        TokenKind::keyword_from_str("False"),
        Some(TokenKind::False)
    );
    assert_eq!(TokenKind::keyword_from_str("None"), Some(TokenKind::None));
    assert_eq!(TokenKind::keyword_from_str("and"), Some(TokenKind::And));
    assert_eq!(TokenKind::keyword_from_str("or"), Some(TokenKind::Or));
    assert_eq!(TokenKind::keyword_from_str("not"), Some(TokenKind::Not));
    assert_eq!(TokenKind::keyword_from_str("notakeyword"), None);
}

#[test]
fn test_token_display() {
    let tok = TokenKind::Integer(42);
    assert_eq!(format!("{}", tok), "Integer(42)");

    let tok = TokenKind::Float(3.14);
    assert_eq!(format!("{}", tok), "Float(3.14)");

    let tok = TokenKind::String("hello".to_string());
    assert_eq!(format!("{}", tok), "String(\"hello\")");

    let tok = TokenKind::Identifier("foo".to_string());
    assert_eq!(format!("{}", tok), "Identifier(foo)");
}
