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
    previous_position: SourcePosition,
}

impl Parser {
    /// Create a new parser from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut parser = Parser {
            tokens: tokens.into_iter().peekable(),
            current_token: None,
            previous_position: SourcePosition::new(0, 0, 0),
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
        // TODO: Add support for remaining statement types: if, while, for, def, class, import, etc.
        
        match self.current_kind() {
            Some(TokenKind::Pass) => self.parse_pass(),
            Some(TokenKind::Break) => self.parse_break(),
            Some(TokenKind::Continue) => self.parse_continue(),
            Some(TokenKind::Return) => self.parse_return(),
            Some(TokenKind::Assert) => self.parse_assert(),
            Some(TokenKind::Del) => self.parse_del(),
            Some(TokenKind::Global) => self.parse_global(),
            Some(TokenKind::Nonlocal) => self.parse_nonlocal(),
            Some(TokenKind::Raise) => self.parse_raise(),
            Some(TokenKind::Import) => self.parse_import(),
            Some(TokenKind::From) => self.parse_from_import(),
            Some(TokenKind::If) => self.parse_if(),
            Some(TokenKind::While) => self.parse_while(),
            Some(TokenKind::For) => self.parse_for(),
            Some(TokenKind::Def) => self.parse_function_def(),
            Some(TokenKind::Class) => self.parse_class_def(),
            _ => {
                // Try to parse as assignment or expression
                let expr = self.parse_assignment_target()?;
                
                // Check for comma (tuple without parentheses) - this could be unpacking
                if self.check(&TokenKind::Comma) {
                    // Build a tuple from comma-separated targets
                    let mut elements = vec![expr];
                    let pos = elements[0].position().clone();
                    
                    while self.match_token(&TokenKind::Comma) {
                        // Allow trailing comma before assignment or newline
                        if self.check(&TokenKind::Assign) || self.check(&TokenKind::Newline) || self.is_at_end() {
                            break;
                        }
                        elements.push(self.parse_assignment_target()?);
                    }
                    
                    // Create tuple expression
                    let tuple_expr = Expression::Tuple {
                        elements: elements.clone(),
                        position: pos.clone(),
                    };
                    
                    // Now check for assignment
                    if self.match_token(&TokenKind::Assign) {
                        // Validate starred expressions in unpacking
                        self.validate_unpacking_targets(&elements)?;
                        
                        let value = self.parse_tuple_or_expression()?;
                        self.consume_newline_or_eof()?;
                        return Ok(Statement::Assignment {
                            targets: vec![tuple_expr],
                            value,
                            position: pos,
                        });
                    }
                    
                    // Otherwise it's an expression statement (tuple)
                    self.consume_newline_or_eof()?;
                    return Ok(Statement::Expression(tuple_expr));
                }
                
                // Check for assignment (including chained assignments like x = y = 5)
                if self.match_token(&TokenKind::Assign) {
                    let mut targets = vec![expr.clone()];
                    let pos = targets[0].position().clone();
                    
                    // Validate starred expressions in the first target
                    self.validate_single_target(&expr)?;
                    
                    // Parse chained assignments: x = y = z = value
                    loop {
                        let next_expr = self.parse_tuple_or_expression()?;
                        
                        if self.match_token(&TokenKind::Assign) {
                            // More assignments coming - validate this target too
                            self.validate_single_target(&next_expr)?;
                            targets.push(next_expr);
                        } else {
                            // This is the final value
                            self.consume_newline_or_eof()?;
                            return Ok(Statement::Assignment {
                                targets,
                                value: next_expr,
                                position: pos,
                            });
                        }
                    }
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

    /// Parse assert statement (assert condition, optional_message)
    fn parse_assert(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'assert'
        
        // Parse the condition
        let condition = self.parse_expression()?;
        
        // Check for optional message after comma
        let message = if self.match_token(&TokenKind::Comma) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.consume_newline_or_eof()?;
        Ok(Statement::Assert {
            condition,
            message,
            position: pos,
        })
    }

    /// Parse del statement (del x or del x, y, z)
    fn parse_del(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'del'
        
        // Parse comma-separated targets
        let mut targets = vec![self.parse_expression()?];
        
        while self.match_token(&TokenKind::Comma) {
            // Allow trailing comma
            if self.check(&TokenKind::Newline) || self.is_at_end() {
                break;
            }
            targets.push(self.parse_expression()?);
        }
        
        self.consume_newline_or_eof()?;
        Ok(Statement::Del {
            targets,
            position: pos,
        })
    }

    /// Parse global statement (global x, y, z)
    fn parse_global(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'global'
        
        let names = self.parse_name_list("global")?;
        self.consume_newline_or_eof()?;
        
        Ok(Statement::Global {
            names,
            position: pos,
        })
    }

    /// Parse nonlocal statement (nonlocal x, y, z)
    fn parse_nonlocal(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'nonlocal'
        
        let names = self.parse_name_list("nonlocal")?;
        self.consume_newline_or_eof()?;
        
        Ok(Statement::Nonlocal {
            names,
            position: pos,
        })
    }

    /// Helper function to parse comma-separated identifier names
    /// Used by global and nonlocal statements
    fn parse_name_list(&mut self, keyword: &str) -> ParseResult<Vec<String>> {
        let mut names = Vec::new();
        
        loop {
            match self.current_kind() {
                Some(TokenKind::Identifier(name)) => {
                    names.push(name.clone());
                    self.advance();
                }
                _ => {
                    // Provide more specific error message if no names were parsed yet
                    if names.is_empty() {
                        return Err(MambaError::ParseError(
                            format!("Expected at least one identifier after '{}' at {}:{}", 
                                keyword,
                                self.current_position().line, 
                                self.current_position().column)
                        ));
                    } else {
                        return Err(MambaError::ParseError(
                            format!("Expected identifier after '{}' at {}:{}", 
                                keyword,
                                self.current_position().line, 
                                self.current_position().column)
                        ));
                    }
                }
            }
            
            // Check for comma
            if self.match_token(&TokenKind::Comma) {
                // Allow trailing comma
                if self.check(&TokenKind::Newline) || self.is_at_end() {
                    break;
                }
                continue;
            } else {
                break;
            }
        }
        
        Ok(names)
    }

    /// Parse raise statement (raise, raise Exception, raise Exception("msg"))
    fn parse_raise(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'raise'
        
        // Check if there's an exception expression
        let exception = if self.check(&TokenKind::Newline) || self.is_at_end() {
            // Bare raise (re-raises current exception)
            None
        } else {
            // Parse exception expression
            Some(self.parse_expression()?)
        };
        
        self.consume_newline_or_eof()?;
        Ok(Statement::Raise {
            exception,
            position: pos,
        })
    }

    /// Parse import statement (import module, import module.submodule as alias)
    fn parse_import(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'import'
        
        let mut items = Vec::new();
        
        loop {
            // Parse module name (possibly dotted, like os.path)
            let item_pos = self.current_position();
            let module = self.parse_dotted_name("import")?;
            
            // Check for optional 'as' alias
            let alias = if self.match_token(&TokenKind::As) {
                match self.current_kind() {
                    Some(TokenKind::Identifier(name)) => {
                        let alias_name = name.clone();
                        self.advance();
                        Some(alias_name)
                    }
                    _ => {
                        return Err(MambaError::ParseError(
                            format!("Expected identifier after 'as' at {}:{}", 
                                self.current_position().line, 
                                self.current_position().column)
                        ));
                    }
                }
            } else {
                None
            };
            
            items.push(ImportItem {
                module,
                alias,
                position: item_pos,
            });
            
            // Check for comma (multiple imports)
            if self.match_token(&TokenKind::Comma) {
                // Allow trailing comma
                if self.check(&TokenKind::Newline) || self.is_at_end() {
                    break;
                }
                continue;
            } else {
                break;
            }
        }
        
        self.consume_newline_or_eof()?;
        Ok(Statement::Import {
            items,
            position: pos,
        })
    }

    /// Parse a dotted module name (e.g., "os", "os.path", "package.submodule")
    /// The `context` parameter is used for error messages (e.g., "import" or "from")
    fn parse_dotted_name(&mut self, context: &str) -> ParseResult<String> {
        let mut parts = Vec::new();
        
        // Parse first identifier
        match self.current_kind() {
            Some(TokenKind::Identifier(name)) => {
                parts.push(name.clone());
                self.advance();
            }
            _ => {
                return Err(MambaError::ParseError(
                    format!("Expected module name after '{}' at {}:{}", 
                        context,
                        self.current_position().line, 
                        self.current_position().column)
                ));
            }
        }
        
        // Parse subsequent parts separated by dots
        while self.match_token(&TokenKind::Dot) {
            match self.current_kind() {
                Some(TokenKind::Identifier(name)) => {
                    parts.push(name.clone());
                    self.advance();
                }
                _ => {
                    return Err(MambaError::ParseError(
                        format!("Expected identifier after '.' in module name at {}:{}", 
                            self.current_position().line, 
                            self.current_position().column)
                    ));
                }
            }
        }
        
        Ok(parts.join("."))
    }

    /// Parse from...import statement (from module import name, from module import *)
    fn parse_from_import(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'from'
        
        // Parse module name (possibly dotted, like os.path)
        let module = self.parse_dotted_name("from")?;
        
        // Expect 'import' keyword
        if !self.match_token(&TokenKind::Import) {
            return Err(MambaError::ParseError(
                format!("Expected 'import' after module name in from...import statement at {}:{}", 
                    self.current_position().line, 
                    self.current_position().column)
            ));
        }
        
        let mut items = Vec::new();
        
        // Check for wildcard import
        if self.match_token(&TokenKind::Star) {
            // Wildcard import: from module import *
            let wildcard_pos = self.previous_position();
            
            // Wildcard can't have an alias
            if self.check(&TokenKind::As) {
                return Err(MambaError::ParseError(
                    format!("Wildcard import cannot have an alias at {}:{}", 
                        self.current_position().line, 
                        self.current_position().column)
                ));
            }
            
            // Wildcard must be alone (no comma-separated names)
            if self.check(&TokenKind::Comma) {
                return Err(MambaError::ParseError(
                    format!("Wildcard import cannot be combined with other imports at {}:{}", 
                        self.current_position().line, 
                        self.current_position().column)
                ));
            }
            
            items.push(FromImportItem {
                name: "*".to_string(),
                alias: None,
                position: wildcard_pos,
            });
        } else {
            // Named imports: from module import name1, name2, ...
            loop {
                let item_pos = self.current_position();
                
                // Parse imported name
                let name = match self.current_kind() {
                    Some(TokenKind::Identifier(n)) => {
                        let name_str = n.clone();
                        self.advance();
                        name_str
                    }
                    _ => {
                        return Err(MambaError::ParseError(
                            format!("Expected identifier after 'import' at {}:{}", 
                                self.current_position().line, 
                                self.current_position().column)
                        ));
                    }
                };
                
                // Check for optional 'as' alias
                let alias = if self.match_token(&TokenKind::As) {
                    match self.current_kind() {
                        Some(TokenKind::Identifier(a)) => {
                            let alias_name = a.clone();
                            self.advance();
                            Some(alias_name)
                        }
                        _ => {
                            return Err(MambaError::ParseError(
                                format!("Expected identifier after 'as' at {}:{}", 
                                    self.current_position().line, 
                                    self.current_position().column)
                            ));
                        }
                    }
                } else {
                    None
                };
                
                items.push(FromImportItem {
                    name,
                    alias,
                    position: item_pos,
                });
                
                // Check for comma (multiple imports)
                if self.match_token(&TokenKind::Comma) {
                    // Allow trailing comma
                    if self.check(&TokenKind::Newline) || self.is_at_end() {
                        break;
                    }
                    continue;
                } else {
                    break;
                }
            }
        }
        
        self.consume_newline_or_eof()?;
        Ok(Statement::FromImport {
            module,
            items,
            position: pos,
        })
    }

    /// Parse if statement with optional elif and else blocks
    fn parse_if(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'if'
        
        // Parse condition
        let condition = self.parse_expression()?;
        
        // Expect colon
        if !self.match_token(&TokenKind::Colon) {
            return Err(MambaError::ParseError(
                format!("Expected ':' after if condition at {}:{}", 
                    self.current_position().line, 
                    self.current_position().column)
            ));
        }
        
        // Parse then block
        let then_block = self.parse_block()?;
        
        // Parse elif blocks (zero or more)
        let mut elif_blocks = Vec::new();
        while self.check(&TokenKind::Elif) {
            self.advance(); // consume 'elif'
            
            let elif_condition = self.parse_expression()?;
            
            if !self.match_token(&TokenKind::Colon) {
                return Err(MambaError::ParseError(
                    format!("Expected ':' after elif condition at {}:{}", 
                        self.current_position().line, 
                        self.current_position().column)
                ));
            }
            
            let elif_body = self.parse_block()?;
            elif_blocks.push((elif_condition, elif_body));
        }
        
        // Parse optional else block
        let else_block = if self.match_token(&TokenKind::Else) {
            if !self.match_token(&TokenKind::Colon) {
                return Err(MambaError::ParseError(
                    format!("Expected ':' after else at {}:{}", 
                        self.current_position().line, 
                        self.current_position().column)
                ));
            }
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(Statement::If {
            condition,
            then_block,
            elif_blocks,
            else_block,
            position: pos,
        })
    }

    /// Parse while loop with optional else block
    fn parse_while(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'while'
        
        // Parse condition
        let condition = self.parse_expression()?;
        
        // Expect colon
        if !self.match_token(&TokenKind::Colon) {
            return Err(MambaError::ParseError(
                format!("Expected ':' after while condition at {}:{}", 
                    self.current_position().line, 
                    self.current_position().column)
            ));
        }
        
        // Parse body
        let body = self.parse_block()?;
        
        // Parse optional else block
        let else_block = if self.match_token(&TokenKind::Else) {
            if !self.match_token(&TokenKind::Colon) {
                return Err(MambaError::ParseError(
                    format!("Expected ':' after else at {}:{}", 
                        self.current_position().line, 
                        self.current_position().column)
                ));
            }
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(Statement::While {
            condition,
            body,
            else_block,
            position: pos,
        })
    }

    /// Parse for loop with optional else block
    fn parse_for(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'for'
        
        // Parse target (loop variable) - can be identifier or tuple unpacking
        // We need to be careful not to parse 'in' as part of the target
        let target = self.parse_for_target()?;
        
        // Expect 'in' keyword
        if !self.match_token(&TokenKind::In) {
            return Err(MambaError::ParseError(
                format!("Expected 'in' after for target at {}:{}", 
                    self.current_position().line, 
                    self.current_position().column)
            ));
        }
        
        // Parse iterable expression
        let iter = self.parse_expression()?;
        
        // Expect colon
        if !self.match_token(&TokenKind::Colon) {
            return Err(MambaError::ParseError(
                format!("Expected ':' after for clause at {}:{}", 
                    self.current_position().line, 
                    self.current_position().column)
            ));
        }
        
        // Parse body
        let body = self.parse_block()?;
        
        // Parse optional else block
        let else_block = if self.match_token(&TokenKind::Else) {
            if !self.match_token(&TokenKind::Colon) {
                return Err(MambaError::ParseError(
                    format!("Expected ':' after else at {}:{}", 
                        self.current_position().line, 
                        self.current_position().column)
                ));
            }
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(Statement::For {
            target,
            iter,
            body,
            else_block,
            position: pos,
        })
    }

    /// Parse for loop target (identifier or tuple unpacking, but not full expression)
    fn parse_for_target(&mut self) -> ParseResult<Expression> {
        let start_pos = self.current_position();
        
        // Parse first identifier
        let first = match self.current_kind() {
            Some(TokenKind::Identifier(name)) => {
                let id = Expression::Identifier {
                    name: name.clone(),
                    position: self.current_position(),
                };
                self.advance();
                id
            }
            _ => {
                return Err(MambaError::ParseError(
                    format!("Expected identifier in for target at {}:{}", 
                        self.current_position().line, 
                        self.current_position().column)
                ));
            }
        };
        
        // Check for comma (tuple unpacking)
        if self.match_token(&TokenKind::Comma) {
            let mut elements = vec![first];
            
            // Parse remaining identifiers
            loop {
                // Allow trailing comma before 'in'
                if self.check(&TokenKind::In) {
                    break;
                }
                
                match self.current_kind() {
                    Some(TokenKind::Identifier(name)) => {
                        elements.push(Expression::Identifier {
                            name: name.clone(),
                            position: self.current_position(),
                        });
                        self.advance();
                    }
                    _ => {
                        return Err(MambaError::ParseError(
                            format!("Expected identifier in for target at {}:{}", 
                                self.current_position().line, 
                                self.current_position().column)
                        ));
                    }
                }
                
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            
            Ok(Expression::Tuple {
                elements,
                position: start_pos,
            })
        } else {
            Ok(first)
        }
    }

    /// Parse function definition (def name(params): body)
    fn parse_function_def(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'def'
        
        // Parse function name
        let name = match self.current_kind() {
            Some(TokenKind::Identifier(n)) => {
                let func_name = n.clone();
                self.advance();
                func_name
            }
            _ => {
                return Err(MambaError::ParseError(
                    format!("Expected function name after 'def' at {}:{}", 
                        self.current_position().line, 
                        self.current_position().column)
                ));
            }
        };
        
        // Expect opening parenthesis
        if !self.match_token(&TokenKind::LeftParen) {
            return Err(MambaError::ParseError(
                format!("Expected '(' after function name at {}:{}", 
                    self.current_position().line, 
                    self.current_position().column)
            ));
        }
        
        // Parse parameter list
        let parameters = self.parse_parameter_list()?;
        
        // Expect closing parenthesis
        if !self.match_token(&TokenKind::RightParen) {
            return Err(MambaError::ParseError(
                format!("Expected ')' after parameters at {}:{}", 
                    self.current_position().line, 
                    self.current_position().column)
            ));
        }
        
        // Expect colon
        if !self.match_token(&TokenKind::Colon) {
            return Err(MambaError::ParseError(
                format!("Expected ':' after function signature at {}:{}", 
                    self.current_position().line, 
                    self.current_position().column)
            ));
        }
        
        // Parse body
        let body = self.parse_block()?;
        
        Ok(Statement::FunctionDef {
            name,
            parameters,
            body,
            position: pos,
        })
    }

    /// Parse class definition (class Name[(bases)]: body)
    fn parse_class_def(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'class'
        
        // Parse class name
        let name = match self.current_kind() {
            Some(TokenKind::Identifier(n)) => {
                let class_name = n.clone();
                self.advance();
                class_name
            }
            _ => {
                return Err(MambaError::ParseError(
                    format!("Expected class name after 'class' at {}:{}", 
                        self.current_position().line, 
                        self.current_position().column)
                ));
            }
        };
        
        // Parse optional base classes (inheritance)
        let bases = if self.match_token(&TokenKind::LeftParen) {
            let mut base_list = Vec::new();
            
            // Check for empty parentheses
            if !self.check(&TokenKind::RightParen) {
                loop {
                    // Parse base class expression (identifier or attribute access)
                    let base = self.parse_expression()?;
                    base_list.push(base);
                    
                    // Check for comma (more base classes)
                    if self.match_token(&TokenKind::Comma) {
                        // Allow trailing comma
                        if self.check(&TokenKind::RightParen) {
                            break;
                        }
                        continue;
                    } else {
                        break;
                    }
                }
            }
            
            // Expect closing parenthesis
            if !self.match_token(&TokenKind::RightParen) {
                return Err(MambaError::ParseError(
                    format!("Expected ')' after base classes at {}:{}", 
                        self.current_position().line, 
                        self.current_position().column)
                ));
            }
            
            base_list
        } else {
            Vec::new()
        };
        
        // Expect colon
        if !self.match_token(&TokenKind::Colon) {
            return Err(MambaError::ParseError(
                format!("Expected ':' after class header at {}:{}", 
                    self.current_position().line, 
                    self.current_position().column)
            ));
        }
        
        // Parse body
        let body = self.parse_block()?;
        
        Ok(Statement::ClassDef {
            name,
            bases,
            body,
            position: pos,
        })
    }

    /// Parse parameter list inside function definition
    fn parse_parameter_list(&mut self) -> ParseResult<Vec<Parameter>> {
        let mut parameters = Vec::new();
        let mut seen_slash = false;              // Have we seen / marker?
        let mut seen_varargs_or_bare_star = false;
        let mut seen_varkwargs = false;
        let mut seen_default = false;
        let mut in_kwonly_section = false;
        
        // Check for empty parameter list
        if self.check(&TokenKind::RightParen) {
            return Ok(parameters);
        }
        
        loop {
            let param_pos = self.current_position();
            
            // Check for / (positional-only marker)
            if self.match_token(&TokenKind::Slash) {
                if seen_slash {
                    return Err(MambaError::ParseError(
                        format!("Duplicate '/' parameter at {}:{}", 
                            param_pos.line, param_pos.column)
                    ));
                }
                if seen_varargs_or_bare_star {
                    return Err(MambaError::ParseError(
                        format!("'/' must come before '*' or '*args' at {}:{}", 
                            param_pos.line, param_pos.column)
                    ));
                }
                if seen_varkwargs {
                    return Err(MambaError::ParseError(
                        format!("'/' must come before '**kwargs' at {}:{}", 
                            param_pos.line, param_pos.column)
                    ));
                }
                
                // Mark all previous parameters as positional-only
                for param in &mut parameters {
                    if matches!(param.kind, ParameterKind::Regular) {
                        param.kind = ParameterKind::PositionalOnly;
                    }
                }
                
                seen_slash = true;
                seen_default = false; // Reset default tracking after /
            }
            // Check for **kwargs
            else if self.match_token(&TokenKind::DoubleStar) {
                if seen_varkwargs {
                    return Err(MambaError::ParseError(
                        format!("Duplicate **kwargs parameter at {}:{}", 
                            param_pos.line, param_pos.column)
                    ));
                }
                
                let param_name = match self.current_kind() {
                    Some(TokenKind::Identifier(n)) => {
                        let name = n.clone();
                        self.advance();
                        name
                    }
                    _ => {
                        return Err(MambaError::ParseError(
                            format!("Expected parameter name after '**' at {}:{}", 
                                self.current_position().line, 
                                self.current_position().column)
                        ));
                    }
                };
                
                parameters.push(Parameter {
                    name: param_name,
                    kind: ParameterKind::VarKwargs,
                    default: None,
                    position: param_pos,
                });
                
                seen_varkwargs = true;
            }
            // Check for * (either *args or bare * for keyword-only)
            else if self.match_token(&TokenKind::Star) {
                if seen_varargs_or_bare_star {
                    return Err(MambaError::ParseError(
                        format!("Duplicate * or *args parameter at {}:{}", 
                            param_pos.line, param_pos.column)
                    ));
                }
                if seen_varkwargs {
                    return Err(MambaError::ParseError(
                        format!("* or *args must come before **kwargs at {}:{}", 
                            param_pos.line, param_pos.column)
                    ));
                }
                
                // Check if this is bare * (keyword-only marker) or *args
                if self.check(&TokenKind::Comma) || self.check(&TokenKind::RightParen) {
                    // Bare * - marks start of keyword-only parameters
                    in_kwonly_section = true;
                    seen_varargs_or_bare_star = true;
                } else {
                    // This is *args
                    let param_name = match self.current_kind() {
                        Some(TokenKind::Identifier(n)) => {
                            let name = n.clone();
                            self.advance();
                            name
                        }
                        _ => {
                            return Err(MambaError::ParseError(
                                format!("Expected parameter name after '*' at {}:{}", 
                                    self.current_position().line, 
                                    self.current_position().column)
                            ));
                        }
                    };
                    
                    parameters.push(Parameter {
                        name: param_name,
                        kind: ParameterKind::VarArgs,
                        default: None,
                        position: param_pos,
                    });
                    
                    seen_varargs_or_bare_star = true;
                    in_kwonly_section = true; // Parameters after *args are keyword-only
                }
            }
            // Regular, positional-only, or keyword-only parameter
            else {
                if seen_varkwargs {
                    return Err(MambaError::ParseError(
                        format!("Parameter cannot appear after **kwargs at {}:{}", 
                            param_pos.line, param_pos.column)
                    ));
                }
                
                let param_name = match self.current_kind() {
                    Some(TokenKind::Identifier(n)) => {
                        let name = n.clone();
                        self.advance();
                        name
                    }
                    _ => {
                        return Err(MambaError::ParseError(
                            format!("Expected parameter name at {}:{}", 
                                self.current_position().line, 
                                self.current_position().column)
                        ));
                    }
                };
                
                // Check for default value (=)
                let default = if self.match_token(&TokenKind::Assign) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                
                // Determine parameter kind
                let kind = if in_kwonly_section {
                    ParameterKind::KwOnly
                } else {
                    // Before / or after /: Regular (will be marked PositionalOnly if before /)
                    // Regular parameter validation: no default → default order
                    if default.is_some() {
                        seen_default = true;
                    } else if seen_default {
                        return Err(MambaError::ParseError(
                            format!("Parameter without default cannot follow parameter with default at {}:{}", 
                                param_pos.line, param_pos.column)
                        ));
                    }
                    ParameterKind::Regular
                };
                
                parameters.push(Parameter {
                    name: param_name,
                    kind,
                    default,
                    position: param_pos,
                });
            }
            
            // Check for comma (more parameters)
            if self.match_token(&TokenKind::Comma) {
                // Allow trailing comma
                if self.check(&TokenKind::RightParen) {
                    break;
                }
                continue;
            } else {
                break;
            }
        }
        
        Ok(parameters)
    }
    /// Parse an indented block of statements (INDENT ... DEDENT)
    fn parse_block(&mut self) -> ParseResult<Vec<Statement>> {
        // Consume newline after colon
        if !self.match_token(&TokenKind::Newline) {
            return Err(MambaError::ParseError(
                format!("Expected newline after ':' at {}:{}", 
                    self.current_position().line, 
                    self.current_position().column)
            ));
        }
        
        // Expect INDENT token
        if !self.match_token(&TokenKind::Indent) {
            return Err(MambaError::ParseError(
                format!("Expected indented block at {}:{}", 
                    self.current_position().line, 
                    self.current_position().column)
            ));
        }
        
        // Parse statements until DEDENT
        let mut statements = Vec::new();
        while !self.check(&TokenKind::Dedent) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        // Expect DEDENT token
        if !self.match_token(&TokenKind::Dedent) {
            return Err(MambaError::ParseError(
                format!("Expected dedent after block at {}:{}", 
                    self.current_position().line, 
                    self.current_position().column)
            ));
        }
        
        if statements.is_empty() {
            return Err(MambaError::ParseError(
                format!("Block cannot be empty (use 'pass' for empty blocks) at {}:{}", 
                    self.current_position().line, 
                    self.current_position().column)
            ));
        }
        
        Ok(statements)
    }

    /// Validate that at most one starred expression appears in unpacking targets
    fn validate_unpacking_targets(&self, targets: &[Expression]) -> ParseResult<()> {
        let starred_count = self.count_starred_expressions(targets);
        
        if starred_count > 1 {
            return Err(MambaError::ParseError(
                format!("Multiple starred expressions in assignment (only one allowed)")
            ));
        }
        
        Ok(())
    }

    /// Validate a single assignment target (checks for multiple starred expressions)
    fn validate_single_target(&self, target: &Expression) -> ParseResult<()> {
        match target {
            Expression::Tuple { elements, .. } => {
                self.validate_unpacking_targets(elements)?;
            }
            Expression::List { elements, .. } => {
                self.validate_unpacking_targets(elements)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Count starred expressions in a list of expressions (recursively handles tuples)
    fn count_starred_expressions(&self, exprs: &[Expression]) -> usize {
        let mut count = 0;
        for expr in exprs {
            match expr {
                Expression::Starred { .. } => count += 1,
                Expression::Tuple { elements, .. } => {
                    count += self.count_starred_expressions(elements);
                }
                Expression::List { elements, .. } => {
                    count += self.count_starred_expressions(elements);
                }
                _ => {}
            }
        }
        count
    }

    /// Parse an assignment target (expression or starred expression)
    /// Used on the LHS of assignments to handle starred unpacking
    fn parse_assignment_target(&mut self) -> ParseResult<Expression> {
        // Check for starred expression (*var)
        if self.match_token(&TokenKind::Star) {
            let pos = self.previous_position();
            let value = Box::new(self.parse_expression()?);
            return Ok(Expression::Starred {
                value,
                position: pos,
            });
        }
        
        // Otherwise parse regular expression
        self.parse_expression()
    }

    /// Parse expression or implicit tuple (comma-separated expressions)
    /// Used in assignment RHS where `1, 2` creates a tuple without parentheses
    fn parse_tuple_or_expression(&mut self) -> ParseResult<Expression> {
        let first = self.parse_expression()?;
        
        // Check for comma - creates implicit tuple
        if self.check(&TokenKind::Comma) {
            let mut elements = vec![first];
            let pos = elements[0].position().clone();
            
            while self.match_token(&TokenKind::Comma) {
                // Allow trailing comma before newline or EOF
                if self.check(&TokenKind::Newline) || self.is_at_end() {
                    break;
                }
                elements.push(self.parse_expression()?);
            }
            
            Ok(Expression::Tuple {
                elements,
                position: pos,
            })
        } else {
            Ok(first)
        }
    }

    /// Parse an expression with operator precedence
    fn parse_expression(&mut self) -> ParseResult<Expression> {
        // Check for lambda expression first (lowest precedence)
        if self.check(&TokenKind::Lambda) {
            return self.parse_lambda();
        }
        
        self.parse_conditional()
    }

    /// Parse conditional expression (x if condition else y)
    fn parse_conditional(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_walrus()?;
        
        // Check for 'if' keyword to start conditional
        if self.match_token(&TokenKind::If) {
            let pos = expr.position().clone();
            let condition = Box::new(self.parse_walrus()?);
            
            // Expect 'else' keyword
            self.expect_token(TokenKind::Else, "Expected 'else' in conditional expression")?;
            
            let false_expr = Box::new(self.parse_conditional()?); // Allow chaining
            
            expr = Expression::Conditional {
                condition,
                true_expr: Box::new(expr),
                false_expr,
                position: pos,
            };
        }
        
        Ok(expr)
    }

    /// Parse walrus operator / assignment expression (name := value)
    fn parse_walrus(&mut self) -> ParseResult<Expression> {
        let expr = self.parse_or()?;
        
        // Check if this is an identifier followed by :=
        if let Expression::Identifier { name, position } = &expr {
            if self.match_token(&TokenKind::Walrus) {
                let value = Box::new(self.parse_or()?);
                return Ok(Expression::AssignmentExpr {
                    target: name.clone(),
                    value,
                    position: position.clone(),
                });
            }
        }
        
        Ok(expr)
    }

    /// Parse lambda expression (lambda x, y: expr)
    fn parse_lambda(&mut self) -> ParseResult<Expression> {
        let pos = self.current_position();
        self.expect_token(TokenKind::Lambda, "Expected 'lambda'")?;
        
        let mut parameters = Vec::new();
        
        // Parse parameters (optional)
        if !self.check(&TokenKind::Colon) {
            loop {
                // Expect identifier for parameter name
                match self.current_kind() {
                    Some(TokenKind::Identifier(name)) => {
                        parameters.push(name.clone());
                        self.advance();
                    }
                    _ => {
                        let curr_pos = self.current_position();
                        return Err(MambaError::ParseError(format!(
                            "Expected parameter name at {}:{}",
                            curr_pos.line, curr_pos.column
                        )));
                    }
                }
                
                // Check for comma (more parameters) or colon (end of parameters)
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        
        // Expect colon before body
        self.expect_token(TokenKind::Colon, "Expected ':' after lambda parameters")?;
        
        // Parse body expression (but not another lambda to avoid ambiguity)
        let body = Box::new(self.parse_conditional()?);
        
        Ok(Expression::Lambda {
            parameters,
            body,
            position: pos,
        })
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
            Some(TokenKind::Ellipsis) => {
                let pos = self.current_position();
                self.advance();
                Ok(Expression::Literal(Literal::Ellipsis { position: pos }))
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
                
                // Check if it's a generator expression (has 'for' keyword)
                if self.check(&TokenKind::For) {
                    // Generator expression: (expr for target in iter)
                    let generators = self.parse_comprehension_generators()?;
                    self.expect_token(TokenKind::RightParen, "Expected ')' after generator expression")?;
                    
                    return Ok(Expression::GeneratorExpr {
                        element: Box::new(first_expr),
                        generators,
                        position: pos,
                    });
                }
                
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
                // List literal or list comprehension: [1, 2, 3] or [x for x in iter]
                let pos = self.current_position();
                self.advance(); // consume '['

                // Empty list
                if self.check(&TokenKind::RightBracket) {
                    self.advance();
                    return Ok(Expression::List {
                        elements: Vec::new(),
                        position: pos,
                    });
                }

                // Parse first element
                let first_element = self.parse_expression()?;

                // Check if it's a comprehension (has 'for' keyword)
                if self.check(&TokenKind::For) {
                    // List comprehension: [expr for target in iter]
                    let generators = self.parse_comprehension_generators()?;
                    self.expect_token(TokenKind::RightBracket, "Expected ']' after list comprehension")?;
                    
                    return Ok(Expression::ListComp {
                        element: Box::new(first_element),
                        generators,
                        position: pos,
                    });
                }

                // Regular list: parse remaining elements
                let mut elements = vec![first_element];
                
                while self.match_token(&TokenKind::Comma) {
                    // Allow trailing comma
                    if self.check(&TokenKind::RightBracket) {
                        break;
                    }
                    elements.push(self.parse_expression()?);
                }

                self.expect_token(TokenKind::RightBracket, "Expected ']' after list elements")?;
                
                Ok(Expression::List {
                    elements,
                    position: pos,
                })
            }
            Some(TokenKind::LeftBrace) => {
                // Dict, Set, or comprehension: {key: value} or {elem} or comprehensions
                // Empty braces {} always means empty dict
                let pos = self.current_position();
                self.advance(); // consume '{'

                // Empty braces = empty dict
                if self.check(&TokenKind::RightBrace) {
                    self.advance();
                    return Ok(Expression::Dict {
                        pairs: Vec::new(),
                        position: pos,
                    });
                }

                // Parse first expression
                let first_expr = self.parse_expression()?;

                // Check if it's a dict (has colon) or set (has comma or end)
                if self.match_token(&TokenKind::Colon) {
                    // Dict or dict comprehension
                    let first_value = self.parse_expression()?;
                    
                    // Check for dict comprehension
                    if self.check(&TokenKind::For) {
                        let generators = self.parse_comprehension_generators()?;
                        self.expect_token(TokenKind::RightBrace, "Expected '}' after dict comprehension")?;
                        
                        return Ok(Expression::DictComp {
                            key: Box::new(first_expr),
                            value: Box::new(first_value),
                            generators,
                            position: pos,
                        });
                    }
                    
                    // Regular dict
                    let mut pairs = vec![(first_expr, first_value)];

                    // Parse remaining pairs
                    while self.match_token(&TokenKind::Comma) {
                        // Allow trailing comma
                        if self.check(&TokenKind::RightBrace) {
                            break;
                        }

                        let key = self.parse_expression()?;
                        self.expect_token(TokenKind::Colon, "Expected ':' after dict key")?;
                        let value = self.parse_expression()?;
                        pairs.push((key, value));
                    }

                    self.expect_token(TokenKind::RightBrace, "Expected '}' after dict pairs")?;
                    
                    Ok(Expression::Dict {
                        pairs,
                        position: pos,
                    })
                } else {
                    // Set or set comprehension
                    
                    // Check for set comprehension
                    if self.check(&TokenKind::For) {
                        let generators = self.parse_comprehension_generators()?;
                        self.expect_token(TokenKind::RightBrace, "Expected '}' after set comprehension")?;
                        
                        return Ok(Expression::SetComp {
                            element: Box::new(first_expr),
                            generators,
                            position: pos,
                        });
                    }
                    
                    // Regular set
                    let mut elements = vec![first_expr];

                    // Parse remaining elements
                    while self.match_token(&TokenKind::Comma) {
                        // Allow trailing comma
                        if self.check(&TokenKind::RightBrace) {
                            break;
                        }

                        elements.push(self.parse_expression()?);
                    }

                    self.expect_token(TokenKind::RightBrace, "Expected '}' after set elements")?;
                    
                    Ok(Expression::Set {
                        elements,
                        position: pos,
                    })
                }
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
        // Save current position before advancing
        self.previous_position = self.current_position();
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
        self.previous_position.clone()
    }

    /// Check if we're at end of file
    fn is_at_end(&self) -> bool {
        matches!(
            self.current_kind(),
            Some(TokenKind::Eof) | None
        )
    }

    /// Parse comprehension generators: for target in iter [if cond] [for ...]
    fn parse_comprehension_generators(&mut self) -> ParseResult<Vec<Comprehension>> {
        let mut generators = Vec::new();

        // Parse at least one generator
        loop {
            if !self.match_token(&TokenKind::For) {
                break;
            }

            let pos = self.current_position();

            // Parse target (simple identifier for now)
            let target = match self.current_kind() {
                Some(TokenKind::Identifier(name)) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                _ => {
                    return Err(MambaError::ParseError(format!(
                        "Expected identifier after 'for' at {}:{}",
                        self.current_position().line,
                        self.current_position().column
                    )))
                }
            };

            // Expect 'in' keyword
            self.expect_token(TokenKind::In, "Expected 'in' after loop target")?;

            // Parse iterator expression (use or precedence to avoid 'if' being parsed as conditional)
            let iter = self.parse_or()?;

            // Parse optional 'if' conditions
            let mut conditions = Vec::new();
            while self.check(&TokenKind::If) {
                self.advance(); // consume 'if'
                conditions.push(self.parse_or()?);
            }

            generators.push(Comprehension {
                target,
                iter,
                conditions,
                position: pos,
            });
        }

        Ok(generators)
    }
}
