use mamba_parser::lexer::Lexer;
use mamba_parser::token::TokenKind;

#[test]
fn test_unicode_latin_identifier() {
    let mut lexer = Lexer::new("café = 42");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("café".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Assign);
    assert_eq!(tokens[2].kind, TokenKind::Integer(42));
}

#[test]
fn test_unicode_greek_identifier() {
    let mut lexer = Lexer::new("π = 3.14");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("π".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Assign);
    assert_eq!(tokens[2].kind, TokenKind::Float(3.14));
}

#[test]
fn test_unicode_chinese_identifier() {
    let mut lexer = Lexer::new("数据 = 100");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("数据".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Assign);
    assert_eq!(tokens[2].kind, TokenKind::Integer(100));
}

#[test]
fn test_unicode_japanese_identifier() {
    let mut lexer = Lexer::new("変数 = 'hello'");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("変数".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Assign);
    assert_eq!(tokens[2].kind, TokenKind::String("hello".to_string()));
}

#[test]
fn test_unicode_arabic_identifier() {
    let mut lexer = Lexer::new("متغير = 50");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("متغير".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Assign);
    assert_eq!(tokens[2].kind, TokenKind::Integer(50));
}

#[test]
fn test_unicode_cyrillic_identifier() {
    let mut lexer = Lexer::new("переменная = True");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("переменная".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Assign);
    assert_eq!(tokens[2].kind, TokenKind::True);
}

#[test]
fn test_unicode_emoji_not_allowed() {
    // Emojis are not valid identifier characters in Python
    let mut lexer = Lexer::new("🚀 = 42");
    let result = lexer.tokenize();
    
    // Should fail because 🚀 is not alphabetic
    assert!(result.is_err());
}

#[test]
fn test_unicode_mixed_identifier() {
    let mut lexer = Lexer::new("var_π_2 = 6.28");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("var_π_2".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Assign);
    assert_eq!(tokens[2].kind, TokenKind::Float(6.28));
}

#[test]
fn test_unicode_underscore_start() {
    let mut lexer = Lexer::new("_μ = 1");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("_μ".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Assign);
    assert_eq!(tokens[2].kind, TokenKind::Integer(1));
}

#[test]
fn test_unicode_function_name() {
    let mut lexer = Lexer::new("def calculer_π():");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Def);
    assert_eq!(tokens[1].kind, TokenKind::Identifier("calculer_π".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::LeftParen);
    assert_eq!(tokens[3].kind, TokenKind::RightParen);
    assert_eq!(tokens[4].kind, TokenKind::Colon);
}

#[test]
fn test_unicode_digit_in_identifier() {
    // Unicode digits (like ٢ Arabic-Indic digit 2) should work after the first char
    let mut lexer = Lexer::new("x٢ = 5");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("x٢".to_string()));
}

#[test]
fn test_unicode_cant_start_with_digit() {
    // Even Unicode digits can't start an identifier
    let mut lexer = Lexer::new("٢x = 5");
    let result = lexer.tokenize();

    // The Arabic-Indic digit ٢ is considered a digit by Unicode,
    // but it's not ASCII, so our number tokenizer won't catch it
    // It will hit the "unexpected character" error in next_token
    assert!(result.is_err());
}

#[test]
fn test_unicode_whitespace_not_in_identifier() {
    let mut lexer = Lexer::new("var 数据 = 1");
    let tokens = lexer.tokenize().unwrap();

    // Should be two separate identifiers
    assert_eq!(tokens[0].kind, TokenKind::Identifier("var".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Identifier("数据".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Assign);
}

#[test]
fn test_unicode_mathematical_symbols() {
    let mut lexer = Lexer::new("Δ = 0.1");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("Δ".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Assign);
}

#[test]
fn test_unicode_various_scripts() {
    let code = "ελληνικά = 1\nहिनदी = 2\n한국어 = 3";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();

    // Greek
    assert_eq!(tokens[0].kind, TokenKind::Identifier("ελληνικά".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Assign);
    assert_eq!(tokens[2].kind, TokenKind::Integer(1));
    assert_eq!(tokens[3].kind, TokenKind::Newline);
    
    // Hindi (simplified to avoid combining marks)
    assert_eq!(tokens[4].kind, TokenKind::Identifier("हिनदी".to_string()));
    assert_eq!(tokens[5].kind, TokenKind::Assign);
    assert_eq!(tokens[6].kind, TokenKind::Integer(2));
    assert_eq!(tokens[7].kind, TokenKind::Newline);
    
    // Korean
    assert_eq!(tokens[8].kind, TokenKind::Identifier("한국어".to_string()));
    assert_eq!(tokens[9].kind, TokenKind::Assign);
    assert_eq!(tokens[10].kind, TokenKind::Integer(3));
}

#[test]
fn test_unicode_normalization_not_required() {
    // Different Unicode normalizations of café (é as single char vs e + combining accent)
    // Our lexer accepts them as-is without normalization
    let code1 = "café = 1"; // é as U+00E9
    let code2 = "café = 1"; // e + ́ as U+0065 + U+0301
    
    let mut lexer1 = Lexer::new(code1);
    let tokens1 = lexer1.tokenize().unwrap();
    
    let mut lexer2 = Lexer::new(code2);
    let tokens2 = lexer2.tokenize().unwrap();
    
    // Both should tokenize successfully (identifiers will be different strings though)
    assert!(matches!(tokens1[0].kind, TokenKind::Identifier(_)));
    assert!(matches!(tokens2[0].kind, TokenKind::Identifier(_)));
}

#[test]
fn test_ascii_still_works() {
    // Make sure we didn't break ASCII identifiers
    let mut lexer = Lexer::new("hello_world_123 = 'test'");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("hello_world_123".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Assign);
    assert_eq!(tokens[2].kind, TokenKind::String("test".to_string()));
}
