use mamba_parser::lexer::Lexer;
use mamba_parser::token::TokenKind;

#[test]
fn test_simple_tokens() {
    let mut lexer = Lexer::new("+ - * /");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Plus);
    assert_eq!(tokens[1].kind, TokenKind::Minus);
    assert_eq!(tokens[2].kind, TokenKind::Star);
    assert_eq!(tokens[3].kind, TokenKind::Slash);
}

#[test]
fn test_numbers() {
    let mut lexer = Lexer::new("42 3.14");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Integer(42));
    assert_eq!(tokens[1].kind, TokenKind::Float(3.14));
}

#[test]
fn test_strings() {
    let mut lexer = Lexer::new(r#""hello" 'world'"#);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::String("hello".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::String("world".to_string()));
}

#[test]
fn test_keywords() {
    let mut lexer = Lexer::new("if else for while def");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::If);
    assert_eq!(tokens[1].kind, TokenKind::Else);
    assert_eq!(tokens[2].kind, TokenKind::For);
    assert_eq!(tokens[3].kind, TokenKind::While);
    assert_eq!(tokens[4].kind, TokenKind::Def);
}

#[test]
fn test_identifiers() {
    let mut lexer = Lexer::new("foo bar_baz _private");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(
        tokens[0].kind,
        TokenKind::Identifier("foo".to_string())
    );
    assert_eq!(
        tokens[1].kind,
        TokenKind::Identifier("bar_baz".to_string())
    );
    assert_eq!(
        tokens[2].kind,
        TokenKind::Identifier("_private".to_string())
    );
}

#[test]
fn test_comparison_operators() {
    let mut lexer = Lexer::new("== != < > <= >=");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Equal);
    assert_eq!(tokens[1].kind, TokenKind::NotEqual);
    assert_eq!(tokens[2].kind, TokenKind::Less);
    assert_eq!(tokens[3].kind, TokenKind::Greater);
    assert_eq!(tokens[4].kind, TokenKind::LessEqual);
    assert_eq!(tokens[5].kind, TokenKind::GreaterEqual);
}

#[test]
fn test_assignment_operators() {
    let mut lexer = Lexer::new("= += -= *= /= //=");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Assign);
    assert_eq!(tokens[1].kind, TokenKind::PlusAssign);
    assert_eq!(tokens[2].kind, TokenKind::MinusAssign);
    assert_eq!(tokens[3].kind, TokenKind::StarAssign);
    assert_eq!(tokens[4].kind, TokenKind::SlashAssign);
    assert_eq!(tokens[5].kind, TokenKind::DoubleSlashAssign);
}

#[test]
fn test_delimiters() {
    let mut lexer = Lexer::new("( ) [ ] { } , : ;");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::LeftParen);
    assert_eq!(tokens[1].kind, TokenKind::RightParen);
    assert_eq!(tokens[2].kind, TokenKind::LeftBracket);
    assert_eq!(tokens[3].kind, TokenKind::RightBracket);
    assert_eq!(tokens[4].kind, TokenKind::LeftBrace);
    assert_eq!(tokens[5].kind, TokenKind::RightBrace);
    assert_eq!(tokens[6].kind, TokenKind::Comma);
    assert_eq!(tokens[7].kind, TokenKind::Colon);
    assert_eq!(tokens[8].kind, TokenKind::Semicolon);
}

#[test]
fn test_hex_oct_bin_numbers() {
    let mut lexer = Lexer::new("0xFF 0o77 0b1010");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Integer(255));
    assert_eq!(tokens[1].kind, TokenKind::Integer(63));
    assert_eq!(tokens[2].kind, TokenKind::Integer(10));
}

#[test]
fn test_walrus_and_arrow() {
    let mut lexer = Lexer::new(":= ->");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Walrus);
    assert_eq!(tokens[1].kind, TokenKind::Arrow);
}

#[test]
fn test_ellipsis() {
    let mut lexer = Lexer::new("...");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Ellipsis);
}

#[test]
fn test_double_star() {
    let mut lexer = Lexer::new("** **=");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::DoubleStar);
    assert_eq!(tokens[1].kind, TokenKind::DoubleStarAssign);
}

#[test]
fn test_shift_operators() {
    let mut lexer = Lexer::new("<< >> <<= >>=");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::LeftShift);
    assert_eq!(tokens[1].kind, TokenKind::RightShift);
    assert_eq!(tokens[2].kind, TokenKind::LeftShiftAssign);
    assert_eq!(tokens[3].kind, TokenKind::RightShiftAssign);
}

#[test]
fn test_comment() {
    let mut lexer = Lexer::new("# this is a comment\nx = 5");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(
        tokens[0].kind,
        TokenKind::Comment("this is a comment".to_string())
    );
    assert_eq!(tokens[1].kind, TokenKind::Newline);
    assert_eq!(
        tokens[2].kind,
        TokenKind::Identifier("x".to_string())
    );
}

#[test]
fn test_string_escapes() {
    let mut lexer = Lexer::new(r#""hello\nworld""#);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(
        tokens[0].kind,
        TokenKind::String("hello\nworld".to_string())
    );
}

#[test]
fn test_unterminated_string() {
    let mut lexer = Lexer::new("\"unterminated");
    let result = lexer.tokenize();

    assert!(result.is_err());
}

#[test]
fn test_empty_string() {
    let mut lexer = Lexer::new("\"\"");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::String(String::new()));
}

#[test]
fn test_triple_quoted_string() {
    let mut lexer = Lexer::new(
        r#""""hello
world""""#,
    );
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(
        tokens[0].kind,
        TokenKind::String("hello\nworld".to_string())
    );
}

#[test]
fn test_raw_string() {
    let mut lexer = Lexer::new(r#"r"hello\nworld""#);
    let tokens = lexer.tokenize().unwrap();

    // Raw string: backslashes are literal
    assert_eq!(
        tokens[0].kind,
        TokenKind::String(r"hello\nworld".to_string())
    );
}

#[test]
fn test_fstring_basic() {
    let mut lexer = Lexer::new(r#"f"hello""#);
    let tokens = lexer.tokenize().unwrap();

    // Basic f-string (TODO: interpolation not yet supported)
    assert_eq!(tokens[0].kind, TokenKind::String("hello".to_string()));
}

#[test]
fn test_bitwise_operators() {
    let mut lexer = Lexer::new("& | ^ ~");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Ampersand);
    assert_eq!(tokens[1].kind, TokenKind::Pipe);
    assert_eq!(tokens[2].kind, TokenKind::Caret);
    assert_eq!(tokens[3].kind, TokenKind::Tilde);
}

#[test]
fn test_floor_division() {
    let mut lexer = Lexer::new("// //=");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::DoubleSlash);
    assert_eq!(tokens[1].kind, TokenKind::DoubleSlashAssign);
}

#[test]
fn test_modulo() {
    let mut lexer = Lexer::new("% %=");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Percent);
    assert_eq!(tokens[1].kind, TokenKind::PercentAssign);
}

#[test]
fn test_dot_and_ellipsis() {
    let mut lexer = Lexer::new(". ...");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Dot);
    assert_eq!(tokens[1].kind, TokenKind::Ellipsis);
}

#[test]
fn test_all_keywords() {
    let mut lexer = Lexer::new(
        "and as assert async await break class continue def del \
         elif else except finally for from global if import in is \
         lambda nonlocal not or pass raise return try while with yield \
         match case True False None",
    );
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::And);
    assert_eq!(tokens[1].kind, TokenKind::As);
    assert_eq!(tokens[2].kind, TokenKind::Assert);
    assert_eq!(tokens[3].kind, TokenKind::Async);
    assert_eq!(tokens[4].kind, TokenKind::Await);
    assert_eq!(tokens[5].kind, TokenKind::Break);
    assert_eq!(tokens[6].kind, TokenKind::Class);
    assert_eq!(tokens[7].kind, TokenKind::Continue);
    assert_eq!(tokens[8].kind, TokenKind::Def);
    assert_eq!(tokens[9].kind, TokenKind::Del);
    assert_eq!(tokens[10].kind, TokenKind::Elif);
    assert_eq!(tokens[11].kind, TokenKind::Else);
    assert_eq!(tokens[12].kind, TokenKind::Except);
    assert_eq!(tokens[13].kind, TokenKind::Finally);
    assert_eq!(tokens[14].kind, TokenKind::For);
    assert_eq!(tokens[15].kind, TokenKind::From);
    assert_eq!(tokens[16].kind, TokenKind::Global);
    assert_eq!(tokens[17].kind, TokenKind::If);
    assert_eq!(tokens[18].kind, TokenKind::Import);
    assert_eq!(tokens[19].kind, TokenKind::In);
    assert_eq!(tokens[20].kind, TokenKind::Is);
    assert_eq!(tokens[21].kind, TokenKind::Lambda);
    assert_eq!(tokens[22].kind, TokenKind::Nonlocal);
    assert_eq!(tokens[23].kind, TokenKind::Not);
    assert_eq!(tokens[24].kind, TokenKind::Or);
    assert_eq!(tokens[25].kind, TokenKind::Pass);
    assert_eq!(tokens[26].kind, TokenKind::Raise);
    assert_eq!(tokens[27].kind, TokenKind::Return);
    assert_eq!(tokens[28].kind, TokenKind::Try);
    assert_eq!(tokens[29].kind, TokenKind::While);
    assert_eq!(tokens[30].kind, TokenKind::With);
    assert_eq!(tokens[31].kind, TokenKind::Yield);
    assert_eq!(tokens[32].kind, TokenKind::Match);
    assert_eq!(tokens[33].kind, TokenKind::Case);
    assert_eq!(tokens[34].kind, TokenKind::True);
    assert_eq!(tokens[35].kind, TokenKind::False);
    assert_eq!(tokens[36].kind, TokenKind::None);
}

#[test]
fn test_empty_input() {
    let mut lexer = Lexer::new("");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn test_whitespace_only() {
    let mut lexer = Lexer::new("   \t  \t   ");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn test_newlines_only() {
    let mut lexer = Lexer::new("\n\n\n");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Newline);
    assert_eq!(tokens[1].kind, TokenKind::Newline);
    assert_eq!(tokens[2].kind, TokenKind::Newline);
    assert_eq!(tokens[3].kind, TokenKind::Eof);
}

#[test]
fn test_complex_expression() {
    let mut lexer = Lexer::new("result = (x + y) * 2 - z / 3.14");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(
        tokens[0].kind,
        TokenKind::Identifier("result".to_string())
    );
    assert_eq!(tokens[1].kind, TokenKind::Assign);
    assert_eq!(tokens[2].kind, TokenKind::LeftParen);
    assert_eq!(
        tokens[3].kind,
        TokenKind::Identifier("x".to_string())
    );
    assert_eq!(tokens[4].kind, TokenKind::Plus);
    assert_eq!(
        tokens[5].kind,
        TokenKind::Identifier("y".to_string())
    );
    assert_eq!(tokens[6].kind, TokenKind::RightParen);
    assert_eq!(tokens[7].kind, TokenKind::Star);
    assert_eq!(tokens[8].kind, TokenKind::Integer(2));
    assert_eq!(tokens[9].kind, TokenKind::Minus);
    assert_eq!(
        tokens[10].kind,
        TokenKind::Identifier("z".to_string())
    );
    assert_eq!(tokens[11].kind, TokenKind::Slash);
    assert_eq!(tokens[12].kind, TokenKind::Float(3.14));
}

#[test]
fn test_invalid_hex_literal() {
    let mut lexer = Lexer::new("0x");
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_invalid_oct_literal() {
    let mut lexer = Lexer::new("0o");
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_invalid_bin_literal() {
    let mut lexer = Lexer::new("0b");
    let result = lexer.tokenize();
    assert!(result.is_err());
}
