#![no_main]

use libfuzzer_sys::fuzz_target;
use mamba_parser::lexer::Lexer;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to UTF-8 string
    if let Ok(input) = std::str::from_utf8(data) {
        // Focus on inputs that look like numbers
        if input.chars().any(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == 'x' || c == 'o' || c == 'b') {
            let mut lexer = Lexer::new(input);
            let _ = lexer.tokenize();
        }
    }
});
