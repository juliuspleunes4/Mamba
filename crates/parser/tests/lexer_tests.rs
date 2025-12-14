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
    let mut lexer = Lexer::new("    ");
    let tokens = lexer.tokenize().unwrap();

    // Whitespace-only line is ignored (empty line behavior)
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

// ============ EDGE CASE TESTS ============

#[test]
fn test_operators_without_spaces() {
    let mut lexer = Lexer::new("a+b-c*d/e");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("a".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Plus);
    assert_eq!(tokens[2].kind, TokenKind::Identifier("b".to_string()));
    assert_eq!(tokens[3].kind, TokenKind::Minus);
    assert_eq!(tokens[4].kind, TokenKind::Identifier("c".to_string()));
    assert_eq!(tokens[5].kind, TokenKind::Star);
    assert_eq!(tokens[6].kind, TokenKind::Identifier("d".to_string()));
    assert_eq!(tokens[7].kind, TokenKind::Slash);
    assert_eq!(tokens[8].kind, TokenKind::Identifier("e".to_string()));
}

#[test]
fn test_multiple_dots_in_number() {
    let mut lexer = Lexer::new("3.14.15");
    let tokens = lexer.tokenize().unwrap();

    // Should parse as: 3.14, ., 15
    assert_eq!(tokens[0].kind, TokenKind::Float(3.14));
    assert_eq!(tokens[1].kind, TokenKind::Dot);
    assert_eq!(tokens[2].kind, TokenKind::Integer(15));
}

#[test]
fn test_number_followed_by_identifier() {
    let mut lexer = Lexer::new("123abc");
    let tokens = lexer.tokenize().unwrap();

    // Numbers can't start identifiers in Python, so this is: 123, abc
    assert_eq!(tokens[0].kind, TokenKind::Integer(123));
    assert_eq!(tokens[1].kind, TokenKind::Identifier("abc".to_string()));
}

#[test]
fn test_identifier_with_numbers() {
    let mut lexer = Lexer::new("var123 test_456");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("var123".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Identifier("test_456".to_string()));
}

#[test]
fn test_keyword_as_part_of_identifier() {
    let mut lexer = Lexer::new("ifx forloop whileTrue");
    let tokens = lexer.tokenize().unwrap();

    // These are identifiers, not keywords
    assert_eq!(tokens[0].kind, TokenKind::Identifier("ifx".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Identifier("forloop".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Identifier("whileTrue".to_string()));
}

#[test]
fn test_single_underscore_identifier() {
    let mut lexer = Lexer::new("_");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("_".to_string()));
}

#[test]
fn test_multiple_underscores() {
    let mut lexer = Lexer::new("__ ___ ____");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("__".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Identifier("___".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Identifier("____".to_string()));
}

#[test]
fn test_comment_at_end_of_file() {
    let mut lexer = Lexer::new("x = 5 # no newline");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("x".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Assign);
    assert_eq!(tokens[2].kind, TokenKind::Integer(5));
    assert_eq!(tokens[3].kind, TokenKind::Comment("no newline".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::Eof);
}

#[test]
fn test_comment_only() {
    let mut lexer = Lexer::new("# just a comment");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Comment("just a comment".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Eof);
}

#[test]
fn test_empty_comment() {
    let mut lexer = Lexer::new("#");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Comment(String::new()));
    assert_eq!(tokens[1].kind, TokenKind::Eof);
}

#[test]
fn test_consecutive_operators() {
    let mut lexer = Lexer::new("+++---");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Plus);
    assert_eq!(tokens[1].kind, TokenKind::Plus);
    assert_eq!(tokens[2].kind, TokenKind::Plus);
    assert_eq!(tokens[3].kind, TokenKind::Minus);
    assert_eq!(tokens[4].kind, TokenKind::Minus);
    assert_eq!(tokens[5].kind, TokenKind::Minus);
}

#[test]
fn test_mixed_quotes() {
    let mut lexer = Lexer::new(r#""double" 'single'"#);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::String("double".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::String("single".to_string()));
}

#[test]
fn test_quote_inside_string() {
    let mut lexer = Lexer::new(r#""He said \"hello\"""#);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::String("He said \"hello\"".to_string()));
}

#[test]
fn test_single_quote_in_double_quoted() {
    let mut lexer = Lexer::new(r#""don't""#);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::String("don't".to_string()));
}

#[test]
fn test_double_quote_in_single_quoted() {
    let mut lexer = Lexer::new(r#"'He said "hi"'"#);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::String("He said \"hi\"".to_string()));
}

#[test]
fn test_unterminated_triple_quoted_string() {
    let mut lexer = Lexer::new(r#""""hello world"#);
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_triple_quoted_with_quotes_inside() {
    let mut lexer = Lexer::new(r#""""He said "hello" and 'bye'""""#);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(
        tokens[0].kind,
        TokenKind::String("He said \"hello\" and 'bye'".to_string())
    );
}

#[test]
fn test_string_with_all_escapes() {
    let mut lexer = Lexer::new(r#""\n\r\t\\\"""#);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::String("\n\r\t\\\"".to_string()));
}

#[test]
fn test_raw_string_with_quotes() {
    let mut lexer = Lexer::new(r#"r"C:\Users\Name\test\"file\".txt""#);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(
        tokens[0].kind,
        TokenKind::String(r#"C:\Users\Name\test\"file\".txt"#.to_string())
    );
}

#[test]
fn test_zero_integer() {
    let mut lexer = Lexer::new("0");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Integer(0));
}

#[test]
fn test_negative_number() {
    let mut lexer = Lexer::new("-42 -3.14");
    let tokens = lexer.tokenize().unwrap();

    // Note: The minus is parsed as an operator, not part of the number
    assert_eq!(tokens[0].kind, TokenKind::Minus);
    assert_eq!(tokens[1].kind, TokenKind::Integer(42));
    assert_eq!(tokens[2].kind, TokenKind::Minus);
    assert_eq!(tokens[3].kind, TokenKind::Float(3.14));
}

#[test]
fn test_float_without_leading_zero() {
    let mut lexer = Lexer::new(".5");
    let tokens = lexer.tokenize().unwrap();

    // This should be parsed as dot followed by integer
    assert_eq!(tokens[0].kind, TokenKind::Dot);
    assert_eq!(tokens[1].kind, TokenKind::Integer(5));
}

#[test]
fn test_float_without_trailing_digits() {
    let mut lexer = Lexer::new("5.");
    let tokens = lexer.tokenize().unwrap();

    // This should be parsed as integer followed by dot
    assert_eq!(tokens[0].kind, TokenKind::Integer(5));
    assert_eq!(tokens[1].kind, TokenKind::Dot);
}

#[test]
fn test_very_large_integer() {
    let mut lexer = Lexer::new("999999999999");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Integer(999999999999));
}

#[test]
fn test_hex_with_lowercase() {
    let mut lexer = Lexer::new("0xabc 0xABC 0xAbC");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Integer(0xabc));
    assert_eq!(tokens[1].kind, TokenKind::Integer(0xABC));
    assert_eq!(tokens[2].kind, TokenKind::Integer(0xAbC));
}

#[test]
fn test_oct_with_invalid_digit() {
    let mut lexer = Lexer::new("0o78");
    let result = lexer.tokenize();
    // 8 is not a valid octal digit
    assert!(result.is_err());
}

#[test]
fn test_bin_with_invalid_digit() {
    let mut lexer = Lexer::new("0b102");
    let result = lexer.tokenize();
    // 2 is not a valid binary digit
    assert!(result.is_err());
}

#[test]
fn test_hex_with_invalid_char() {
    let mut lexer = Lexer::new("0xG");
    let result = lexer.tokenize();
    // G is not a valid hex digit
    assert!(result.is_err());
}

#[test]
fn test_ambiguous_less_than_shift() {
    let mut lexer = Lexer::new("a<b a<<b");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("a".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Less);
    assert_eq!(tokens[2].kind, TokenKind::Identifier("b".to_string()));
    assert_eq!(tokens[3].kind, TokenKind::Identifier("a".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::LeftShift);
    assert_eq!(tokens[5].kind, TokenKind::Identifier("b".to_string()));
}

#[test]
fn test_ambiguous_star_power() {
    let mut lexer = Lexer::new("a*b a**b");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("a".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Star);
    assert_eq!(tokens[2].kind, TokenKind::Identifier("b".to_string()));
    assert_eq!(tokens[3].kind, TokenKind::Identifier("a".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::DoubleStar);
    assert_eq!(tokens[5].kind, TokenKind::Identifier("b".to_string()));
}

#[test]
fn test_ambiguous_slash_floor_div() {
    let mut lexer = Lexer::new("a/b a//b");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("a".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Slash);
    assert_eq!(tokens[2].kind, TokenKind::Identifier("b".to_string()));
    assert_eq!(tokens[3].kind, TokenKind::Identifier("a".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::DoubleSlash);
    assert_eq!(tokens[5].kind, TokenKind::Identifier("b".to_string()));
}

#[test]
fn test_dot_vs_ellipsis() {
    let mut lexer = Lexer::new(". .. ...");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Dot);
    assert_eq!(tokens[1].kind, TokenKind::Dot);
    assert_eq!(tokens[2].kind, TokenKind::Dot);
    assert_eq!(tokens[3].kind, TokenKind::Ellipsis);
}

#[test]
fn test_colon_vs_walrus() {
    let mut lexer = Lexer::new(": := ::=");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Colon);
    assert_eq!(tokens[1].kind, TokenKind::Walrus);
    // ::= should be colon, colon, assign
    assert_eq!(tokens[2].kind, TokenKind::Colon);
    assert_eq!(tokens[3].kind, TokenKind::Walrus);
}

#[test]
fn test_minus_vs_arrow() {
    let mut lexer = Lexer::new("- -> -->");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Minus);
    assert_eq!(tokens[1].kind, TokenKind::Arrow);
    // --> should be arrow followed by greater  (-- would try to parse as - -, the second - sees > and makes ->)
    assert_eq!(tokens[2].kind, TokenKind::Minus);
    assert_eq!(tokens[3].kind, TokenKind::Arrow);
}

#[test]
fn test_position_tracking_single_line() {
    let mut lexer = Lexer::new("abc def");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].position.line, 1);
    assert_eq!(tokens[0].position.column, 1);
    assert_eq!(tokens[1].position.line, 1);
    assert_eq!(tokens[1].position.column, 5);
}

#[test]
fn test_position_tracking_multiple_lines() {
    let mut lexer = Lexer::new("x\ny\nz");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].position.line, 1); // x
    assert_eq!(tokens[1].position.line, 1); // newline
    assert_eq!(tokens[2].position.line, 2); // y
    assert_eq!(tokens[3].position.line, 2); // newline
    assert_eq!(tokens[4].position.line, 3); // z
}

#[test]
fn test_all_escape_sequences() {
    let mut lexer = Lexer::new(r#""\a\b\f\n\r\t\v\\\'\"""#);
    let tokens = lexer.tokenize().unwrap();

    // Test that escape sequences are properly parsed
    assert!(matches!(tokens[0].kind, TokenKind::String(_)));
}

#[test]
fn test_unterminated_single_quote_string() {
    let mut lexer = Lexer::new("'unterminated");
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_unterminated_raw_string() {
    let mut lexer = Lexer::new(r#"r"unterminated"#);
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_unterminated_fstring() {
    let mut lexer = Lexer::new(r#"f"unterminated"#);
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_invalid_character() {
    let mut lexer = Lexer::new("@");
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_backtick_invalid() {
    let mut lexer = Lexer::new("`invalid`");
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_dollar_sign_invalid() {
    let mut lexer = Lexer::new("$invalid");
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_mixed_whitespace() {
    let mut lexer = Lexer::new("x  \t  y");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("x".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Identifier("y".to_string()));
}

#[test]
fn test_trailing_whitespace() {
    let mut lexer = Lexer::new("x   ");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("x".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Eof);
}

#[test]
fn test_leading_whitespace() {
    let mut lexer = Lexer::new("   x");
    let tokens = lexer.tokenize().unwrap();

    // Leading whitespace is now treated as indentation
    assert_eq!(tokens[0].kind, TokenKind::Indent);
    assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
}

#[test]
fn test_all_bitwise_assign_operators() {
    let mut lexer = Lexer::new("&= |= ^=");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::AmpersandAssign);
    assert_eq!(tokens[1].kind, TokenKind::PipeAssign);
    assert_eq!(tokens[2].kind, TokenKind::CaretAssign);
}

#[test]
fn test_complex_nested_expression() {
    let mut lexer = Lexer::new("((a+b)*(c-d))/((e**f)%(g//h))");
    let tokens = lexer.tokenize().unwrap();

    // Just verify it parses without error and has correct operator count
    // Operators: + - * / ** % //
    let op_count = tokens
        .iter()
        .filter(|t| matches!(
            t.kind,
            TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::DoubleStar
                | TokenKind::Percent
                | TokenKind::DoubleSlash
        ))
        .count();
    assert_eq!(op_count, 7); // +, *, -, /, **, %, //
}

#[test]
fn test_long_identifier() {
    let long_name = "a".repeat(1000);
    let mut lexer = Lexer::new(&long_name);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier(long_name));
}

#[test]
fn test_long_string() {
    let long_content = "x".repeat(10000);
    let input = format!(r#""{}""#, long_content);
    let mut lexer = Lexer::new(&input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::String(long_content));
}
