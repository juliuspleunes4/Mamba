#![no_main]

use libfuzzer_sys::fuzz_target;
use mamba_parser::lexer::Lexer;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to UTF-8 string
    if let Ok(input) = std::str::from_utf8(data) {
        // Focus on inputs with quotes (strings)
        if input.contains('"') || input.contains('\'') || input.contains('r') || input.contains('f') {
            let mut lexer = Lexer::new(input);
            let _ = lexer.tokenize();
        }
    }
});
