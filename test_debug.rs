use mamba_parser::lexer::Lexer;
use mamba_parser::parser::Parser;

fn main() {
    let input = "def foo():\n    pass\n\ndef bar():\n    pass\n";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    
    println!("Tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {:?}", i, token.kind);
    }
    
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(module) => println!("\nParsed successfully: {} statements", module.statements.len()),
        Err(e) => println!("\nError: {:?}", e),
    }
}
