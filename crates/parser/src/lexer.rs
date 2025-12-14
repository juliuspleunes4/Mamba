//! Lexer implementation for Mamba

use crate::token::{SourcePosition, Token, TokenKind};
use mamba_error::MambaError;
use std::str::Chars;

type LexResult = Result<Token, MambaError>;

/// The lexer converts source code into a stream of tokens
pub struct Lexer<'a> {
    #[allow(dead_code)] // TODO: Used for error reporting
    source: &'a str,
    chars: Chars<'a>,
    current_char: Option<char>,
    position: SourcePosition,
    #[allow(dead_code)] // TODO: Implement indentation handling
    indent_stack: Vec<usize>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer from source code
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars();
        let current_char = chars.next();
        
        Self {
            source,
            chars,
            current_char,
            position: SourcePosition::start(),
            indent_stack: vec![0], // Start with 0 indentation
        }
    }

    /// Tokenize the entire source code
    pub fn tokenize(&mut self) -> Result<Vec<Token>, MambaError> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }

    /// Get the next token from the source
    pub fn next_token(&mut self) -> LexResult {
        // Skip whitespace (except newlines)
        self.skip_whitespace();
        
        let start_pos = self.position;
        
        match self.current_char {
            None => Ok(Token::new(TokenKind::Eof, start_pos, String::new())),
            
            Some('\n') => {
                self.advance();
                Ok(Token::new(TokenKind::Newline, start_pos, "\n".to_string()))
            }
            
            Some('#') => self.tokenize_comment(),
            
            Some('"') | Some('\'') => self.tokenize_string(),
            
            Some(c) if c.is_ascii_digit() => self.tokenize_number(),
            
            Some(c) if c.is_alphabetic() || c == '_' => self.tokenize_identifier_or_keyword(),
            
            Some('+') => self.tokenize_operator_with_assign('+', TokenKind::Plus, TokenKind::PlusAssign),
            Some('-') => self.tokenize_minus_or_arrow(),
            Some('*') => self.tokenize_star(),
            Some('/') => self.tokenize_slash(),
            Some('%') => self.tokenize_operator_with_assign('%', TokenKind::Percent, TokenKind::PercentAssign),
            
            Some('=') => self.tokenize_equal(),
            Some('!') => self.tokenize_not_equal(),
            Some('<') => self.tokenize_less(),
            Some('>') => self.tokenize_greater(),
            
            Some('&') => self.tokenize_operator_with_assign('&', TokenKind::Ampersand, TokenKind::AmpersandAssign),
            Some('|') => self.tokenize_operator_with_assign('|', TokenKind::Pipe, TokenKind::PipeAssign),
            Some('^') => self.tokenize_operator_with_assign('^', TokenKind::Caret, TokenKind::CaretAssign),
            Some('~') => {
                self.advance();
                Ok(Token::new(TokenKind::Tilde, start_pos, "~".to_string()))
            }
            
            Some('(') => {
                self.advance();
                Ok(Token::new(TokenKind::LeftParen, start_pos, "(".to_string()))
            }
            Some(')') => {
                self.advance();
                Ok(Token::new(TokenKind::RightParen, start_pos, ")".to_string()))
            }
            Some('[') => {
                self.advance();
                Ok(Token::new(TokenKind::LeftBracket, start_pos, "[".to_string()))
            }
            Some(']') => {
                self.advance();
                Ok(Token::new(TokenKind::RightBracket, start_pos, "]".to_string()))
            }
            Some('{') => {
                self.advance();
                Ok(Token::new(TokenKind::LeftBrace, start_pos, "{".to_string()))
            }
            Some('}') => {
                self.advance();
                Ok(Token::new(TokenKind::RightBrace, start_pos, "}".to_string()))
            }
            Some(',') => {
                self.advance();
                Ok(Token::new(TokenKind::Comma, start_pos, ",".to_string()))
            }
            Some(':') => self.tokenize_colon(),
            Some(';') => {
                self.advance();
                Ok(Token::new(TokenKind::Semicolon, start_pos, ";".to_string()))
            }
            Some('.') => self.tokenize_dot(),
            
            Some(c) => Err(MambaError::SyntaxError(format!(
                "Unexpected character '{}' at {}",
                c, start_pos
            ))),
        }
    }

    // Helper methods

    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            if ch == '\n' {
                self.position.line += 1;
                self.position.column = 1;
            } else {
                self.position.column += 1;
            }
            self.position.offset += ch.len_utf8();
        }
        
        self.current_char = self.chars.next();
    }

    fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn tokenize_comment(&mut self) -> LexResult {
        let start_pos = self.position;
        let mut comment = String::new();
        
        self.advance(); // Skip '#'
        
        while let Some(c) = self.current_char {
            if c == '\n' {
                break;
            }
            comment.push(c);
            self.advance();
        }
        
        Ok(Token::new(TokenKind::Comment(comment.trim().to_string()), start_pos, format!("#{}", comment)))
    }

    fn tokenize_string(&mut self) -> LexResult {
        let start_pos = self.position;
        let quote = self.current_char.unwrap();
        let mut value = String::new();
        let mut lexeme = String::from(quote);
        
        self.advance(); // Skip opening quote
        
        // TODO: Handle triple quotes, raw strings, f-strings
        
        while let Some(c) = self.current_char {
            if c == quote {
                lexeme.push(c);
                self.advance();
                return Ok(Token::new(TokenKind::String(value), start_pos, lexeme));
            }
            
            if c == '\\' {
                lexeme.push(c);
                self.advance();
                
                if let Some(escaped) = self.current_char {
                    lexeme.push(escaped);
                    let unescaped = match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '\'' => '\'',
                        '"' => '"',
                        _ => escaped, // TODO: Handle more escape sequences
                    };
                    value.push(unescaped);
                    self.advance();
                }
            } else {
                lexeme.push(c);
                value.push(c);
                self.advance();
            }
        }
        
        Err(MambaError::SyntaxError(format!(
            "Unterminated string at {}",
            start_pos
        )))
    }

    fn tokenize_number(&mut self) -> LexResult {
        let start_pos = self.position;
        let mut lexeme = String::new();
        
        // TODO: Handle hex, oct, binary literals (0x, 0o, 0b)
        
        // Read integer part
        while let Some(c) = self.current_char {
            if c.is_ascii_digit() {
                lexeme.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        // Check for float
        if self.current_char == Some('.') && self.peek().map_or(false, |c| c.is_ascii_digit()) {
            lexeme.push('.');
            self.advance();
            
            while let Some(c) = self.current_char {
                if c.is_ascii_digit() {
                    lexeme.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            
            let value = lexeme.parse::<f64>().map_err(|_| {
                MambaError::SyntaxError(format!(
                    "Invalid float literal '{}' at {}",
                    lexeme, start_pos
                ))
            })?;
            
            return Ok(Token::new(TokenKind::Float(value), start_pos, lexeme));
        }
        
        // Integer
        let value = lexeme.parse::<i64>().map_err(|_| {
            MambaError::SyntaxError(format!(
                "Invalid integer literal '{}' at {}",
                lexeme, start_pos
            ))
        })?;
        
        Ok(Token::new(TokenKind::Integer(value), start_pos, lexeme))
    }

    fn tokenize_identifier_or_keyword(&mut self) -> LexResult {
        let start_pos = self.position;
        let mut lexeme = String::new();
        
        while let Some(c) = self.current_char {
            if c.is_alphanumeric() || c == '_' {
                lexeme.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        // Check if it's a keyword
        if let Some(keyword) = TokenKind::keyword_from_str(&lexeme) {
            Ok(Token::new(keyword, start_pos, lexeme))
        } else {
            Ok(Token::new(TokenKind::Identifier(lexeme.clone()), start_pos, lexeme))
        }
    }

    fn tokenize_operator_with_assign(
        &mut self,
        op_char: char,
        single: TokenKind,
        with_assign: TokenKind,
    ) -> LexResult {
        let start_pos = self.position;
        let mut lexeme = String::from(op_char);
        
        self.advance();
        
        if self.current_char == Some('=') {
            lexeme.push('=');
            self.advance();
            Ok(Token::new(with_assign, start_pos, lexeme))
        } else {
            Ok(Token::new(single, start_pos, lexeme))
        }
    }

    fn tokenize_minus_or_arrow(&mut self) -> LexResult {
        let start_pos = self.position;
        let mut lexeme = String::from('-');
        
        self.advance();
        
        match self.current_char {
            Some('=') => {
                lexeme.push('=');
                self.advance();
                Ok(Token::new(TokenKind::MinusAssign, start_pos, lexeme))
            }
            Some('>') => {
                lexeme.push('>');
                self.advance();
                Ok(Token::new(TokenKind::Arrow, start_pos, lexeme))
            }
            _ => Ok(Token::new(TokenKind::Minus, start_pos, lexeme)),
        }
    }

    fn tokenize_star(&mut self) -> LexResult {
        let start_pos = self.position;
        let mut lexeme = String::from('*');
        
        self.advance();
        
        match self.current_char {
            Some('*') => {
                lexeme.push('*');
                self.advance();
                
                if self.current_char == Some('=') {
                    lexeme.push('=');
                    self.advance();
                    Ok(Token::new(TokenKind::DoubleStarAssign, start_pos, lexeme))
                } else {
                    Ok(Token::new(TokenKind::DoubleStar, start_pos, lexeme))
                }
            }
            Some('=') => {
                lexeme.push('=');
                self.advance();
                Ok(Token::new(TokenKind::StarAssign, start_pos, lexeme))
            }
            _ => Ok(Token::new(TokenKind::Star, start_pos, lexeme)),
        }
    }

    fn tokenize_slash(&mut self) -> LexResult {
        let start_pos = self.position;
        let mut lexeme = String::from('/');
        
        self.advance();
        
        match self.current_char {
            Some('/') => {
                lexeme.push('/');
                self.advance();
                
                if self.current_char == Some('=') {
                    lexeme.push('=');
                    self.advance();
                    Ok(Token::new(TokenKind::DoubleSlashAssign, start_pos, lexeme))
                } else {
                    Ok(Token::new(TokenKind::DoubleSlash, start_pos, lexeme))
                }
            }
            Some('=') => {
                lexeme.push('=');
                self.advance();
                Ok(Token::new(TokenKind::SlashAssign, start_pos, lexeme))
            }
            _ => Ok(Token::new(TokenKind::Slash, start_pos, lexeme)),
        }
    }

    fn tokenize_equal(&mut self) -> LexResult {
        let start_pos = self.position;
        let mut lexeme = String::from('=');
        
        self.advance();
        
        if self.current_char == Some('=') {
            lexeme.push('=');
            self.advance();
            Ok(Token::new(TokenKind::Equal, start_pos, lexeme))
        } else {
            Ok(Token::new(TokenKind::Assign, start_pos, lexeme))
        }
    }

    fn tokenize_not_equal(&mut self) -> LexResult {
        let start_pos = self.position;
        let mut lexeme = String::from('!');
        
        self.advance();
        
        if self.current_char == Some('=') {
            lexeme.push('=');
            self.advance();
            Ok(Token::new(TokenKind::NotEqual, start_pos, lexeme))
        } else {
            Err(MambaError::SyntaxError(format!(
                "Unexpected character '!' at {} (did you mean '!='?)",
                start_pos
            )))
        }
    }

    fn tokenize_less(&mut self) -> LexResult {
        let start_pos = self.position;
        let mut lexeme = String::from('<');
        
        self.advance();
        
        match self.current_char {
            Some('=') => {
                lexeme.push('=');
                self.advance();
                Ok(Token::new(TokenKind::LessEqual, start_pos, lexeme))
            }
            Some('<') => {
                lexeme.push('<');
                self.advance();
                
                if self.current_char == Some('=') {
                    lexeme.push('=');
                    self.advance();
                    Ok(Token::new(TokenKind::LeftShiftAssign, start_pos, lexeme))
                } else {
                    Ok(Token::new(TokenKind::LeftShift, start_pos, lexeme))
                }
            }
            _ => Ok(Token::new(TokenKind::Less, start_pos, lexeme)),
        }
    }

    fn tokenize_greater(&mut self) -> LexResult {
        let start_pos = self.position;
        let mut lexeme = String::from('>');
        
        self.advance();
        
        match self.current_char {
            Some('=') => {
                lexeme.push('=');
                self.advance();
                Ok(Token::new(TokenKind::GreaterEqual, start_pos, lexeme))
            }
            Some('>') => {
                lexeme.push('>');
                self.advance();
                
                if self.current_char == Some('=') {
                    lexeme.push('=');
                    self.advance();
                    Ok(Token::new(TokenKind::RightShiftAssign, start_pos, lexeme))
                } else {
                    Ok(Token::new(TokenKind::RightShift, start_pos, lexeme))
                }
            }
            _ => Ok(Token::new(TokenKind::Greater, start_pos, lexeme)),
        }
    }

    fn tokenize_colon(&mut self) -> LexResult {
        let start_pos = self.position;
        let mut lexeme = String::from(':');
        
        self.advance();
        
        if self.current_char == Some('=') {
            lexeme.push('=');
            self.advance();
            Ok(Token::new(TokenKind::Walrus, start_pos, lexeme))
        } else {
            Ok(Token::new(TokenKind::Colon, start_pos, lexeme))
        }
    }

    fn tokenize_dot(&mut self) -> LexResult {
        let start_pos = self.position;
        let mut lexeme = String::from('.');
        
        self.advance();
        
        // Check for ellipsis (...)
        if self.current_char == Some('.') {
            if self.peek() == Some('.') {
                lexeme.push('.');
                self.advance();
                lexeme.push('.');
                self.advance();
                return Ok(Token::new(TokenKind::Ellipsis, start_pos, lexeme));
            }
        }
        
        Ok(Token::new(TokenKind::Dot, start_pos, lexeme))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        
        assert_eq!(tokens[0].kind, TokenKind::Identifier("foo".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Identifier("bar_baz".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Identifier("_private".to_string()));
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
}
