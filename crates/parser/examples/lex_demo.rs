use mamba_parser::lexer::Lexer;

fn main() {
    let source = std::fs::read_to_string("examples/test_lexer.mmb")
        .expect("Failed to read test file");
    
    let mut lexer = Lexer::new(&source);
    
    match lexer.tokenize() {
        Ok(tokens) => {
            println!("Successfully tokenized {} tokens:\n", tokens.len());
            for token in tokens {
                println!("{:?} at {} => {:?}", token.kind, token.position, token.lexeme);
            }
        }
        Err(e) => {
            eprintln!("Lexer error: {}", e);
        }
    }
}
