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
        
        // Check for triple-quoted string
        let is_triple = if self.current_char == Some(quote) && self.peek() == Some(quote) {
            lexeme.push(quote);
            lexeme.push(quote);
            self.advance();
            self.advance();
            true
        } else {
            false
        };
        
        if is_triple {
            // Triple-quoted string (multiline)
            let mut quote_count = 0;
            
            while let Some(c) = self.current_char {
                if c == quote {
                    quote_count += 1;
                    lexeme.push(c);
                    self.advance();
                    
                    if quote_count == 3 {
                        return Ok(Token::new(TokenKind::String(value), start_pos, lexeme));
                    }
                } else {
                    // Add any accumulated quotes to value
                    for _ in 0..quote_count {
                        value.push(quote);
                    }
                    quote_count = 0;
                    
                    lexeme.push(c);
                    value.push(c);
                    self.advance();
                }
            }
            
            return Err(MambaError::SyntaxError(format!(
                "Unterminated triple-quoted string at {}",
                start_pos
            )));
        }
        
        // Single-quoted string
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
                        '0' => '\0',
                        _ => escaped,
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
        
        // Check for hex, oct, binary literals (0x, 0o, 0b)
        if self.current_char == Some('0') {
            lexeme.push('0');
            self.advance();
            
            match self.current_char {
                Some('x') | Some('X') => {
                    lexeme.push('x');
                    self.advance();
                    return self.tokenize_hex_number(start_pos, lexeme);
                }
                Some('o') | Some('O') => {
                    lexeme.push('o');
                    self.advance();
                    return self.tokenize_oct_number(start_pos, lexeme);
                }
                Some('b') | Some('B') => {
                    lexeme.push('b');
                    self.advance();
                    return self.tokenize_bin_number(start_pos, lexeme);
                }
                _ => {}
            }
        }
        
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
        
        // Check for raw strings (r"..." or r'...')
        if (lexeme == "r" || lexeme == "R") && matches!(self.current_char, Some('"') | Some('\'')) {
            return self.tokenize_raw_string();
        }
        
        // Check for f-strings (f"..." or f'...')
        if (lexeme == "f" || lexeme == "F") && matches!(self.current_char, Some('"') | Some('\'')) {
            return self.tokenize_fstring();
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

    fn tokenize_hex_number(&mut self, start_pos: SourcePosition, mut lexeme: String) -> LexResult {
        let mut has_digits = false;
        
        while let Some(c) = self.current_char {
            if c.is_ascii_hexdigit() {
                lexeme.push(c);
                self.advance();
                has_digits = true;
            } else {
                break;
            }
        }
        
        if !has_digits {
            return Err(MambaError::SyntaxError(format!(
                "Invalid hexadecimal literal '{}' at {}",
                lexeme, start_pos
            )));
        }
        
        let value = i64::from_str_radix(&lexeme[2..], 16).map_err(|_| {
            MambaError::SyntaxError(format!(
                "Invalid hexadecimal literal '{}' at {}",
                lexeme, start_pos
            ))
        })?;
        
        Ok(Token::new(TokenKind::Integer(value), start_pos, lexeme))
    }

    fn tokenize_oct_number(&mut self, start_pos: SourcePosition, mut lexeme: String) -> LexResult {
        let mut has_digits = false;
        
        while let Some(c) = self.current_char {
            if c >= '0' && c <= '7' {
                lexeme.push(c);
                self.advance();
                has_digits = true;
            } else if c.is_ascii_digit() {
                // Invalid octal digit (8 or 9)
                return Err(MambaError::SyntaxError(format!(
                    "Invalid octal digit '{}' in '{}' at {}",
                    c, lexeme, start_pos
                )));
            } else {
                break;
            }
        }
        
        if !has_digits {
            return Err(MambaError::SyntaxError(format!(
                "Invalid octal literal '{}' at {}",
                lexeme, start_pos
            )));
        }
        
        let value = i64::from_str_radix(&lexeme[2..], 8).map_err(|_| {
            MambaError::SyntaxError(format!(
                "Invalid octal literal '{}' at {}",
                lexeme, start_pos
            ))
        })?;
        
        Ok(Token::new(TokenKind::Integer(value), start_pos, lexeme))
    }

    fn tokenize_bin_number(&mut self, start_pos: SourcePosition, mut lexeme: String) -> LexResult {
        let mut has_digits = false;
        
        while let Some(c) = self.current_char {
            if c == '0' || c == '1' {
                lexeme.push(c);
                self.advance();
                has_digits = true;
            } else if c.is_ascii_digit() {
                // Invalid binary digit (2-9)
                return Err(MambaError::SyntaxError(format!(
                    "Invalid binary digit '{}' in '{}' at {}",
                    c, lexeme, start_pos
                )));
            } else {
                break;
            }
        }
        
        if !has_digits {
            return Err(MambaError::SyntaxError(format!(
                "Invalid binary literal '{}' at {}",
                lexeme, start_pos
            )));
        }
        
        let value = i64::from_str_radix(&lexeme[2..], 2).map_err(|_| {
            MambaError::SyntaxError(format!(
                "Invalid binary literal '{}' at {}",
                lexeme, start_pos
            ))
        })?;
        
        Ok(Token::new(TokenKind::Integer(value), start_pos, lexeme))
    }

    fn tokenize_raw_string(&mut self) -> LexResult {
        let start_pos = self.position;
        let quote = self.current_char.unwrap();
        let mut value = String::new();
        let mut lexeme = format!("r{}", quote);
        
        self.advance(); // Skip opening quote
        
        // Check for triple-quoted raw string
        let is_triple = if self.current_char == Some(quote) && self.peek() == Some(quote) {
            lexeme.push(quote);
            lexeme.push(quote);
            self.advance();
            self.advance();
            true
        } else {
            false
        };
        
        if is_triple {
            // Triple-quoted raw string
            let mut quote_count = 0;
            
            while let Some(c) = self.current_char {
                if c == quote {
                    quote_count += 1;
                    lexeme.push(c);
                    self.advance();
                    
                    if quote_count == 3 {
                        return Ok(Token::new(TokenKind::String(value), start_pos, lexeme));
                    }
                } else {
                    for _ in 0..quote_count {
                        value.push(quote);
                    }
                    quote_count = 0;
                    
                    lexeme.push(c);
                    value.push(c); // Raw string: no escape processing
                    self.advance();
                }
            }
            
            return Err(MambaError::SyntaxError(format!(
                "Unterminated triple-quoted raw string at {}",
                start_pos
            )));
        }
        
        // Single-quoted raw string - no escape sequences processed
        // BUT we still need to handle escaped quotes in raw strings (r"\"" should include the backslash AND quote)
        while let Some(c) = self.current_char {
            if c == '\\' && self.peek() == Some(quote) {
                // In raw strings, backslash before quote is literal
                lexeme.push(c);
                value.push(c);
                self.advance();
                
                // Also include the quote
                if let Some(q) = self.current_char {
                    lexeme.push(q);
                    value.push(q);
                    self.advance();
                }
            } else if c == quote {
                lexeme.push(c);
                self.advance();
                return Ok(Token::new(TokenKind::String(value), start_pos, lexeme));
            } else {
                lexeme.push(c);
                value.push(c); // Raw string: backslashes are literal
                self.advance();
            }
        }
        
        Err(MambaError::SyntaxError(format!(
            "Unterminated raw string at {}",
            start_pos
        )))
    }

    fn tokenize_fstring(&mut self) -> LexResult {
        let start_pos = self.position;
        // TODO: Proper f-string support with expression interpolation
        // For now, treat as regular string and emit warning
        log::warn!("F-string support is incomplete at {}", start_pos);
        
        let quote = self.current_char.unwrap();
        let mut value = String::new();
        let mut lexeme = format!("f{}", quote);
        
        self.advance(); // Skip opening quote
        
        while let Some(c) = self.current_char {
            if c == quote {
                lexeme.push(c);
                self.advance();
                return Ok(Token::new(TokenKind::String(value), start_pos, lexeme));
            }
            
            lexeme.push(c);
            value.push(c);
            self.advance();
        }
        
        Err(MambaError::SyntaxError(format!(
            "Unterminated f-string at {}",
            start_pos
        )))
    }
}
