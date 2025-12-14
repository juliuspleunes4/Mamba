//! Recursive descent parser for Mamba
//!
//! Converts a stream of tokens into an Abstract Syntax Tree (AST).

use crate::ast::*;
use crate::token::{SourcePosition, Token, TokenKind};
use mamba_error::MambaError;
use std::iter::Peekable;
use std::vec::IntoIter;

type ParseResult<T> = Result<T, MambaError>;

/// Parser converts tokens into an AST
pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    current_token: Option<Token>,
}

impl Parser {
    /// Create a new parser from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut parser = Parser {
            tokens: tokens.into_iter().peekable(),
            current_token: None,
        };
        parser.advance(); // Load first token
        parser
    }

    /// Parse a complete module (list of statements)
    pub fn parse(&mut self) -> ParseResult<Module> {
        let start_pos = self.current_position();
        let mut statements = Vec::new();

        while !self.is_at_end() {
            // Skip newlines at module level
            if self.match_token(&TokenKind::Newline) {
                continue;
            }
            
            statements.push(self.parse_statement()?);
        }

        Ok(Module {
            statements,
            position: start_pos,
        })
    }

    /// Parse a single statement
    fn parse_statement(&mut self) -> ParseResult<Statement> {
        // For now, only parse expression statements
        // TODO: Add other statement types (if, while, def, etc.)
        
        match self.current_kind() {
            Some(TokenKind::Pass) => self.parse_pass(),
            Some(TokenKind::Break) => self.parse_break(),
            Some(TokenKind::Continue) => self.parse_continue(),
            Some(TokenKind::Return) => self.parse_return(),
            _ => {
                // Try to parse as assignment or expression
                let expr = self.parse_expression()?;
                
                // Check for assignment
                if self.match_token(&TokenKind::Assign) {
                    let value = self.parse_expression()?;
                    self.consume_newline_or_eof()?;
                    let pos = expr.position().clone();
                    return Ok(Statement::Assignment {
                        target: expr,
                        value,
                        position: pos,
                    });
                }
                
                // Check for augmented assignment
                if let Some(aug_op) = self.match_augmented_assign() {
                    let value = self.parse_expression()?;
                    self.consume_newline_or_eof()?;
                    let pos = expr.position().clone();
                    return Ok(Statement::AugmentedAssignment {
                        target: expr,
                        op: aug_op,
                        value,
                        position: pos,
                    });
                }
                
                // Otherwise it's an expression statement
                self.consume_newline_or_eof()?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    /// Parse pass statement
    fn parse_pass(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'pass'
        self.consume_newline_or_eof()?;
        Ok(Statement::Pass(pos))
    }

    /// Parse break statement
    fn parse_break(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'break'
        self.consume_newline_or_eof()?;
        Ok(Statement::Break(pos))
    }

    /// Parse continue statement
    fn parse_continue(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'continue'
        self.consume_newline_or_eof()?;
        Ok(Statement::Continue(pos))
    }

    /// Parse return statement
    fn parse_return(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'return'
        
        // Check if there's a value to return
        let value = if self.check(&TokenKind::Newline) || self.is_at_end() {
            None
        } else {
            Some(self.parse_expression()?)
        };
        
        self.consume_newline_or_eof()?;
        Ok(Statement::Return { value, position: pos })
    }

    /// Parse an expression with operator precedence
    fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_or()
    }

    /// Parse logical OR expression (lowest precedence)
    fn parse_or(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_and()?;

        while self.match_token(&TokenKind::Or) {
            let op_pos = self.previous_position();
            let right = self.parse_and()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::Or,
                right: Box::new(right),
                position: op_pos,
            };
        }

        Ok(left)
    }

    /// Parse logical AND expression
    fn parse_and(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_not()?;

        while self.match_token(&TokenKind::And) {
            let op_pos = self.previous_position();
            let right = self.parse_not()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::And,
                right: Box::new(right),
                position: op_pos,
            };
        }

        Ok(left)
    }

    /// Parse NOT expression (unary)
    fn parse_not(&mut self) -> ParseResult<Expression> {
        if self.match_token(&TokenKind::Not) {
            let op_pos = self.previous_position();
            let operand = self.parse_not()?;
            return Ok(Expression::UnaryOp {
                op: UnaryOperator::Not,
                operand: Box::new(operand),
                position: op_pos,
            });
        }

        self.parse_comparison()
    }

    /// Parse comparison expressions (==, !=, <, >, <=, >=, in, is)
    fn parse_comparison(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_bitwise_or()?;

        while let Some(op) = self.match_comparison_op() {
            let op_pos = self.previous_position();
            let right = self.parse_bitwise_or()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                position: op_pos,
            };
        }

        Ok(left)
    }

    /// Parse bitwise OR
    fn parse_bitwise_or(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_bitwise_xor()?;

        while self.match_token(&TokenKind::Pipe) {
            let op_pos = self.previous_position();
            let right = self.parse_bitwise_xor()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::BitwiseOr,
                right: Box::new(right),
                position: op_pos,
            };
        }

        Ok(left)
    }

    /// Parse bitwise XOR
    fn parse_bitwise_xor(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_bitwise_and()?;

        while self.match_token(&TokenKind::Caret) {
            let op_pos = self.previous_position();
            let right = self.parse_bitwise_and()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::BitwiseXor,
                right: Box::new(right),
                position: op_pos,
            };
        }

        Ok(left)
    }

    /// Parse bitwise AND
    fn parse_bitwise_and(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_shift()?;

        while self.match_token(&TokenKind::Ampersand) {
            let op_pos = self.previous_position();
            let right = self.parse_shift()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::BitwiseAnd,
                right: Box::new(right),
                position: op_pos,
            };
        }

        Ok(left)
    }

    /// Parse shift operations (<<, >>)
    fn parse_shift(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_addition()?;

        while let Some(op) = self.match_shift_op() {
            let op_pos = self.previous_position();
            let right = self.parse_addition()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                position: op_pos,
            };
        }

        Ok(left)
    }

    /// Parse addition and subtraction
    fn parse_addition(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_multiplication()?;

        while let Some(op) = self.match_add_sub_op() {
            let op_pos = self.previous_position();
            let right = self.parse_multiplication()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                position: op_pos,
            };
        }

        Ok(left)
    }

    /// Parse multiplication, division, and modulo
    fn parse_multiplication(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_unary()?;

        while let Some(op) = self.match_mul_div_mod_op() {
            let op_pos = self.previous_position();
            let right = self.parse_unary()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                position: op_pos,
            };
        }

        Ok(left)
    }

    /// Parse unary operations (-, +, ~)
    fn parse_unary(&mut self) -> ParseResult<Expression> {
        if let Some(op) = self.match_unary_op() {
            let op_pos = self.previous_position();
            let operand = self.parse_unary()?;
            return Ok(Expression::UnaryOp {
                op,
                operand: Box::new(operand),
                position: op_pos,
            });
        }

        self.parse_power()
    }

    /// Parse power operation (**)
    fn parse_power(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_postfix()?;

        if self.match_token(&TokenKind::DoubleStar) {
            let op_pos = self.previous_position();
            // Right-associative: a ** b ** c = a ** (b ** c)
            let right = self.parse_power()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::Power,
                right: Box::new(right),
                position: op_pos,
            };
        }

        Ok(left)
    }

    /// Parse postfix operations (function calls, attribute access, subscripts)
    fn parse_postfix(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.current_kind() {
                Some(TokenKind::LeftParen) => {
                    // Function call: func(args)
                    self.advance(); // consume '('
                    let mut arguments = Vec::new();
                    let call_pos = expr.position().clone();

                    // Parse arguments if not empty
                    if !self.check(&TokenKind::RightParen) {
                        loop {
                            arguments.push(self.parse_expression()?);
                            
                            if !self.match_token(&TokenKind::Comma) {
                                break;
                            }
                            
                            // Allow trailing comma
                            if self.check(&TokenKind::RightParen) {
                                break;
                            }
                        }
                    }

                    self.expect_token(TokenKind::RightParen, "Expected ')' after arguments")?;
                    
                    expr = Expression::Call {
                        function: Box::new(expr),
                        arguments,
                        position: call_pos,
                    };
                }
                Some(TokenKind::LeftBracket) => {
                    // Subscript: obj[index]
                    self.advance(); // consume '['
                    let subscript_pos = expr.position().clone();
                    
                    let index = self.parse_expression()?;
                    
                    self.expect_token(TokenKind::RightBracket, "Expected ']' after subscript index")?;
                    
                    expr = Expression::Subscript {
                        object: Box::new(expr),
                        index: Box::new(index),
                        position: subscript_pos,
                    };
                }
                Some(TokenKind::Dot) => {
                    // Attribute access: obj.attr
                    self.advance(); // consume '.'
                    let attr_pos = expr.position().clone();
                    
                    // Expect identifier after dot
                    match self.current_kind() {
                        Some(TokenKind::Identifier(name)) => {
                            let attr_name = name.clone();
                            self.advance();
                            
                            expr = Expression::Attribute {
                                object: Box::new(expr),
                                attribute: attr_name,
                                position: attr_pos,
                            };
                        }
                        _ => {
                            return Err(MambaError::ParseError(
                                format!("Expected identifier after '.' at {}", self.current_position())
                            ));
                        }
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parse primary expressions (literals, identifiers, parenthesized)
    fn parse_primary(&mut self) -> ParseResult<Expression> {
        match self.current_kind() {
            Some(TokenKind::Integer(value)) => {
                let pos = self.current_position();
                let val = *value;
                self.advance();
                Ok(Expression::Literal(Literal::Integer {
                    value: val,
                    position: pos,
                }))
            }
            Some(TokenKind::Float(value)) => {
                let pos = self.current_position();
                let val = *value;
                self.advance();
                Ok(Expression::Literal(Literal::Float {
                    value: val,
                    position: pos,
                }))
            }
            Some(TokenKind::String(value)) => {
                let pos = self.current_position();
                let val = value.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String {
                    value: val,
                    position: pos,
                }))
            }
            Some(TokenKind::True) => {
                let pos = self.current_position();
                self.advance();
                Ok(Expression::Literal(Literal::Boolean {
                    value: true,
                    position: pos,
                }))
            }
            Some(TokenKind::False) => {
                let pos = self.current_position();
                self.advance();
                Ok(Expression::Literal(Literal::Boolean {
                    value: false,
                    position: pos,
                }))
            }
            Some(TokenKind::None) => {
                let pos = self.current_position();
                self.advance();
                Ok(Expression::Literal(Literal::None { position: pos }))
            }
            Some(TokenKind::Identifier(name)) => {
                let pos = self.current_position();
                let name_str = name.clone();
                self.advance();
                Ok(Expression::Identifier {
                    name: name_str,
                    position: pos,
                })
            }
            Some(TokenKind::LeftParen) => {
                // Could be: parenthesized expression (x), empty tuple (), or tuple (x,) or (x, y)
                let pos = self.current_position();
                self.advance(); // consume '('
                
                // Check for empty tuple
                if self.check(&TokenKind::RightParen) {
                    self.advance(); // consume ')'
                    return Ok(Expression::Tuple {
                        elements: Vec::new(),
                        position: pos,
                    });
                }
                
                // Parse first expression
                let first_expr = self.parse_expression()?;
                
                // Check if it's a tuple (has comma) or parenthesized expression
                if self.match_token(&TokenKind::Comma) {
                    // It's a tuple
                    let mut elements = vec![first_expr];
                    
                    // Parse remaining elements if not at closing paren
                    if !self.check(&TokenKind::RightParen) {
                        loop {
                            elements.push(self.parse_expression()?);
                            
                            if !self.match_token(&TokenKind::Comma) {
                                break;
                            }
                            
                            // Allow trailing comma
                            if self.check(&TokenKind::RightParen) {
                                break;
                            }
                        }
                    }
                    
                    self.expect_token(TokenKind::RightParen, "Expected ')' after tuple elements")?;
                    
                    Ok(Expression::Tuple {
                        elements,
                        position: pos,
                    })
                } else {
                    // It's a parenthesized expression
                    self.expect_token(TokenKind::RightParen, "Expected ')' after expression")?;
                    Ok(Expression::Parenthesized {
                        expr: Box::new(first_expr),
                        position: pos,
                    })
                }
            }
            Some(TokenKind::LeftBracket) => {
                // List literal: [1, 2, 3]
                let pos = self.current_position();
                self.advance(); // consume '['
                let mut elements = Vec::new();

                // Parse elements if not empty
                if !self.check(&TokenKind::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                        
                        // Allow trailing comma
                        if self.check(&TokenKind::RightBracket) {
                            break;
                        }
                    }
                }

                self.expect_token(TokenKind::RightBracket, "Expected ']' after list elements")?;
                
                Ok(Expression::List {
                    elements,
                    position: pos,
                })
            }
            Some(TokenKind::LeftBrace) => {
                // Dict literal: {key: value, ...}
                let pos = self.current_position();
                self.advance(); // consume '{'
                let mut pairs = Vec::new();

                // Parse pairs if not empty
                if !self.check(&TokenKind::RightBrace) {
                    loop {
                        // Parse key
                        let key = self.parse_expression()?;
                        
                        // Expect colon
                        self.expect_token(TokenKind::Colon, "Expected ':' after dict key")?;
                        
                        // Parse value
                        let value = self.parse_expression()?;
                        
                        pairs.push((key, value));
                        
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                        
                        // Allow trailing comma
                        if self.check(&TokenKind::RightBrace) {
                            break;
                        }
                    }
                }

                self.expect_token(TokenKind::RightBrace, "Expected '}' after dict pairs")?;
                
                Ok(Expression::Dict {
                    pairs,
                    position: pos,
                })
            }
            _ => Err(MambaError::ParseError(format!(
                "Unexpected token at {}:{}: {:?}",
                self.current_position().line,
                self.current_position().column,
                self.current_token.as_ref().map(|t| &t.kind)
            ))),
        }
    }

    // ===== Helper methods =====

    /// Check if current token matches the given kind
    fn check(&self, kind: &TokenKind) -> bool {
        self.current_token
            .as_ref()
            .map(|t| &t.kind == kind)
            .unwrap_or(false)
    }

    /// Match and consume token if it matches
    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Match comparison operators
    fn match_comparison_op(&mut self) -> Option<BinaryOperator> {
        // Handle compound operators first (not in, is not)
        match self.current_kind() {
            Some(TokenKind::Not) => {
                // Peek ahead to see if this is "not in"
                if let Some(next_token) = self.tokens.peek() {
                    if next_token.kind == TokenKind::In {
                        self.advance(); // consume "not"
                        self.advance(); // consume "in"
                        return Some(BinaryOperator::NotIn);
                    }
                }
                // Not a comparison operator
                return None;
            }
            Some(TokenKind::Is) => {
                // Peek ahead to see if this is "is not"
                if let Some(next_token) = self.tokens.peek() {
                    if next_token.kind == TokenKind::Not {
                        self.advance(); // consume "is"
                        self.advance(); // consume "not"
                        return Some(BinaryOperator::IsNot);
                    }
                }
                // Just "is"
                self.advance();
                return Some(BinaryOperator::Is);
            }
            _ => {}
        }
        
        // Handle simple operators
        let op = match self.current_kind() {
            Some(TokenKind::Equal) => BinaryOperator::Equal,
            Some(TokenKind::NotEqual) => BinaryOperator::NotEqual,
            Some(TokenKind::Less) => BinaryOperator::LessThan,
            Some(TokenKind::LessEqual) => BinaryOperator::LessThanEq,
            Some(TokenKind::Greater) => BinaryOperator::GreaterThan,
            Some(TokenKind::GreaterEqual) => BinaryOperator::GreaterThanEq,
            Some(TokenKind::In) => BinaryOperator::In,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    /// Match shift operators
    fn match_shift_op(&mut self) -> Option<BinaryOperator> {
        let op = match self.current_kind() {
            Some(TokenKind::LeftShift) => BinaryOperator::LeftShift,
            Some(TokenKind::RightShift) => BinaryOperator::RightShift,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    /// Match addition/subtraction operators
    fn match_add_sub_op(&mut self) -> Option<BinaryOperator> {
        let op = match self.current_kind() {
            Some(TokenKind::Plus) => BinaryOperator::Add,
            Some(TokenKind::Minus) => BinaryOperator::Subtract,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    /// Match multiplication/division/modulo operators
    fn match_mul_div_mod_op(&mut self) -> Option<BinaryOperator> {
        let op = match self.current_kind() {
            Some(TokenKind::Star) => BinaryOperator::Multiply,
            Some(TokenKind::Slash) => BinaryOperator::Divide,
            Some(TokenKind::DoubleSlash) => BinaryOperator::FloorDivide,
            Some(TokenKind::Percent) => BinaryOperator::Modulo,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    /// Match unary operators
    fn match_unary_op(&mut self) -> Option<UnaryOperator> {
        let op = match self.current_kind() {
            Some(TokenKind::Minus) => UnaryOperator::Minus,
            Some(TokenKind::Plus) => UnaryOperator::Plus,
            Some(TokenKind::Tilde) => UnaryOperator::BitwiseNot,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    /// Match augmented assignment operators
    fn match_augmented_assign(&mut self) -> Option<AugmentedOperator> {
        let op = match self.current_kind() {
            Some(TokenKind::PlusAssign) => AugmentedOperator::Add,
            Some(TokenKind::MinusAssign) => AugmentedOperator::Subtract,
            Some(TokenKind::StarAssign) => AugmentedOperator::Multiply,
            Some(TokenKind::SlashAssign) => AugmentedOperator::Divide,
            Some(TokenKind::DoubleSlashAssign) => AugmentedOperator::FloorDivide,
            Some(TokenKind::PercentAssign) => AugmentedOperator::Modulo,
            Some(TokenKind::DoubleStarAssign) => AugmentedOperator::Power,
            Some(TokenKind::AmpersandAssign) => AugmentedOperator::BitwiseAnd,
            Some(TokenKind::PipeAssign) => AugmentedOperator::BitwiseOr,
            Some(TokenKind::CaretAssign) => AugmentedOperator::BitwiseXor,
            Some(TokenKind::LeftShiftAssign) => AugmentedOperator::LeftShift,
            Some(TokenKind::RightShiftAssign) => AugmentedOperator::RightShift,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    /// Expect a specific token and consume it
    fn expect_token(&mut self, kind: TokenKind, error_msg: &str) -> ParseResult<()> {
        if self.check(&kind) {
            self.advance();
            Ok(())
        } else {
            Err(MambaError::ParseError(format!(
                "{} at {}:{}",
                error_msg,
                self.current_position().line,
                self.current_position().column
            )))
        }
    }

    /// Consume newline or EOF (required at end of statements)
    fn consume_newline_or_eof(&mut self) -> ParseResult<()> {
        if self.is_at_end() {
            Ok(())
        } else if self.match_token(&TokenKind::Newline) {
            Ok(())
        } else {
            Err(MambaError::ParseError(format!(
                "Expected newline or end of file at {}:{}",
                self.current_position().line,
                self.current_position().column
            )))
        }
    }

    /// Advance to next token
    fn advance(&mut self) {
        self.current_token = self.tokens.next();
    }

    /// Get current token kind
    fn current_kind(&self) -> Option<&TokenKind> {
        self.current_token.as_ref().map(|t| &t.kind)
    }

    /// Get current token position
    fn current_position(&self) -> SourcePosition {
        self.current_token
            .as_ref()
            .map(|t| t.position.clone())
            .unwrap_or_else(|| SourcePosition::new(0, 0, 0))
    }

    /// Get previous token position (after advance)
    fn previous_position(&self) -> SourcePosition {
        // This is a simplification; ideally we'd track the previous token
        self.current_position()
    }

    /// Check if we're at end of file
    fn is_at_end(&self) -> bool {
        matches!(
            self.current_kind(),
            Some(TokenKind::Eof) | None
        )
    }
}
