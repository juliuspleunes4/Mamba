use proptest::prelude::*;
use mamba_parser::lexer::Lexer;

/// Property: Lexer should never panic on any input
#[test]
fn lexer_never_panics() {
    proptest!(|(input: String)| {
        let mut lexer = Lexer::new(&input);
        let _ = lexer.tokenize();
        // Test passes if we reach here without panicking
    });
}

/// Property: Lexer should handle arbitrary ASCII strings
#[test]
fn lexer_handles_ascii() {
    proptest!(|(input in "[[:ascii:]]*")| {
        let mut lexer = Lexer::new(&input);
        let _ = lexer.tokenize();
    });
}

/// Property: Lexer should handle valid Python identifiers
#[test]
fn lexer_handles_identifiers() {
    proptest!(|(input in "[a-zA-Z_][a-zA-Z0-9_]*")| {
        let mut lexer = Lexer::new(&input);
        let result = lexer.tokenize();
        // Valid identifier should tokenize without error
        prop_assert!(result.is_ok());
    });
}

/// Property: Lexer should handle number literals
#[test]
fn lexer_handles_numbers() {
    proptest!(|(n: i64)| {
        let input = n.to_string();
        let mut lexer = Lexer::new(&input);
        let result = lexer.tokenize();
        // Valid number should tokenize without error
        prop_assert!(result.is_ok());
    });
}

/// Property: Lexer should handle floating point numbers
#[test]
fn lexer_handles_floats() {
    proptest!(|(f in -1e15f64..1e15f64)| {
        // Generate floats that actually format with decimal points
        // Very large/small floats may format as integers or scientific notation
        let input = format!("{:.2}", f); // Always include decimal point
        let mut lexer = Lexer::new(&input);
        let result = lexer.tokenize();
        // Valid float should tokenize without error
        prop_assert!(result.is_ok(), "Failed to tokenize: {}", input);
    });
}

/// Property: Lexer should handle whitespace
#[test]
fn lexer_handles_whitespace() {
    proptest!(|(input in r"[ \t\n\r]*")| {
        let mut lexer = Lexer::new(&input);
        let _ = lexer.tokenize();
    });
}

/// Property: Lexer should handle mixed content
#[test]
fn lexer_handles_mixed_content() {
    proptest!(|(
        ident in "[a-zA-Z_][a-zA-Z0-9_]{0,20}",
        num: i32,
        ws in "[ \t]*"
    )| {
        let input = format!("{}{ws}={ws}{}", ident, num);
        let mut lexer = Lexer::new(&input);
        let result = lexer.tokenize();
        // Valid assignment should tokenize without error
        prop_assert!(result.is_ok());
    });
}

/// Property: Lexer should handle Unicode identifiers
#[test]
fn lexer_handles_unicode_identifiers() {
    proptest!(|(base in "[a-z_][a-z0-9_]{0,5}", suffix in "[α-ωа-яא-ת]{0,5}")| {
        let input = format!("{}{}", base, suffix);
        let mut lexer = Lexer::new(&input);
        let result = lexer.tokenize();
        // Valid Unicode identifier should tokenize without error
        prop_assert!(result.is_ok());
    });
}

/// Property: Tokenizing twice should give same result
#[test]
fn lexer_is_deterministic() {
    proptest!(|(input: String)| {
        let mut lexer1 = Lexer::new(&input);
        let mut lexer2 = Lexer::new(&input);
        
        let result1 = lexer1.tokenize();
        let result2 = lexer2.tokenize();
        
        // Same input should always produce same result
        match (&result1, &result2) {
            (Ok(tokens1), Ok(tokens2)) => {
                prop_assert_eq!(tokens1.len(), tokens2.len());
            }
            (Err(_), Err(_)) => {
                // Both errors is also consistent
            }
            _ => {
                // One succeeded, one failed - inconsistent!
                prop_assert!(false, "Lexer produced different results for same input");
            }
        }
    });
}

/// Property: Empty input should produce no tokens (except EOF)
#[test]
fn lexer_empty_input() {
    let mut lexer = Lexer::new("");
    let result = lexer.tokenize();
    assert!(result.is_ok());
    let tokens = result.unwrap();
    // Should only have EOF token
    assert_eq!(tokens.len(), 1);
}

/// Property: Very long identifiers should not cause issues
#[test]
fn lexer_handles_long_identifiers() {
    proptest!(|(len in 1usize..1000)| {
        let input = "a".repeat(len);
        let mut lexer = Lexer::new(&input);
        let result = lexer.tokenize();
        prop_assert!(result.is_ok());
    });
}

/// Property: Very long strings should not cause issues  
#[test]
fn lexer_handles_long_strings() {
    proptest!(|(len in 1usize..1000)| {
        let inner = "a".repeat(len);
        let input = format!("\"{}\"", inner);
        let mut lexer = Lexer::new(&input);
        let result = lexer.tokenize();
        prop_assert!(result.is_ok());
    });
}

/// Property: Operators without spaces should tokenize correctly
#[test]
fn lexer_handles_operators_no_spaces() {
    proptest!(|(a: i32, b: i32)| {
        let ops = vec!["+", "-", "*", "/", "==", "!=", "<", ">", "<=", ">="];
        for op in ops {
            let input = format!("{}{}{}", a, op, b);
            let mut lexer = Lexer::new(&input);
            let result = lexer.tokenize();
            prop_assert!(result.is_ok());
        }
    });
}

/// Property: Comments should be handled correctly
#[test]
fn lexer_handles_comments() {
    proptest!(|(comment in "[[:ascii:]]*")| {
        let input = format!("# {}", comment);
        let mut lexer = Lexer::new(&input);
        let _ = lexer.tokenize();
    });
}

/// Property: Multiple lines should be handled
#[test]
fn lexer_handles_multiple_lines() {
    proptest!(|(lines in prop::collection::vec("[a-zA-Z_][a-zA-Z0-9_]*", 1..10))| {
        let input = lines.join("\n");
        let mut lexer = Lexer::new(&input);
        let _ = lexer.tokenize();
    });
}
