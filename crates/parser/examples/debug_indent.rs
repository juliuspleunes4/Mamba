use mamba_parser::lexer::Lexer;
use std::fs;

fn main() {
    let code = fs::read_to_string("examples/test_indentation.mmb")
        .expect("Failed to read test file");
    
    let mut lexer = Lexer::new(&code);
    let tokens = lexer.tokenize().unwrap();

    println!("Total tokens: {}", tokens.len());
    println!("\nIndentation tokens:");
    for (i, token) in tokens.iter().enumerate() {
        if matches!(token.kind, mamba_parser::token::TokenKind::Indent | mamba_parser::token::TokenKind::Dedent) {
            println!("{}: {:?} at line {}", i, token.kind, token.position.line);
        }
    }
    
    let indent_count = tokens.iter().filter(|t| matches!(t.kind, mamba_parser::token::TokenKind::Indent)).count();
    let dedent_count = tokens.iter().filter(|t| matches!(t.kind, mamba_parser::token::TokenKind::Dedent)).count();
    
    println!("\nSummary:");
    println!("INDENT tokens: {}", indent_count);
    println!("DEDENT tokens: {}", dedent_count);
    println!("Balanced: {}", indent_count == dedent_count);
}
