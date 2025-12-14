#![no_main]

use libfuzzer_sys::fuzz_target;
use mamba_parser::lexer::Lexer;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to UTF-8 string (lossy conversion is fine for fuzzing)
    if let Ok(input) = std::str::from_utf8(data) {
        // Try to tokenize the input
        // We don't care about the result, just that it doesn't panic or crash
        let mut lexer = Lexer::new(input);
        let _ = lexer.tokenize();
    }
});
