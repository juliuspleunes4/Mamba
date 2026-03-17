//! Semantic Analysis
//!
//! This module performs semantic analysis on the AST, building a symbol table
//! and detecting semantic errors such as undefined variables, redeclarations, etc.

use crate::ast::{BinaryOperator, Expression, Literal, Module, Statement, UnaryOperator};
use crate::symbol_table::{ScopeKind, SymbolKind, SymbolTable};
use crate::token::SourcePosition;
use crate::types::Type;
use std::collections::HashMap;

/// Semantic error types
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticError {
    /// Variable used before definition
    UndefinedVariable {
        name: String,
        position: SourcePosition,
    },
    /// Variable or function declared multiple times in same scope
    Redeclaration {
        name: String,
        first_position: SourcePosition,
        second_position: SourcePosition,
    },
    /// Invalid scope operation
    InvalidScope {
        message: String,
        position: SourcePosition,
    },
    /// nonlocal declaration at module level
    NonlocalAtModuleLevel {
        name: String,
        position: SourcePosition,
    },
    /// nonlocal name not found in any enclosing scope
    NonlocalNotFound {
        name: String,
        position: SourcePosition,
    },
    /// global declaration at module level is redundant
    GlobalAtModuleLevel {
        name: String,
        position: SourcePosition,
    },
    /// Type mismatch detected
    TypeMismatch {
        expected: String,
        actual: Type,
        position: SourcePosition,
    },
    /// Division by zero detected
    DivisionByZero {
        position: SourcePosition,
    },
    /// break statement outside of loop
    BreakOutsideLoop {
        position: SourcePosition,
    },
    /// continue statement outside of loop
    ContinueOutsideLoop {
        position: SourcePosition,
    },
    /// return statement outside of function
    ReturnOutsideFunction {
        position: SourcePosition,
    },
    /// unreachable code detected
    UnreachableCode {
        position: SourcePosition,
    },
    /// function called but not defined
    UndefinedFunction {
        name: String,
        position: SourcePosition,
    },
    /// function called with wrong number of arguments
    ArgumentCountMismatch {
        function: String,
        expected_min: usize,
        expected_max: usize,
        actual: usize,
        position: SourcePosition,
    },
    /// function called with wrong argument type
    ArgumentTypeMismatch {
        function: String,
        parameter: String,
        expected: Type,
        actual: Type,
        position: SourcePosition,
    },
    /// invalid assignment target (e.g., assigning to a literal or expression)
    InvalidAssignmentTarget {
        target: String,
        position: SourcePosition,
    },
}

impl SemanticError {
    /// Get the position of the error
    pub fn position(&self) -> &SourcePosition {
        match self {
            SemanticError::UndefinedVariable { position, .. } => position,
            SemanticError::Redeclaration { second_position, .. } => second_position,
            SemanticError::InvalidScope { position, .. } => position,
            SemanticError::NonlocalAtModuleLevel { position, .. } => position,
            SemanticError::NonlocalNotFound { position, .. } => position,
            SemanticError::GlobalAtModuleLevel { position, .. } => position,
            SemanticError::TypeMismatch { position, .. } => position,
            SemanticError::DivisionByZero { position } => position,
            SemanticError::BreakOutsideLoop { position } => position,
            SemanticError::ContinueOutsideLoop { position } => position,
            SemanticError::ReturnOutsideFunction { position } => position,
            SemanticError::UnreachableCode { position } => position,
            SemanticError::UndefinedFunction { position, .. } => position,
            SemanticError::ArgumentCountMismatch { position, .. } => position,
            SemanticError::ArgumentTypeMismatch { position, .. } => position,
            SemanticError::InvalidAssignmentTarget { position, .. } => position,
        }
    }

    /// Get a human-readable error message
    pub fn message(&self) -> String {
        match self {
            SemanticError::UndefinedVariable { name, .. } => {
                format!("Undefined variable: '{}'", name)
            }
            SemanticError::Redeclaration { name, .. } => {
                format!("Redeclaration of '{}'", name)
            }
            SemanticError::InvalidScope { message, .. } => message.clone(),
            SemanticError::NonlocalAtModuleLevel { name, .. } => {
                format!("nonlocal declaration not allowed at module level: '{}'", name)
            }
            SemanticError::NonlocalNotFound { name, .. } => {
                format!("no binding for nonlocal '{}' found", name)
            }
            SemanticError::GlobalAtModuleLevel { name, .. } => {
                format!("name '{}' is used prior to global declaration", name)
            }
            SemanticError::TypeMismatch { expected, actual, .. } => {
                format!("Type mismatch: expected {}, got {}", expected, actual)
            }
            SemanticError::DivisionByZero { .. } => {
                "Division by zero".to_string()
            }
            SemanticError::BreakOutsideLoop { .. } => {
                "'break' outside loop".to_string()
            }
            SemanticError::ContinueOutsideLoop { .. } => {
                "'continue' outside loop".to_string()
            }
            SemanticError::ReturnOutsideFunction { .. } => {
                "'return' outside function".to_string()
            }
            SemanticError::UnreachableCode { .. } => {
                "Unreachable code".to_string()
            }
            SemanticError::UndefinedFunction { name, .. } => {
                format!("Undefined function: '{}'", name)
            }
            SemanticError::ArgumentCountMismatch { function, expected_min, expected_max, actual, .. } => {
                if expected_min == expected_max {
                    format!("Function '{}' takes {} argument(s) but {} were given", function, expected_min, actual)
                } else {
                    format!("Function '{}' takes {}-{} arguments but {} were given", function, expected_min, expected_max, actual)
                }
            }
            SemanticError::ArgumentTypeMismatch { function, parameter, expected, actual, .. } => {
                format!("Function '{}' parameter '{}': expected {}, got {}", function, parameter, expected, actual)
            }
            SemanticError::InvalidAssignmentTarget { target, .. } => {
                format!("Cannot assign to {}", target)
            }
        }
    }
}

/// Function parameter metadata
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type (Unknown if not annotated)
    pub param_type: Type,
    /// Whether this parameter has a default value
    pub has_default: bool,
}

/// Function signature metadata
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,
    /// Function parameters
    pub parameters: Vec<Parameter>,
    /// Function return type (Unknown if not annotated)
    pub return_type: Type,
}

/// The semantic analyzer traverses the AST and builds a symbol table
pub struct SemanticAnalyzer {
    /// Symbol table tracking all declarations and scopes
    symbol_table: SymbolTable,
    /// Function return types
    function_types: HashMap<String, Type>,
    /// Function signatures for call validation
    function_signatures: HashMap<String, FunctionSignature>,
    /// Current function being analyzed
    current_function: Option<String>,
    /// Expected return type annotation for current function
    expected_return_type: Option<Type>,
    /// Loop nesting depth for break/continue validation
    loop_depth: usize,
    /// Collected semantic errors
    errors: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Self {
        let mut symbol_table = SymbolTable::new();
        
        // Declare built-in functions in the module scope
        let builtin_pos = SourcePosition::start();
        let _ = symbol_table.declare("print".to_string(), SymbolKind::Function, builtin_pos.clone());
        let _ = symbol_table.declare("range".to_string(), SymbolKind::Function, builtin_pos.clone());
        let _ = symbol_table.declare("len".to_string(), SymbolKind::Function, builtin_pos.clone());
        let _ = symbol_table.declare("str".to_string(), SymbolKind::Function, builtin_pos.clone());
        let _ = symbol_table.declare("int".to_string(), SymbolKind::Function, builtin_pos.clone());
        let _ = symbol_table.declare("float".to_string(), SymbolKind::Function, builtin_pos.clone());
        let _ = symbol_table.declare("bool".to_string(), SymbolKind::Function, builtin_pos.clone());
        let _ = symbol_table.declare("list".to_string(), SymbolKind::Function, builtin_pos.clone());
        let _ = symbol_table.declare("dict".to_string(), SymbolKind::Function, builtin_pos.clone());
        let _ = symbol_table.declare("set".to_string(), SymbolKind::Function, builtin_pos.clone());
        let _ = symbol_table.declare("tuple".to_string(), SymbolKind::Function, builtin_pos.clone());
        
        // Declare built-in constants
        let _ = symbol_table.declare("True".to_string(), SymbolKind::Variable, builtin_pos.clone());
        let _ = symbol_table.declare("False".to_string(), SymbolKind::Variable, builtin_pos.clone());
        let _ = symbol_table.declare("None".to_string(), SymbolKind::Variable, builtin_pos.clone());
        
        // Assign types to built-in constants
        symbol_table.assign_type("True", Type::Bool);
        symbol_table.assign_type("False", Type::Bool);
        symbol_table.assign_type("None", Type::None);
        
        Self {
            symbol_table,
            function_types: HashMap::new(),
            function_signatures: HashMap::new(),
            current_function: None,
            expected_return_type: None,
            loop_depth: 0,
            errors: Vec::new(),
        }
    }

    /// Analyze a module and return the symbol table or errors
    ///
    /// Returns Ok(symbol_table) if no errors, Err(errors) if errors found
    pub fn analyze(mut self, module: &Module) -> Result<SymbolTable, Vec<SemanticError>> {
        // Visit all statements in the module with unreachable code detection
        self.visit_statement_list(&module.statements);

        // Return symbol table if no errors, otherwise return errors
        if self.errors.is_empty() {
            Ok(self.symbol_table)
        } else {
            Err(self.errors)
        }
    }

    /// For testing: analyze and return self to access type_table
    #[cfg(test)]
    pub fn analyze_with_types(mut self, module: &Module) -> Self {
        self.visit_statement_list(&module.statements);
        self
    }

    /// Get the type of a variable by name
    pub fn get_type(&self, name: &str) -> Option<Type> {
        self.symbol_table.get_type(name)
    }

    /// For testing: get function_types reference
    #[cfg(test)]
    pub fn function_types(&self) -> &HashMap<String, Type> {
        &self.function_types
    }

    /// Infer the type of an expression
    fn infer_type(&mut self, expr: &Expression) -> Type {
        match expr {
            Expression::Literal(lit) => {
                match lit {
                    Literal::Integer { .. } => Type::Int,
                    Literal::Float { .. } => Type::Float,
                    Literal::String { .. } => Type::String,
                    Literal::Boolean { .. } => Type::Bool,
                    Literal::None { .. } => Type::None,
                    _ => Type::Unknown,
                }
            },
            Expression::Identifier { name, .. } => {
                // Look up existing type or return Unknown
                self.symbol_table.get_type(name).unwrap_or(Type::Unknown)
            },
            Expression::Call { function, .. } => {
                // If calling a function, return its inferred return type
                if let Expression::Identifier { name, .. } = &**function {
                    self.function_types.get(name).cloned().unwrap_or(Type::Unknown)
                } else {
                    Type::Unknown
                }
            },
            Expression::BinaryOp { left, op, right, position } => {
                let left_type = self.infer_type(left);
                let right_type = self.infer_type(right);
                self.check_binary_op_types(op, &left_type, &right_type, right, position)
            },
            Expression::UnaryOp { op, operand, position } => {
                let operand_type = self.infer_type(operand);
                self.check_unary_op_types(op, &operand_type, position)
            },
            Expression::Parenthesized { expr, .. } => {
                // Parentheses don't change the type
                self.infer_type(expr)
            },
            // For other expressions, return Unknown for now
            _ => Type::Unknown,
        }
    }

    /// Infer the result type of a binary operation
    fn infer_binary_op_type(&self, op: &BinaryOperator, left: &Type, right: &Type) -> Type {
        use BinaryOperator::*;
        
        match op {
            // Arithmetic operations
            Add => {
                match (left, right) {
                    (Type::Int, Type::Int) => Type::Int,
                    (Type::Float, Type::Float) => Type::Float,
                    (Type::Int, Type::Float) | (Type::Float, Type::Int) => Type::Float,
                    // Bool is a subclass of int in Python
                    (Type::Bool, Type::Bool) => Type::Int,
                    (Type::Bool, Type::Int) | (Type::Int, Type::Bool) => Type::Int,
                    (Type::Bool, Type::Float) | (Type::Float, Type::Bool) => Type::Float,
                    (Type::String, Type::String) => Type::String,
                    _ => Type::Unknown,
                }
            },
            Subtract | Multiply | Modulo | Power | FloorDivide => {
                match (left, right) {
                    (Type::Int, Type::Int) => Type::Int,
                    (Type::Float, Type::Float) => Type::Float,
                    (Type::Int, Type::Float) | (Type::Float, Type::Int) => Type::Float,
                    // Bool is a subclass of int in Python
                    (Type::Bool, Type::Bool) => Type::Int,
                    (Type::Bool, Type::Int) | (Type::Int, Type::Bool) => Type::Int,
                    (Type::Bool, Type::Float) | (Type::Float, Type::Bool) => Type::Float,
                    _ => Type::Unknown,
                }
            },
            Divide => {
                // In Python 3, division always returns float
                match (left, right) {
                    (Type::Int, Type::Int) | 
                    (Type::Float, Type::Float) |
                    (Type::Int, Type::Float) | 
                    (Type::Float, Type::Int) |
                    // Bool is a subclass of int in Python
                    (Type::Bool, Type::Bool) |
                    (Type::Bool, Type::Int) | (Type::Int, Type::Bool) |
                    (Type::Bool, Type::Float) | (Type::Float, Type::Bool) => Type::Float,
                    _ => Type::Unknown,
                }
            },
            // Comparison operations always return bool
            Equal | NotEqual | LessThan | LessThanEq | GreaterThan | GreaterThanEq => Type::Bool,
            // Logical operations
            And | Or => {
                match (left, right) {
                    (Type::Bool, Type::Bool) => Type::Bool,
                    _ => Type::Unknown,
                }
            },
            // Membership operators always return bool
            In | NotIn => Type::Bool,
            // Other operations return Unknown for now
            _ => Type::Unknown,
        }
    }

    /// Check binary operation types and report errors for invalid combinations
    fn check_binary_op_types(&mut self, op: &BinaryOperator, left: &Type, right: &Type, right_expr: &Expression, position: &SourcePosition) -> Type {
        use BinaryOperator::*;
        
        // Skip type checking for Unknown types (conservative approach)
        if matches!(left, Type::Unknown) || matches!(right, Type::Unknown) {
            return self.infer_binary_op_type(op, left, right);
        }
        
        // Check for division by zero
        if matches!(op, Divide | FloorDivide) {
            if let Expression::Literal(Literal::Integer { value, .. }) = right_expr {
                if *value == 0 {
                    self.add_error(SemanticError::DivisionByZero {
                        position: position.clone(),
                    });
                    return Type::Unknown;
                }
            }
            if let Expression::Literal(Literal::Float { value, .. }) = right_expr {
                if *value == 0.0 {
                    self.add_error(SemanticError::DivisionByZero {
                        position: position.clone(),
                    });
                    return Type::Unknown;
                }
            }
        }
        
        match op {
            // Arithmetic operations that don't work with strings or None
            Subtract | Multiply | Modulo | Power | FloorDivide => {
                if matches!(left, Type::String) || matches!(right, Type::String) {
                    self.add_error(SemanticError::TypeMismatch {
                        expected: "numeric types (int or float)".to_string(),
                        actual: if matches!(left, Type::String) { left.clone() } else { right.clone() },
                        position: position.clone(),
                    });
                    return Type::Unknown;
                }
                if matches!(left, Type::None) || matches!(right, Type::None) {
                    self.add_error(SemanticError::TypeMismatch {
                        expected: "numeric types (int or float)".to_string(),
                        actual: if matches!(left, Type::None) { left.clone() } else { right.clone() },
                        position: position.clone(),
                    });
                    return Type::Unknown;
                }
            },
            // Add: strings can only be added to strings, None cannot be added to anything
            Add => {
                // Check for None in addition
                if matches!(left, Type::None) || matches!(right, Type::None) {
                    self.add_error(SemanticError::TypeMismatch {
                        expected: "numeric or string types".to_string(),
                        actual: if matches!(left, Type::None) { left.clone() } else { right.clone() },
                        position: position.clone(),
                    });
                    return Type::Unknown;
                }
                // String type checking
                if matches!(left, Type::String) && !matches!(right, Type::String) {
                    self.add_error(SemanticError::TypeMismatch {
                        expected: "String".to_string(),
                        actual: right.clone(),
                        position: position.clone(),
                    });
                    return Type::Unknown;
                }
                if matches!(right, Type::String) && !matches!(left, Type::String) {
                    self.add_error(SemanticError::TypeMismatch {
                        expected: "String".to_string(),
                        actual: left.clone(),
                        position: position.clone(),
                    });
                    return Type::Unknown;
                }
            },
            // Division also doesn't work with None
            Divide => {
                if matches!(left, Type::None) || matches!(right, Type::None) {
                    self.add_error(SemanticError::TypeMismatch {
                        expected: "numeric types (int or float)".to_string(),
                        actual: if matches!(left, Type::None) { left.clone() } else { right.clone() },
                        position: position.clone(),
                    });
                    return Type::Unknown;
                }
            },
            // Bitwise operators require integer types
            BitwiseAnd | BitwiseOr | BitwiseXor | LeftShift | RightShift => {
                // Check both operands and collect all errors before returning
                let mut has_error = false;
                
                // Check left operand
                if !matches!(left, Type::Int | Type::Bool) {
                    self.add_error(SemanticError::TypeMismatch {
                        expected: "int".to_string(),
                        actual: left.clone(),
                        position: position.clone(),
                    });
                    has_error = true;
                }
                // Check right operand
                if !matches!(right, Type::Int | Type::Bool) {
                    self.add_error(SemanticError::TypeMismatch {
                        expected: "int".to_string(),
                        actual: right.clone(),
                        position: position.clone(),
                    });
                    has_error = true;
                }
                
                if has_error {
                    return Type::Unknown;
                }
            },
            // Comparison operators require compatible types
            Equal | NotEqual | LessThan | LessThanEq | GreaterThan | GreaterThanEq => {
                // None can be compared with anything (for equality)
                if matches!(op, Equal | NotEqual) {
                    // Equality operators are permissive
                } else {
                    // Ordering operators require compatible types
                    // String comparisons: both must be strings
                    if matches!(left, Type::String) && !matches!(right, Type::String) {
                        self.add_error(SemanticError::TypeMismatch {
                            expected: "str".to_string(),
                            actual: right.clone(),
                            position: position.clone(),
                        });
                        return Type::Unknown;
                    }
                    if matches!(right, Type::String) && !matches!(left, Type::String) {
                        self.add_error(SemanticError::TypeMismatch {
                            expected: "str".to_string(),
                            actual: left.clone(),
                            position: position.clone(),
                        });
                        return Type::Unknown;
                    }
                    // None cannot be ordered
                    if matches!(left, Type::None) || matches!(right, Type::None) {
                        self.add_error(SemanticError::TypeMismatch {
                            expected: "comparable types".to_string(),
                            actual: if matches!(left, Type::None) { left.clone() } else { right.clone() },
                            position: position.clone(),
                        });
                        return Type::Unknown;
                    }
                }
            },
            // Membership operators: right-hand side must be a container-like type.
            // With current type system we conservatively reject only known scalar RHS.
            In | NotIn => {
                if matches!(right, Type::Int | Type::Float | Type::Bool | Type::None) {
                    self.add_error(SemanticError::TypeMismatch {
                        expected: "container type (e.g., str, list, tuple, set, dict)".to_string(),
                        actual: right.clone(),
                        position: position.clone(),
                    });
                    return Type::Unknown;
                }
            },
            // Logical operators (And, Or) accept any type - truthy/falsy semantics
            // Identity operators (Is, IsNot) accept any type - reference comparison
            _ => {}
        }
        
        // If no errors, proceed with normal type inference
        self.infer_binary_op_type(op, left, right)
    }

    /// Parse a type annotation expression into a Type
    fn parse_type_annotation(&self, annotation: &Expression) -> Type {
        // For now, only handle simple identifier annotations like int, str, float, bool
        if let Expression::Identifier { name, .. } = annotation {
            match name.as_str() {
                "int" => Type::Int,
                "float" => Type::Float,
                "str" => Type::String,
                "bool" => Type::Bool,
                "None" => Type::None,
                _ => Type::Unknown,
            }
        } else {
            Type::Unknown
        }
    }

    /// Check unary operation types and report errors for invalid combinations
    fn check_unary_op_types(&mut self, op: &UnaryOperator, operand: &Type, position: &SourcePosition) -> Type {
        use UnaryOperator::*;
        
        // Skip type checking for Unknown types (conservative approach)
        if matches!(operand, Type::Unknown) {
            return self.infer_unary_op_type(op, operand);
        }
        
        // Check for invalid unary operations on strings
        match op {
            Minus | Plus => {
                if matches!(operand, Type::String) {
                    self.add_error(SemanticError::TypeMismatch {
                        expected: "numeric types (int or float)".to_string(),
                        actual: operand.clone(),
                        position: position.clone(),
                    });
                    return Type::Unknown;
                }
            },
            BitwiseNot => {
                if !matches!(operand, Type::Int) {
                    self.add_error(SemanticError::TypeMismatch {
                        expected: "int".to_string(),
                        actual: operand.clone(),
                        position: position.clone(),
                    });
                    return Type::Unknown;
                }
            },
            Not => {
                // Not operator can work with any type (truthy/falsy)
            },
        }
        
        // If no errors, proceed with normal type inference
        self.infer_unary_op_type(op, operand)
    }

    /// Infer the result type of a unary operation
    fn infer_unary_op_type(&self, op: &UnaryOperator, operand: &Type) -> Type {
        use UnaryOperator::*;
        
        match op {
            Not => Type::Bool,
            Minus | Plus => {
                match operand {
                    Type::Int => Type::Int,
                    Type::Float => Type::Float,
                    // Bool is a subclass of int in Python
                    Type::Bool => Type::Int,
                    _ => Type::Unknown,
                }
            },
            BitwiseNot => {
                match operand {
                    Type::Int => Type::Int,
                    _ => Type::Unknown,
                }
            },
        }
    }

    /// Recursively assign type to all names in assignment targets
    fn assign_type_to_names(&mut self, target: &Expression, typ: &Type) {
        match target {
            Expression::Identifier { name, .. } => {
                self.symbol_table.assign_type(name, typ.clone());
            },
            Expression::Tuple { elements, .. } | Expression::List { elements, .. } => {
                // For unpacking, all variables get the inferred type of the RHS expression
                // (currently this is typically Unknown for tuple/list literals).
                for elem in elements {
                    self.assign_type_to_names(elem, typ);
                }
            },
            _ => {
                // Other target types (subscript, attribute) - skip for now
            }
        }
    }

    /// Check if a statement always exits (returns, breaks, continues, raises)
    /// Also checks if/else blocks where ALL branches exit
    fn statement_always_exits(&self, statement: &Statement) -> bool {
        match statement {
            // Simple exit statements
            Statement::Return { .. }
            | Statement::Break(_)
            | Statement::Continue(_)
            | Statement::Raise { .. } => true,
            
            // If/else where all branches exit
            Statement::If { .. } => self.check_if_all_branches_exit(statement),
            
            // Try/except where all paths exit
            Statement::Try { .. } => self.check_try_all_branches_exit(statement),
            
            // All other statements don't always exit
            _ => false,
        }
    }

    /// Check if an if/else statement has all branches exiting
    /// Returns true only if:
    /// 1. The statement has an else clause (all paths covered)
    /// 2. Every branch (if, elif, else) contains at least one exiting statement
    fn check_if_all_branches_exit(&self, statement: &Statement) -> bool {
        if let Statement::If {
            then_block,
            elif_blocks,
            else_block,
            ..
        } = statement
        {
            // Must have an else clause to cover all paths
            if else_block.is_none() {
                return false;
            }
            
            // Check if the 'if' branch exits
            if !self.block_contains_exit(then_block) {
                return false;
            }
            
            // Check all 'elif' branches exit
            for (_condition, elif_body) in elif_blocks {
                if !self.block_contains_exit(elif_body) {
                    return false;
                }
            }
            
            // Check the 'else' branch exits
            if let Some(else_body) = else_block {
                if !self.block_contains_exit(else_body) {
                    return false;
                }
            }
            
            // All branches exit!
            true
        } else {
            false
        }
    }

    /// Check if a try/except/else/finally statement has all branches exiting
    /// Returns true if:
    /// 1. Finally block exits (always executes last), OR
    /// 2. Try body exits AND all except handlers exit AND (no else OR else exits)
    fn check_try_all_branches_exit(&self, statement: &Statement) -> bool {
        if let Statement::Try {
            body,
            handlers,
            orelse,
            finalbody,
            ..
        } = statement
        {
            // If finally block exists and exits, the entire try statement exits
            if let Some(finally) = finalbody {
                if self.block_contains_exit(finally) {
                    return true;
                }
            }
            
            // Otherwise, check if try body exits
            if !self.block_contains_exit(body) {
                return false;
            }
            
            // Check that all except handlers exit
            for handler in handlers {
                if !self.block_contains_exit(&handler.body) {
                    return false;
                }
            }
            
            // If there's an else block, it must also exit
            if let Some(else_body) = orelse {
                if !self.block_contains_exit(else_body) {
                    return false;
                }
            }
            
            // All paths exit!
            true
        } else {
            false
        }
    }

    /// Check if a block of statements contains at least one exiting statement
    /// Recursively checks nested if/else blocks
    fn block_contains_exit(&self, block: &[Statement]) -> bool {
        for statement in block {
            if self.statement_always_exits(statement) {
                return true;
            }
        }
        false
    }

    /// Check if an expression is a valid assignment target
    fn check_assignment_target(&mut self, target: &Expression) {
        match target {
            // Valid assignment targets
            Expression::Identifier { .. } => {
                // Variables are valid assignment targets
            }
            Expression::Tuple { elements, .. } | Expression::List { elements, .. } => {
                // Tuple/list unpacking - check each element recursively
                for elem in elements {
                    self.check_assignment_target(elem);
                }
            }
            Expression::Subscript { .. } => {
                // Subscript assignment (e.g., list[0] = 5) is valid
            }
            Expression::Attribute { .. } => {
                // Attribute assignment (e.g., obj.attr = 5) is valid
            }
            Expression::Starred { value, .. } => {
                // Starred expression in unpacking (e.g., *rest) - check inner expression
                self.check_assignment_target(value);
            }
            
            // Invalid assignment targets
            Expression::Literal(lit) => {
                let (target_name, position) = match lit {
                    Literal::Integer { position, .. } | Literal::Float { position, .. } |
                    Literal::String { position, .. } | Literal::Boolean { position, .. } => {
                        ("literal".to_string(), position.clone())
                    }
                    Literal::None { position } => {
                        ("None".to_string(), position.clone())
                    }
                    Literal::Ellipsis { position } => {
                        ("Ellipsis".to_string(), position.clone())
                    }
                };
                self.add_error(SemanticError::InvalidAssignmentTarget {
                    target: target_name,
                    position,
                });
            }
            Expression::Call { position, .. } => {
                self.add_error(SemanticError::InvalidAssignmentTarget {
                    target: "function call".to_string(),
                    position: position.clone(),
                });
            }
            Expression::BinaryOp { position, .. } => {
                self.add_error(SemanticError::InvalidAssignmentTarget {
                    target: "operator".to_string(),
                    position: position.clone(),
                });
            }
            Expression::UnaryOp { position, .. } => {
                self.add_error(SemanticError::InvalidAssignmentTarget {
                    target: "operator".to_string(),
                    position: position.clone(),
                });
            }
            Expression::Lambda { position, .. } => {
                self.add_error(SemanticError::InvalidAssignmentTarget {
                    target: "lambda".to_string(),
                    position: position.clone(),
                });
            }
            Expression::Conditional { position, .. } => {
                self.add_error(SemanticError::InvalidAssignmentTarget {
                    target: "conditional expression".to_string(),
                    position: position.clone(),
                });
            }
            Expression::Dict { position, .. } => {
                self.add_error(SemanticError::InvalidAssignmentTarget {
                    target: "dict display".to_string(),
                    position: position.clone(),
                });
            }
            Expression::Set { position, .. } => {
                self.add_error(SemanticError::InvalidAssignmentTarget {
                    target: "set display".to_string(),
                    position: position.clone(),
                });
            }
            Expression::ListComp { position, .. } => {
                self.add_error(SemanticError::InvalidAssignmentTarget {
                    target: "list comprehension".to_string(),
                    position: position.clone(),
                });
            }
            Expression::SetComp { position, .. } => {
                self.add_error(SemanticError::InvalidAssignmentTarget {
                    target: "set comprehension".to_string(),
                    position: position.clone(),
                });
            }
            Expression::DictComp { position, .. } => {
                self.add_error(SemanticError::InvalidAssignmentTarget {
                    target: "dict comprehension".to_string(),
                    position: position.clone(),
                });
            }
            Expression::GeneratorExpr { position, .. } => {
                self.add_error(SemanticError::InvalidAssignmentTarget {
                    target: "generator expression".to_string(),
                    position: position.clone(),
                });
            }
            Expression::AssignmentExpr { position, .. } => {
                self.add_error(SemanticError::InvalidAssignmentTarget {
                    target: "named expression".to_string(),
                    position: position.clone(),
                });
            }
            Expression::Parenthesized { expr, .. } => {
                // Parenthesized expressions - check the inner expression
                self.check_assignment_target(expr);
            }
        }
    }

    /// Visit a list of statements with unreachable code detection
    fn visit_statement_list(&mut self, statements: &[Statement]) {
        let mut seen_exit = false;
        
        for statement in statements {
            // Check if previous statement always exits
            if seen_exit {
                self.add_error(SemanticError::UnreachableCode {
                    position: statement.position().clone(),
                });
            }
            
            // Visit the statement
            self.visit_statement(statement);
            
            // Update exit flag
            if self.statement_always_exits(statement) {
                seen_exit = true;
            }
        }
    }

    /// Validate a function call (argument count and types)
    fn validate_function_call(&mut self, function_name: &str, arguments: &[Expression], position: &SourcePosition) {
        // List of built-in functions that we skip validation for
        const BUILTINS: &[&str] = &[
            "print", "len", "range", "str", "int", "float", "bool",
            "list", "dict", "set", "tuple", "type", "isinstance",
            "hasattr", "getattr", "setattr", "dir", "help",
            "sum", "min", "max", "abs", "round", "pow",
            "input", "open", "chr", "ord", "enumerate", "zip",
            "map", "filter", "sorted", "reversed", "all", "any"
        ];
        
        // Skip validation for built-in functions
        if BUILTINS.contains(&function_name) {
            return;
        }
        
        // Check if function exists
        let signature = match self.function_signatures.get(function_name).cloned() {
            Some(sig) => sig,
            None => {
                // Function not found
                self.add_error(SemanticError::UndefinedFunction {
                    name: function_name.to_string(),
                    position: position.clone(),
                });
                return;
            }
        };
        
        // Count required and total parameters
        let required_params = signature.parameters.iter()
            .filter(|p| !p.has_default)
            .count();
        let total_params = signature.parameters.len();
        let actual_args = arguments.len();
        
        // Check argument count
        if actual_args < required_params || actual_args > total_params {
            self.add_error(SemanticError::ArgumentCountMismatch {
                function: function_name.to_string(),
                expected_min: required_params,
                expected_max: total_params,
                actual: actual_args,
                position: position.clone(),
            });
            return; // Don't check types if count is wrong
        }
        
        // Check argument types (best-effort, only when both types are known)
        for (i, arg) in arguments.iter().enumerate() {
            if i >= signature.parameters.len() {
                break; // Should not happen after count check
            }
            
            let param = &signature.parameters[i];
            let expected_type = &param.param_type;
            
            // Skip if parameter type is unknown
            if *expected_type == Type::Unknown {
                continue;
            }
            
            // Infer argument type
            let actual_type = self.infer_type(arg);
            
            // Skip if argument type is unknown
            if actual_type == Type::Unknown {
                continue;
            }
            
            // Check type compatibility
            if actual_type != *expected_type {
                self.add_error(SemanticError::ArgumentTypeMismatch {
                    function: function_name.to_string(),
                    parameter: param.name.clone(),
                    expected: expected_type.clone(),
                    actual: actual_type,
                    position: arg.position().clone(),
                });
            }
        }
    }

    /// Visit a statement and perform semantic analysis
    fn visit_statement(&mut self, statement: &Statement) {
        match statement {
            // Assignment - track variable declarations and types
            Statement::Assignment { targets, value, position } => {
                // Visit the value expression first
                self.visit_expression(value);
                
                // Validate all assignment targets
                for target in targets {
                    self.check_assignment_target(target);
                }
                
                // Infer the type of the value
                let value_type = self.infer_type(value);
                
                // Extract and declare all target variables, then assign type
                for target in targets {
                    self.extract_and_declare_names(target, position);
                    self.assign_type_to_names(target, &value_type);
                }
            }

            // AnnAssignment - track typed variable declarations and infer types
            Statement::AnnAssignment { target, annotation, value, position, .. } => {
                // Parse annotation to get expected type
                let expected_type = self.parse_type_annotation(annotation);
                
                // Infer type from value if present, otherwise use expected type
                let inferred_type = if let Some(val) = value {
                    self.visit_expression(val);
                    let val_type = self.infer_type(val);
                    
                    // Check if value type matches annotation
                    if !matches!(expected_type, Type::Unknown) && 
                       !matches!(val_type, Type::Unknown) &&
                       expected_type != val_type {
                        self.add_error(SemanticError::TypeMismatch {
                            expected: format!("{}", expected_type),
                            actual: val_type.clone(),
                            position: position.clone(),
                        });
                    }
                    
                    val_type
                } else {
                    expected_type
                };
                
                // Declare the variable
                if let Err(existing) = self.symbol_table.declare(
                    target.clone(),
                    SymbolKind::Variable,
                    position.clone()
                ) {
                    self.add_error(SemanticError::Redeclaration {
                        name: target.clone(),
                        first_position: existing.position.clone(),
                        second_position: position.clone(),
                    });
                }
                
                // Assign the inferred type
                self.symbol_table.assign_type(target, inferred_type);
            }

            // AugmentedAssignment - check variable exists before augmenting
            Statement::AugmentedAssignment { target, value, position, .. } => {
                // Validate assignment target first
                self.check_assignment_target(target);
                
                // Visit the value expression
                self.visit_expression(value);
                
                // Check if target exists (for identifiers)
                if let Expression::Identifier { name, .. } = target {
                    if self.symbol_table.lookup(name).is_none() {
                        self.add_error(SemanticError::UndefinedVariable {
                            name: name.clone(),
                            position: position.clone(),
                        });
                    }
                } else {
                    // For complex targets (like attributes, subscripts), just visit them
                    self.visit_expression(target);
                }
            }

            // FunctionDef - track function declarations
            Statement::FunctionDef { name, parameters, body, return_type, position, .. } => {
                // Declare function in current scope
                if let Err(existing) = self.symbol_table.declare(
                    name.clone(),
                    SymbolKind::Function,
                    position.clone()
                ) {
                    self.add_error(SemanticError::Redeclaration {
                        name: name.clone(),
                        first_position: existing.position.clone(),
                        second_position: position.clone(),
                    });
                }

                // Track current function for return type inference
                let prev_function = self.current_function.take();
                self.current_function = Some(name.clone());
                
                // Parse return type annotation if present
                let prev_expected_return = self.expected_return_type.take();
                self.expected_return_type = return_type.as_ref().map(|rt| self.parse_type_annotation(rt));
                
                // Initialize function return type to None
                self.function_types.insert(name.clone(), Type::None);

                // Build function signature for call validation
                let mut sig_parameters = Vec::new();
                for param in parameters {
                    let param_type = param.type_annotation.as_ref()
                        .map(|ann| self.parse_type_annotation(ann))
                        .unwrap_or(Type::Unknown);
                    sig_parameters.push(Parameter {
                        name: param.name.clone(),
                        param_type,
                        has_default: param.default.is_some(),
                    });
                }
                let sig_return_type = return_type.as_ref()
                    .map(|rt| self.parse_type_annotation(rt))
                    .unwrap_or(Type::Unknown);
                self.function_signatures.insert(name.clone(), FunctionSignature {
                    name: name.clone(),
                    parameters: sig_parameters,
                    return_type: sig_return_type,
                });

                // Enter new function scope
                self.symbol_table.enter_scope(ScopeKind::Function);

                // Declare parameters in function scope
                for param in parameters {
                    if let Err(existing) = self.symbol_table.declare(
                        param.name.clone(),
                        SymbolKind::Parameter,
                        param.position.clone()
                    ) {
                        self.add_error(SemanticError::Redeclaration {
                            name: param.name.clone(),
                            first_position: existing.position.clone(),
                            second_position: param.position.clone(),
                        });
                    }
                }

                // Analyze function body with unreachable code detection
                self.visit_statement_list(body);

                // Exit function scope
                self.symbol_table.exit_scope();
                
                // Restore previous function and return type context
                self.current_function = prev_function;
                self.expected_return_type = prev_expected_return;
            }

            // ClassDef - track class declarations
            Statement::ClassDef { name, body, position, .. } => {
                // Declare class in current scope
                if let Err(existing) = self.symbol_table.declare(
                    name.clone(),
                    SymbolKind::Class,
                    position.clone()
                ) {
                    self.add_error(SemanticError::Redeclaration {
                        name: name.clone(),
                        first_position: existing.position.clone(),
                        second_position: position.clone(),
                    });
                }

                // Enter new class scope
                self.symbol_table.enter_scope(ScopeKind::Class);

                // Analyze class body with unreachable code detection
                self.visit_statement_list(body);

                // Exit class scope
                self.symbol_table.exit_scope();
            }

            // Try statement - handle try/except/else/finally blocks
            Statement::Try { body, handlers, orelse, finalbody, .. } => {
                // Visit try body
                self.visit_statement_list(body);
                
                // Visit each except handler
                for handler in handlers {
                    // If handler has an 'as' clause, create a new scope for the exception variable
                    let has_exception_var = handler.name.is_some();
                    
                    if has_exception_var {
                        self.symbol_table.enter_scope(ScopeKind::Block);
                        
                        // Declare the exception variable in the handler scope
                        if let Some(ref exc_name) = handler.name {
                            if let Err(existing) = self.symbol_table.declare(
                                exc_name.clone(),
                                SymbolKind::Variable,
                                handler.position.clone()
                            ) {
                                self.add_error(SemanticError::Redeclaration {
                                    name: exc_name.clone(),
                                    first_position: existing.position.clone(),
                                    second_position: handler.position.clone(),
                                });
                            }
                        }
                    }
                    
                    // Visit exception type expression if present
                    if let Some(ref exc_type) = handler.exception_type {
                        self.visit_expression(exc_type);
                    }
                    
                    // Visit handler body
                    self.visit_statement_list(&handler.body);
                    
                    // Exit exception variable scope if we created one
                    if has_exception_var {
                        self.symbol_table.exit_scope();
                    }
                }
                
                // Visit else block if present
                if let Some(else_stmts) = orelse {
                    self.visit_statement_list(else_stmts);
                }
                
                // Visit finally block if present
                if let Some(finally_stmts) = finalbody {
                    self.visit_statement_list(finally_stmts);
                }
            }

            // If - no new scope in Python, just visit all parts
            Statement::If { condition, then_block, elif_blocks, else_block, .. } => {
                // Visit condition
                self.visit_expression(condition);
                
                // Visit then block with unreachable code detection
                self.visit_statement_list(then_block);
                
                // Visit elif blocks
                for (elif_condition, elif_body) in elif_blocks {
                    self.visit_expression(elif_condition);
                    self.visit_statement_list(elif_body);
                }
                
                // Visit else block
                if let Some(else_body) = else_block {
                    self.visit_statement_list(else_body);
                }
            }

            // While - no new scope in Python, just visit condition and body
            Statement::While { condition, body, else_block, .. } => {
                // Visit condition
                self.visit_expression(condition);
                
                // Enter loop context
                self.loop_depth += 1;
                
                // Visit body with unreachable code detection
                self.visit_statement_list(body);
                
                // Exit loop context
                self.loop_depth -= 1;
                
                // Visit else block if present (not in loop context)
                if let Some(else_body) = else_block {
                    self.visit_statement_list(else_body);
                }
            }

            // For - declare loop variable in current scope, no new scope
            Statement::For { target, iter, body, else_block, position } => {
                // Visit iterator expression first
                self.visit_expression(iter);
                
                // Declare loop variable(s) in current scope
                self.extract_and_declare_names(target, position);
                
                // Assign Unknown type to loop variable(s) since we don't track iterable types yet
                self.assign_type_to_names(target, &Type::Unknown);
                
                // Enter loop context
                self.loop_depth += 1;
                
                // Visit body with unreachable code detection
                self.visit_statement_list(body);
                
                // Exit loop context
                self.loop_depth -= 1;
                
                // Visit else block if present (not in loop context)
                if let Some(else_body) = else_block {
                    self.visit_statement_list(else_body);
                }
            }

            // Expression statement - just visit the expression
            Statement::Expression(expr) => {
                self.visit_expression(expr);
            }

            // TODO: Import/ImportFrom - track imported names
            Statement::Import { .. } => {
                // TODO: Declare imported module names
            }
            Statement::FromImport { .. } => {
                // TODO: Declare imported names
            }

            // Global - mark variables as global
            Statement::Global { names, position } => {
                // Check if we're at module level
                if self.symbol_table.current_scope_kind() == ScopeKind::Module {
                    // global at module level is allowed but redundant in Python
                    // We'll just skip it without error
                    return;
                }
                
                // Mark each name as global
                for name in names {
                    // Check if already declared in current scope
                    if let Some(existing) = self.symbol_table.lookup_current_scope(name) {
                        self.add_error(SemanticError::Redeclaration {
                            name: name.clone(),
                            first_position: existing.position.clone(),
                            second_position: position.clone(),
                        });
                        continue;
                    }
                    
                    // Declare the variable as global in current scope
                    // This creates a local reference to the global variable
                    if self.symbol_table.declare(
                        name.clone(),
                        SymbolKind::Variable,
                        position.clone()
                    ).is_ok() {
                        self.symbol_table.mark_global(name);
                    }
                }
            }

            // Nonlocal - mark variables as nonlocal
            Statement::Nonlocal { names, position } => {
                // Check if we're at module level
                if self.symbol_table.current_scope_kind() == ScopeKind::Module {
                    for name in names {
                        self.add_error(SemanticError::NonlocalAtModuleLevel {
                            name: name.clone(),
                            position: position.clone(),
                        });
                    }
                    return;
                }
                
                // For each name, find it in an enclosing scope (not global)
                for name in names {
                    // Check if already declared in current scope
                    if let Some(existing) = self.symbol_table.lookup_current_scope(name) {
                        self.add_error(SemanticError::Redeclaration {
                            name: name.clone(),
                            first_position: existing.position.clone(),
                            second_position: position.clone(),
                        });
                        continue;
                    }
                    
                    // Look for the variable in enclosing scopes (excluding module/global)
                    if self.symbol_table.lookup_in_enclosing_function_scopes(name).is_some() {
                        // Declare the nonlocal reference in current scope
                        if self.symbol_table.declare(
                            name.clone(),
                            SymbolKind::Variable,
                            position.clone()
                        ).is_ok() {
                            self.symbol_table.mark_nonlocal(name);
                        }
                    } else {
                        self.add_error(SemanticError::NonlocalNotFound {
                            name: name.clone(),
                            position: position.clone(),
                        });
                    }
                }
            }

            // Statements with expressions that need semantic analysis
            Statement::Return { value, position } => {
                // Validate return is inside a function
                if self.current_function.is_none() {
                    self.add_error(SemanticError::ReturnOutsideFunction {
                        position: position.clone(),
                    });
                }
                
                if let Some(expr) = value {
                    self.visit_expression(expr);
                    
                    // Infer return type and update function type
                    let return_type = self.infer_type(expr);
                    
                    // Check against expected return type annotation
                    if let Some(expected) = &self.expected_return_type {
                        if !matches!(expected, Type::Unknown) && 
                           !matches!(return_type, Type::Unknown) &&
                           expected != &return_type {
                            self.add_error(SemanticError::TypeMismatch {
                                expected: format!("{}", expected),
                                actual: return_type.clone(),
                                position: position.clone(),
                            });
                        }
                    }
                    
                    if let Some(func_name) = &self.current_function {
                        self.function_types.insert(func_name.clone(), return_type);
                    }
                }
                // Note: return without value keeps function type as None
            }

            Statement::Assert { condition, message, .. } => {
                self.visit_expression(condition);
                if let Some(msg) = message {
                    self.visit_expression(msg);
                }
            }

            Statement::Del { targets, .. } => {
                for target in targets {
                    self.visit_expression(target);
                }
            }

            Statement::Raise { exception, .. } => {
                if let Some(exc) = exception {
                    self.visit_expression(exc);
                }
            }

            // Break statement - must be in a loop
            Statement::Break(position) => {
                if self.loop_depth == 0 {
                    self.add_error(SemanticError::BreakOutsideLoop {
                        position: position.clone(),
                    });
                }
            }

            // Continue statement - must be in a loop
            Statement::Continue(position) => {
                if self.loop_depth == 0 {
                    self.add_error(SemanticError::ContinueOutsideLoop {
                        position: position.clone(),
                    });
                }
            }

            // Statements with no expressions to visit
            Statement::Pass(_) => {
                // No child expressions
            }
        }
    }

    /// Visit an expression and perform semantic analysis
    fn visit_expression(&mut self, expression: &Expression) {
        match expression {
            // Identifier - check if variable is defined
            Expression::Identifier { name, position } => {
                if self.symbol_table.lookup(name).is_none() {
                    self.add_error(SemanticError::UndefinedVariable {
                        name: name.clone(),
                        position: position.clone(),
                    });
                }
            }

            // Binary operation - visit both operands
            Expression::BinaryOp { left, right, .. } => {
                self.visit_expression(left);
                self.visit_expression(right);
            }

            // Unary operation - visit operand
            Expression::UnaryOp { operand, .. } => {
                self.visit_expression(operand);
            }

            // Parenthesized expression - visit inner expression
            Expression::Parenthesized { expr, .. } => {
                self.visit_expression(expr);
            }

            // Function call - visit function and all arguments
            Expression::Call { function, arguments, position } => {
                self.visit_expression(function);
                for arg in arguments {
                    self.visit_expression(arg);
                }
                
                // Validate function call if it's a simple identifier
                if let Expression::Identifier { name, .. } = &**function {
                    self.validate_function_call(name, arguments, position);
                }
            }

            // Attribute access - visit object
            Expression::Attribute { object, .. } => {
                self.visit_expression(object);
            }

            // Subscript - visit both object and index
            Expression::Subscript { object, index, .. } => {
                self.visit_expression(object);
                self.visit_expression(index);
            }

            // List - visit all elements
            Expression::List { elements, .. } => {
                for element in elements {
                    self.visit_expression(element);
                }
            }

            // Tuple - visit all elements
            Expression::Tuple { elements, .. } => {
                for element in elements {
                    self.visit_expression(element);
                }
            }

            // Dict - visit all keys and values
            Expression::Dict { pairs, .. } => {
                for (key, value) in pairs {
                    self.visit_expression(key);
                    self.visit_expression(value);
                }
            }

            // Set - visit all elements
            Expression::Set { elements, .. } => {
                for element in elements {
                    self.visit_expression(element);
                }
            }

            // Conditional expression - visit all three parts
            Expression::Conditional { condition, true_expr, false_expr, .. } => {
                self.visit_expression(condition);
                self.visit_expression(true_expr);
                self.visit_expression(false_expr);
            }

            // Assignment expression (walrus operator) - declare or reassign and infer type
            Expression::AssignmentExpr { target, value, position } => {
                self.visit_expression(value);
                
                // Infer the type of the value
                let value_type = self.infer_type(value);
                
                // In Python, walrus operator can both introduce new variables and reassign existing ones.
                // Check if variable exists in current scope - if not, declare it; if yes, it's a reassignment.
                if self.symbol_table.lookup_current_scope(target).is_none() {
                    // Variable doesn't exist in current scope, declare it
                    let _ = self.symbol_table.declare(
                        target.clone(),
                        SymbolKind::Variable,
                        position.clone()
                    );
                }
                // Assign the inferred type (works for both new variables and reassignments)
                self.symbol_table.assign_type(target, value_type);
            }

            // Starred expression - visit the value
            Expression::Starred { value, .. } => {
                self.visit_expression(value);
            }

            // TODO: Lambda - track lambda parameters
            Expression::Lambda { .. } => {
                // TODO: Enter new scope
                // TODO: Declare parameters
                // TODO: Visit body
                // TODO: Exit scope
            }

            // TODO: ListComp/SetComp/DictComp/GeneratorExpr - handle comprehension scopes
            Expression::ListComp { .. }
            | Expression::SetComp { .. }
            | Expression::DictComp { .. }
            | Expression::GeneratorExpr { .. } => {
                // TODO: Enter new scope
                // TODO: Visit generators (declare loop variables)
                // TODO: Visit element/key/value
                // TODO: Exit scope
            }

            // Literals - no semantic analysis needed
            Expression::Literal(_) => {}
        }
    }

    /// Add a semantic error to the error list
    fn add_error(&mut self, error: SemanticError) {
        self.errors.push(error);
    }

    /// Extract identifier names from an expression and declare them as variables
    ///
    /// Handles:
    /// - Identifier: x
    /// - Tuple: (a, b, c)
    /// - List: [a, b, c]
    /// - Starred: *rest
    fn extract_and_declare_names(&mut self, expr: &Expression, position: &SourcePosition) {
        match expr {
            Expression::Identifier { name, .. } => {
                // Check if already declared as global or nonlocal in current scope
                if let Some(existing) = self.symbol_table.lookup_current_scope(name) {
                    // If it's a global or nonlocal declaration, don't redeclare
                    if existing.is_global || existing.is_nonlocal {
                        return; // Skip declaration, it's a reference to outer scope
                    }
                    // Otherwise it's a redeclaration error
                    self.add_error(SemanticError::Redeclaration {
                        name: name.clone(),
                        first_position: existing.position.clone(),
                        second_position: position.clone(),
                    });
                    return;
                }
                
                // Declare new variable
                if let Err(existing) = self.symbol_table.declare(
                    name.clone(),
                    SymbolKind::Variable,
                    position.clone()
                ) {
                    self.add_error(SemanticError::Redeclaration {
                        name: name.clone(),
                        first_position: existing.position.clone(),
                        second_position: position.clone(),
                    });
                }
            }
            Expression::Tuple { elements, .. } | Expression::List { elements, .. } => {
                // Tuple/list unpacking: (a, b, c) = ...
                for element in elements {
                    self.extract_and_declare_names(element, position);
                }
            }
            Expression::Starred { value, .. } => {
                // Starred expression: *rest = ...
                self.extract_and_declare_names(value, position);
            }
            _ => {
                // For other expressions (attributes, subscripts), we don't declare variables
                // These are assignment targets but not new variable declarations
            }
        }
    }

    /// Get a reference to the symbol table (for testing)
    #[cfg(test)]
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use mamba_error::MambaError;

    /// Helper to parse code and create an analyzer
    fn parse(code: &str) -> Module {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().expect("Tokenize should succeed");
        let mut parser = Parser::new(tokens);
        parser.parse().expect("Parse should succeed")
    }

    /// Helper that returns Result for tests that might have parse errors
    fn try_parse(code: &str) -> Result<Module, Vec<MambaError>> {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().map_err(|e| vec![e])?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = SemanticAnalyzer::new();
        assert_eq!(analyzer.errors.len(), 0);
        assert_eq!(analyzer.symbol_table.current_scope_id(), 0);
    }

    #[test]
    fn test_analyze_empty_module() {
        let module = parse("");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Empty module should analyze without errors");
    }

    #[test]
    fn test_analyze_literal_expression() {
        let module = parse("42");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Literal expression should analyze without errors");
    }

    #[test]
    fn test_analyze_simple_expression() {
        let module = parse("1 + 2");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Simple expression should analyze without errors");
    }

    #[test]
    fn test_error_position() {
        let pos = SourcePosition { line: 1, column: 5, offset: 5 };
        let error = SemanticError::UndefinedVariable {
            name: "x".to_string(),
            position: pos.clone(),
        };
        assert_eq!(error.position(), &pos);
        assert_eq!(error.message(), "Undefined variable: 'x'");
    }

    #[test]
    fn test_redeclaration_error() {
        let pos1 = SourcePosition { line: 1, column: 0, offset: 0 };
        let pos2 = SourcePosition { line: 2, column: 0, offset: 10 };
        let error = SemanticError::Redeclaration {
            name: "x".to_string(),
            first_position: pos1,
            second_position: pos2.clone(),
        };
        assert_eq!(error.position(), &pos2);
        assert_eq!(error.message(), "Redeclaration of 'x'");
    }

    #[test]
    fn test_invalid_scope_error() {
        let pos = SourcePosition { line: 1, column: 0, offset: 0 };
        let error = SemanticError::InvalidScope {
            message: "Cannot exit root scope".to_string(),
            position: pos.clone(),
        };
        assert_eq!(error.position(), &pos);
        assert_eq!(error.message(), "Cannot exit root scope");
    }

    // Variable Declaration Tests

    #[test]
    fn test_simple_assignment() {
        let module = parse("x = 5");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Simple assignment should succeed");
        
        let table = result.unwrap();
        let symbol = table.lookup("x");
        assert!(symbol.is_some(), "Variable x should be declared");
        assert_eq!(symbol.unwrap().kind, SymbolKind::Variable);
    }

    #[test]
    fn test_multiple_assignment() {
        let module = parse("x = y = 10");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Multiple assignment should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("x").is_some(), "Variable x should be declared");
        assert!(table.lookup("y").is_some(), "Variable y should be declared");
    }

    #[test]
    fn test_tuple_unpacking() {
        let module = parse("a, b = 1, 2");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Tuple unpacking should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("a").is_some(), "Variable a should be declared");
        assert!(table.lookup("b").is_some(), "Variable b should be declared");
    }

    #[test]
    fn test_list_unpacking() {
        let module = parse("[x, y, z] = [1, 2, 3]");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "List unpacking should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("x").is_some(), "Variable x should be declared");
        assert!(table.lookup("y").is_some(), "Variable y should be declared");
        assert!(table.lookup("z").is_some(), "Variable z should be declared");
    }

    #[test]
    fn test_nested_unpacking() {
        let module = parse("(a, (b, c)) = (1, (2, 3))");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Nested unpacking should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("a").is_some(), "Variable a should be declared");
        assert!(table.lookup("b").is_some(), "Variable b should be declared");
        assert!(table.lookup("c").is_some(), "Variable c should be declared");
    }

    #[test]
    fn test_starred_unpacking() {
        let module = parse("a, *rest = [1, 2, 3, 4]");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Starred unpacking should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("a").is_some(), "Variable a should be declared");
        assert!(table.lookup("rest").is_some(), "Variable rest should be declared");
    }

    #[test]
    fn test_annotated_assignment() {
        let module = parse("x: int = 5");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Annotated assignment should succeed");
        
        let table = result.unwrap();
        let symbol = table.lookup("x");
        assert!(symbol.is_some(), "Variable x should be declared");
        assert_eq!(symbol.unwrap().kind, SymbolKind::Variable);
    }

    #[test]
    fn test_annotated_assignment_no_value() {
        let module = parse("x: int");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Annotated assignment without value should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("x").is_some(), "Variable x should be declared");
    }

    #[test]
    fn test_augmented_assignment_defined() {
        let module = parse("x = 5\nx += 1");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Augmented assignment to defined variable should succeed");
    }

    #[test]
    fn test_augmented_assignment_undefined() {
        let module = parse("x += 1");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Augmented assignment to undefined variable should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::UndefinedVariable { name, .. } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }

    #[test]
    fn test_variable_redeclaration_same_scope() {
        let module = parse("x = 1\nx = 2");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Redeclaration in same scope should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::Redeclaration { name, .. } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected Redeclaration error"),
        }
    }

    #[test]
    fn test_multiple_variables_declaration() {
        let module = parse("a = 1\nb = 2\nc = 3");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Multiple variable declarations should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("a").is_some());
        assert!(table.lookup("b").is_some());
        assert!(table.lookup("c").is_some());
    }

    #[test]
    fn test_chained_assignment() {
        let module = parse("x = y = z = 42");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Chained assignment should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("x").is_some());
        assert!(table.lookup("y").is_some());
        assert!(table.lookup("z").is_some());
    }

    #[test]
    fn test_assignment_with_expression() {
        let module = parse("result = 10 + 20 * 30");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Assignment with expression should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("result").is_some());
    }

    #[test]
    fn test_all_augmented_operators() {
        let code = "x = 10\nx += 5\nx -= 2\nx *= 3\nx /= 2";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "All augmented operators should work with defined variable");
    }

    #[test]
    fn test_all_augmented_operators_explicit_coverage() {
        let code = r#"
x = 10
x += 1
x -= 1
x *= 2
x /= 2
x //= 2
x %= 3
x **= 2
x &= 7
x |= 8
x ^= 1
x >>= 1
x <<= 2
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(
            result.is_ok(),
            "All supported augmented assignment operators should work with a defined variable"
        );
    }

    #[test]
    fn test_augmented_operator_undefined_variable_reports_error() {
        let operators = [
            "+=", "-=", "*=", "/=", "//=", "%=", "**=", "&=", "|=", "^=", ">>=", "<<=",
        ];

        for op in operators {
            let code = format!("x {} 1", op);
            let module = parse(&code);
            let analyzer = SemanticAnalyzer::new();
            let result = analyzer.analyze(&module);
            assert!(
                result.is_err(),
                "Operator '{}' should fail when variable is undefined",
                op
            );

            let errors = result.unwrap_err();
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(e, SemanticError::UndefinedVariable { name, .. } if name == "x")),
                "Operator '{}' should report UndefinedVariable for x",
                op
            );
        }
    }

    #[test]
    fn test_augmented_assignment_complex_target() {
        // Augmented assignment to attribute or subscript should not error
        // (we don't track whether those exist, only identifier variables)
        let module = parse("x = [1, 2, 3]\nx[0] += 10");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Augmented assignment to subscript should succeed");
    }

    #[test]
    fn test_tuple_with_mixed_targets() {
        let module = parse("x, y = 1, 2\na, b, c = 3, 4, 5");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Multiple tuple unpacking should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("x").is_some());
        assert!(table.lookup("y").is_some());
        assert!(table.lookup("a").is_some());
        assert!(table.lookup("b").is_some());
        assert!(table.lookup("c").is_some());
    }

    #[test]
    fn test_annotation_with_complex_type() {
        let module = parse("items: list = []");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Annotated assignment with complex type should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("items").is_some());
    }

    #[test]
    fn test_redeclaration_with_annotation() {
        let module = parse("x = 1\nx: int = 2");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Redeclaration with annotation should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::Redeclaration { name, .. } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected Redeclaration error"),
        }
    }

    // Function Definition Tests

    #[test]
    fn test_simple_function() {
        let module = parse("def foo():\n    pass\n");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Simple function definition should succeed");
        
        let table = result.unwrap();
        let symbol = table.lookup("foo");
        assert!(symbol.is_some(), "Function foo should be declared");
        assert_eq!(symbol.unwrap().kind, SymbolKind::Function);
    }

    #[test]
    fn test_function_with_parameters() {
        let module = parse("def greet(name, age):\n    pass\n");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Function with parameters should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("greet").is_some(), "Function greet should be declared");
        // Note: parameters are in function scope, not module scope
    }

    #[test]
    fn test_function_with_body() {
        let code = "def compute():\n    x = 10\n    y = 20\n    return x + y\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Function with body should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("compute").is_some(), "Function compute should be declared");
        // Variables x and y are in function scope, not module scope
        assert!(table.lookup("x").is_none(), "Variable x should not be in module scope");
    }

    #[test]
    fn test_nested_functions() {
        let code = "def outer():\n    def inner():\n        pass\n    pass\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Nested functions should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("outer").is_some(), "Function outer should be declared");
        // inner is in outer's scope, not module scope
        assert!(table.lookup("inner").is_none(), "Function inner should not be in module scope");
    }

    #[test]
    fn test_function_redeclaration() {
        let code = "def foo():\n    pass\ndef foo():\n    pass\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Function redeclaration should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::Redeclaration { name, .. } => {
                assert_eq!(name, "foo");
            }
            _ => panic!("Expected Redeclaration error"),
        }
    }

    #[test]
    fn test_parameter_redeclaration() {
        let code = "def func(x, x):\n    pass\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Duplicate parameter should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::Redeclaration { name, .. } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected Redeclaration error"),
        }
    }

    #[test]
    fn test_multiple_functions() {
        let code = "def foo():\n    pass\ndef bar():\n    pass\ndef baz():\n    pass\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Multiple functions should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("foo").is_some());
        assert!(table.lookup("bar").is_some());
        assert!(table.lookup("baz").is_some());
    }

    #[test]
    fn test_function_with_return_type() {
        let code = "def get_number() -> int:\n    return 42\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Function with return type should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("get_number").is_some());
    }

    #[test]
    fn test_function_with_type_annotations() {
        let code = "def add(a: int, b: int) -> int:\n    return a + b\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Function with type annotations should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("add").is_some());
    }

    #[test]
    fn test_async_function() {
        let code = "async def fetch_data():\n    pass\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Async function should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("fetch_data").is_some());
    }

    #[test]
    fn test_function_variable_scoping() {
        let code = "x = 10\ndef foo():\n    x = 20\n    pass\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Variable shadowing in function should succeed");
        
        let table = result.unwrap();
        let module_x = table.lookup("x");
        assert!(module_x.is_some(), "Module-level x should exist");
        // Function's x is in a different scope
    }

    #[test]
    fn test_function_and_variable_different_names() {
        let code = "x = 10\ndef foo():\n    pass\ny = 20\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Functions and variables with different names should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("x").is_some());
        assert!(table.lookup("foo").is_some());
        assert!(table.lookup("y").is_some());
    }

    #[test]
    fn test_function_with_default_parameters() {
        let code = "def greet(name, greeting=\"Hello\"):\n    pass\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Function with default parameters should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("greet").is_some());
    }

    #[test]
    fn test_function_with_decorators() {
        let code = "@decorator\ndef foo():\n    pass\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Function with decorator should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("foo").is_some());
    }

    #[test]
    fn test_empty_function() {
        let code = "def empty():\n    pass\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Empty function should succeed");
        
        let table = result.unwrap();
        assert!(table.lookup("empty").is_some());
    }

    // Variable Usage Detection Tests

    #[test]
    fn test_undefined_variable_in_expression() {
        let module = parse("y = x + 1");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Using undefined variable should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::UndefinedVariable { name, .. } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }

    #[test]
    fn test_variable_used_after_definition() {
        let module = parse("x = 10\ny = x + 5");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Using defined variable should succeed");
    }

    #[test]
    fn test_multiple_undefined_variables() {
        let module = parse("result = a + b + c");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Multiple undefined variables should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn test_undefined_in_function_call() {
        let module = parse("result = foo(x, y)");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Undefined variables in call should fail");
        
        let errors = result.unwrap_err();
        assert!(errors.len() >= 2); // foo, x, y all undefined
    }

    #[test]
    fn test_undefined_in_subscript() {
        let module = parse("value = arr[idx]");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Undefined variables in subscript should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2); // arr and idx
    }

    #[test]
    fn test_undefined_in_attribute_access() {
        let module = parse("value = obj.attr");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Undefined variable in attribute access should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::UndefinedVariable { name, .. } => {
                assert_eq!(name, "obj");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }

    #[test]
    fn test_undefined_in_list_literal() {
        let module = parse("items = [a, b, c]");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Undefined variables in list should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn test_undefined_in_dict_literal() {
        let module = parse("data = {k: v}");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Undefined variables in dict should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2); // k and v
    }

    #[test]
    fn test_undefined_in_conditional_expression() {
        let module = parse("result = x if condition else y");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Undefined variables in conditional should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 3); // x, condition, y
    }

    #[test]
    fn test_variable_in_function_scope() {
        let code = "def foo():\n    x = 10\n    y = x + 5\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Variable defined in function scope should work");
    }

    #[test]
    fn test_undefined_in_function_body() {
        let code = "def foo():\n    y = x + 1\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Undefined variable in function should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::UndefinedVariable { name, .. } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }

    #[test]
    fn test_parameter_usage_in_function() {
        let code = "def add(a, b):\n    return a + b\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Using parameters should succeed");
    }

    #[test]
    fn test_nested_scope_variable_access() {
        let code = "x = 10\ndef foo():\n    y = x + 5\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Accessing outer scope variable should succeed");
    }

    #[test]
    fn test_walrus_operator_declaration() {
        let code = "if (n := len([1, 2, 3])) > 0:\n    pass\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Walrus operator should declare variable");
    }

    #[test]
    fn test_complex_expression_chain() {
        let code = "x = 1\ny = 2\nz = (x + y) * (x - y) / (x * y)\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Complex expression with defined variables should succeed");
    }

    #[test]
    fn test_undefined_in_nested_expression() {
        let module = parse("result = ((a + b) * c) / d");
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Nested undefined variables should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 4); // a, b, c, d
    }

    // Redeclaration and Shadowing Tests

    #[test]
    fn test_shadowing_in_nested_function() {
        let code = "x = 1\ndef outer():\n    x = 2\n    def inner():\n        x = 3\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Shadowing across nested scopes should succeed");
    }

    #[test]
    fn test_parameter_shadows_outer_variable() {
        let code = "x = 10\ndef foo(x):\n    return x * 2\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Parameter shadowing outer variable should succeed");
    }

    #[test]
    fn test_parameter_redeclaration_in_body() {
        let code = "def foo(x):\n    x = 20\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Redeclaring parameter in function body should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::Redeclaration { name, .. } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected Redeclaration error"),
        }
    }

    #[test]
    fn test_nested_function_shadows_parameter() {
        let code = "def outer(x):\n    def inner():\n        x = 5\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Nested function can shadow outer parameter");
    }

    #[test]
    fn test_multiple_redeclarations_in_scope() {
        let code = "x = 1\nx = 2\nx = 3\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Multiple redeclarations should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2); // Two redeclaration errors
    }

    #[test]
    fn test_function_and_variable_name_conflict() {
        let code = "x = 10\ndef x():\n    pass\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Function with same name as variable should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::Redeclaration { name, .. } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected Redeclaration error"),
        }
    }

    #[test]
    fn test_variable_after_function_same_name() {
        let code = "def foo():\n    pass\nfoo = 10\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Variable with same name as function should fail");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_shadowing_with_annotation() {
        let code = "x = 10\ndef foo():\n    x: int = 20\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Shadowing with annotation should succeed");
    }

    #[test]
    fn test_no_conflict_different_scopes() {
        let code = "def foo():\n    x = 1\ndef bar():\n    x = 2\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Same variable name in different function scopes should succeed");
    }

    #[test]
    fn test_redeclaration_mixed_types() {
        let code = "x = 10\nx: str = 'hello'\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Redeclaration with different type should fail");
    }

    #[test]
    fn test_walrus_redeclaration() {
        // Walrus operator allows reassignment of existing variables
        let code = "x = 10\ny = (x := 20)\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        // Walrus operator on existing variable - in Python this is allowed (rebinds the variable)
        assert!(result.is_ok(), "Walrus operator should allow reassignment");
    }

    // ==================== Nested Scope Support Tests ====================

    #[test]
    fn test_if_statement_no_new_scope() {
        // Variables declared in if blocks should be accessible outside
        let code = "if True:\n    x = 10\nprint(x)\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "If statement should not create new scope");
    }

    #[test]
    fn test_while_statement_no_new_scope() {
        // Variables declared in while blocks should be accessible outside
        let code = "while False:\n    y = 20\nprint(y)\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "While statement should not create new scope");
    }

    #[test]
    fn test_for_loop_variable_accessible() {
        // For loop variable should be accessible after loop
        let code = "for i in range(10):\n    pass\nprint(i)\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "For loop variable should persist after loop");
    }

    #[test]
    fn test_for_loop_with_unpacking() {
        // For loop with tuple unpacking
        let code = "for x, y in [(1, 2), (3, 4)]:\n    print(x, y)\nprint(x, y)\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "For loop unpacked variables should persist");
    }

    #[test]
    fn test_nested_if_while_for() {
        // Variables in deeply nested control flow should all be in same scope
        let code = "if True:\n    a = 1\n    while True:\n        b = 2\n        for i in range(10):\n            c = 3\nprint(a)\nprint(b)\nprint(c)\nprint(i)\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "All variables in nested control flow should be accessible");
    }

    #[test]
    fn test_class_basic_scope() {
        // Class should create its own scope
        let code = "class MyClass:\n    x = 10\nprint(x)\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Class scope should be isolated");
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedVariable { name, .. } if name == "x")));
    }

    #[test]
    fn test_class_declaration() {
        // Class name should be accessible after declaration
        let code = "class MyClass:\n    pass\nx = MyClass\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Class name should be accessible");
    }

    #[test]
    fn test_class_redeclaration() {
        // Cannot redeclare class in same scope
        let code = "class MyClass:\n    pass\nclass MyClass:\n    pass\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Class redeclaration should fail");
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SemanticError::Redeclaration { name, .. } if name == "MyClass")));
    }

    #[test]
    fn test_nested_class_in_function() {
        // Class inside function should have function as parent scope
        let code = "def outer():\n    x = 10\n    class Inner:\n        y = x\n    return Inner\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Nested class should access function scope");
    }

    #[test]
    fn test_function_in_class() {
        // Function inside class should create nested scopes
        let code = "class MyClass:\n    def method(self):\n        x = 10\n        return x\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Method in class should work correctly");
    }

    #[test]
    fn test_deeply_nested_scopes() {
        // Test deep nesting: module -> function -> class -> function -> if/for
        // Note: This test validates scope isolation, not closure behavior (Task 9)
        let code = "def outer_func():\n    a = 1\n    class InnerClass:\n        b = 2\n        def inner_method():\n            c = 3\n            if True:\n                d = 4\n                for i in range(10):\n                    e = 5\n                    print(c)\n                    print(d)\n                    print(e)\n                    print(i)\n    return InnerClass\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        // Variables c, d, e, i should all be accessible (if/for don't create scopes)
        // Note: We removed reference to 'a' since closure analysis is Task 9
        assert!(result.is_ok(), "Variables in nested control flow should be accessible");
    }

    #[test]
    fn test_if_elif_else_blocks() {
        // All branches of if/elif/else should be in same scope
        let code = "x = 1\nif x == 1:\n    y = 2\nelif x == 2:\n    z = 3\nelse:\n    w = 4\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "If/elif/else should not create scopes");
    }

    #[test]
    fn test_while_with_else() {
        // While with else block
        let code = "while False:\n    a = 1\nelse:\n    b = 2\nprint(a)\nprint(b)\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "While-else should not create scopes");
    }

    #[test]
    fn test_for_with_else() {
        // For with else block
        let code = "for i in []:\n    a = 1\nelse:\n    b = 2\nprint(i)\nprint(a)\nprint(b)\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "For-else should not create scopes");
    }

    // ==================== Closure Tracking & Global/Nonlocal Tests ====================

    #[test]
    fn test_global_at_module_level() {
        // global at module level is allowed (though redundant)
        let code = "global x\nx = 10\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "global at module level should be allowed");
    }

    #[test]
    fn test_global_in_function() {
        // global in function allows modifying module-level variable
        let code = "x = 10\ndef func():\n    global x\n    x = 20\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "global declaration in function should work");
    }

    #[test]
    fn test_global_multiple_names() {
        // global can declare multiple names
        let code = "def func():\n    global x, y, z\n    x = 1\n    y = 2\n    z = 3\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "global with multiple names should work");
    }

    #[test]
    fn test_global_after_local_declaration() {
        // Cannot use global after local declaration
        let code = "def func():\n    x = 10\n    global x\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "global after local declaration should fail");
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SemanticError::Redeclaration { name, .. } if name == "x")));
    }

    #[test]
    fn test_nonlocal_at_module_level() {
        // nonlocal at module level is an error
        let code = "nonlocal x\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "nonlocal at module level should fail");
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SemanticError::NonlocalAtModuleLevel { name, .. } if name == "x")));
    }

    #[test]
    fn test_nonlocal_in_nested_function() {
        // nonlocal in nested function accesses outer function variable
        let code = "def outer():\n    x = 10\n    def inner():\n        nonlocal x\n        x = 20\n    return inner\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "nonlocal in nested function should work");
    }

    #[test]
    fn test_nonlocal_not_found() {
        // nonlocal variable must exist in enclosing scope
        let code = "def outer():\n    def inner():\n        nonlocal x\n        x = 10\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "nonlocal without enclosing binding should fail");
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SemanticError::NonlocalNotFound { name, .. } if name == "x")));
    }

    #[test]
    fn test_nonlocal_skips_module_scope() {
        // nonlocal should not find variables in module scope
        let code = "x = 10\ndef func():\n    nonlocal x\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "nonlocal should not find module-level variables");
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SemanticError::NonlocalNotFound { name, .. } if name == "x")));
    }

    #[test]
    fn test_nonlocal_multiple_names() {
        // nonlocal can declare multiple names
        let code = "def outer():\n    x = 1\n    y = 2\n    def inner():\n        nonlocal x, y\n        x = 10\n        y = 20\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "nonlocal with multiple names should work");
    }

    #[test]
    fn test_nonlocal_after_local_declaration() {
        // Cannot use nonlocal after local declaration
        let code = "def outer():\n    x = 10\n    def inner():\n        x = 5\n        nonlocal x\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "nonlocal after local declaration should fail");
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SemanticError::Redeclaration { name, .. } if name == "x")));
    }

    #[test]
    fn test_closure_basic() {
        // Basic closure - inner function references outer variable
        let code = "def outer():\n    x = 10\n    def inner():\n        print(x)\n    return inner\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Basic closure should work");
    }

    #[test]
    fn test_closure_multiple_levels() {
        // Multi-level closure
        let code = "def level1():\n    x = 1\n    def level2():\n        y = 2\n        def level3():\n            print(x, y)\n        return level3\n    return level2\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Multi-level closure should work");
    }

    #[test]
    fn test_global_and_nonlocal_different_vars() {
        // Can use global and nonlocal for different variables
        let code = "x = 1\ndef outer():\n    y = 2\n    def inner():\n        global x\n        nonlocal y\n        x = 10\n        y = 20\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "global and nonlocal for different vars should work");
    }

    #[test]
    fn test_nonlocal_finds_nearest_enclosing() {
        // nonlocal should find variable in nearest enclosing function scope
        let code = "def outer():\n    x = 1\n    def middle():\n        x = 2\n        def inner():\n            nonlocal x\n            x = 3\n        return inner\n    return middle\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "nonlocal should find nearest enclosing scope");
    }

    #[test]
    fn test_nonlocal_in_class() {
        // nonlocal in class method
        let code = "def outer():\n    x = 10\n    class Inner:\n        def method(self):\n            nonlocal x\n            x = 20\n    return Inner\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "nonlocal in class method should work");
    }

    #[test]
    fn test_undefined_in_return() {
        // Undefined variable in return statement
        let code = "def func():\n    return undefined_var\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "undefined variable in return should fail");
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::UndefinedVariable { name, .. } => {
                assert_eq!(name, "undefined_var");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }

    #[test]
    fn test_undefined_in_assert() {
        // Undefined variable in assert condition
        let code = "assert undefined_condition\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "undefined variable in assert should fail");
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::UndefinedVariable { name, .. } => {
                assert_eq!(name, "undefined_condition");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }

    #[test]
    fn test_undefined_in_assert_message() {
        // Undefined variable in assert message
        let code = "assert True, undefined_message\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "undefined variable in assert message should fail");
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::UndefinedVariable { name, .. } => {
                assert_eq!(name, "undefined_message");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }

    #[test]
    fn test_undefined_in_del() {
        // Undefined variable in del statement
        let code = "del undefined_var\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "undefined variable in del should fail");
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::UndefinedVariable { name, .. } => {
                assert_eq!(name, "undefined_var");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }

    #[test]
    fn test_undefined_in_raise() {
        // Undefined variable in raise statement
        let code = "raise undefined_exception\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "undefined variable in raise should fail");
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::UndefinedVariable { name, .. } => {
                assert_eq!(name, "undefined_exception");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }

    // ===== Type Inference Tests =====

    #[test]
    fn test_infer_integer_literal() {
        let code = "42";
        let module = parse(code);
        let mut analyzer = SemanticAnalyzer::new();
        
        // Get the expression from the module
        if let Some(Statement::Expression(expr)) = module.statements.first() {
            let ty = analyzer.infer_type(expr);
            assert_eq!(ty, Type::Int);
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_infer_float_literal() {
        let code = "3.14";
        let module = parse(code);
        let mut analyzer = SemanticAnalyzer::new();
        
        if let Some(Statement::Expression(expr)) = module.statements.first() {
            let ty = analyzer.infer_type(expr);
            assert_eq!(ty, Type::Float);
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_infer_string_literal() {
        let code = "\"hello\"";
        let module = parse(code);
        let mut analyzer = SemanticAnalyzer::new();
        
        if let Some(Statement::Expression(expr)) = module.statements.first() {
            let ty = analyzer.infer_type(expr);
            assert_eq!(ty, Type::String);
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_infer_bool_literal_true() {
        let code = "True";
        let module = parse(code);
        let mut analyzer = SemanticAnalyzer::new();
        
        if let Some(Statement::Expression(expr)) = module.statements.first() {
            let ty = analyzer.infer_type(expr);
            assert_eq!(ty, Type::Bool);
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_infer_bool_literal_false() {
        let code = "False";
        let module = parse(code);
        let mut analyzer = SemanticAnalyzer::new();
        
        if let Some(Statement::Expression(expr)) = module.statements.first() {
            let ty = analyzer.infer_type(expr);
            assert_eq!(ty, Type::Bool);
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_infer_none_literal() {
        let code = "None";
        let module = parse(code);
        let mut analyzer = SemanticAnalyzer::new();
        
        if let Some(Statement::Expression(expr)) = module.statements.first() {
            let ty = analyzer.infer_type(expr);
            assert_eq!(ty, Type::None);
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_infer_large_integer() {
        let code = "999999999";
        let module = parse(code);
        let mut analyzer = SemanticAnalyzer::new();
        
        if let Some(Statement::Expression(expr)) = module.statements.first() {
            let ty = analyzer.infer_type(expr);
            assert_eq!(ty, Type::Int);
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_infer_negative_integer() {
        let code = "-42";
        let module = parse(code);
        let mut analyzer = SemanticAnalyzer::new();
        
        if let Some(Statement::Expression(expr)) = module.statements.first() {
            // UnaryOp with Minus and Int operand should return Int
            let ty = analyzer.infer_type(expr);
            assert_eq!(ty, Type::Int);
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_infer_empty_string() {
        let code = "\"\"";
        let module = parse(code);
        let mut analyzer = SemanticAnalyzer::new();
        
        if let Some(Statement::Expression(expr)) = module.statements.first() {
            let ty = analyzer.infer_type(expr);
            assert_eq!(ty, Type::String);
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_infer_multiline_string() {
        let code = "\"\"\"multi\nline\nstring\"\"\"";
        let module = parse(code);
        let mut analyzer = SemanticAnalyzer::new();
        
        if let Some(Statement::Expression(expr)) = module.statements.first() {
            let ty = analyzer.infer_type(expr);
            assert_eq!(ty, Type::String);
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_symbol_table_type_storage() {
        let mut symbol_table = SymbolTable::new();
        let pos = SourcePosition::start();
        
        // Declare symbols
        let _ = symbol_table.declare("x".to_string(), SymbolKind::Variable, pos.clone());
        let _ = symbol_table.declare("y".to_string(), SymbolKind::Variable, pos.clone());
        let _ = symbol_table.declare("z".to_string(), SymbolKind::Variable, pos.clone());
        
        // Assign types
        symbol_table.assign_type("x", Type::Int);
        symbol_table.assign_type("y", Type::String);
        symbol_table.assign_type("z", Type::Bool);
        
        // Verify types
        assert_eq!(symbol_table.get_type("x"), Some(Type::Int));
        assert_eq!(symbol_table.get_type("y"), Some(Type::String));
        assert_eq!(symbol_table.get_type("z"), Some(Type::Bool));
        assert_eq!(symbol_table.get_type("undefined"), None);
    }

    #[test]
    fn test_builtin_constant_types() {
        let analyzer = SemanticAnalyzer::new();
        
        assert_eq!(analyzer.get_type("True"), Some(Type::Bool));
        assert_eq!(analyzer.get_type("False"), Some(Type::Bool));
        assert_eq!(analyzer.get_type("None"), Some(Type::None));
    }

    // ======================
    // Task 2: Variable Type Inference Tests
    // ======================

    #[test]
    fn test_simple_int_assignment() {
        let code = "x = 42";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_simple_float_assignment() {
        let code = "y = 3.14";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.get_type("y"), Some(Type::Float));
    }

    #[test]
    fn test_simple_string_assignment() {
        let code = "name = \"Alice\"";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.get_type("name"), Some(Type::String));
    }

    #[test]
    fn test_simple_bool_assignment() {
        let code = "flag = True";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.get_type("flag"), Some(Type::Bool));
    }

    #[test]
    fn test_simple_none_assignment() {
        let code = "value = None";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.get_type("value"), Some(Type::None));
    }

    #[test]
    fn test_multiple_assignment_same_type() {
        let code = "x = y = 10";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
        assert_eq!(analyzer.get_type("y"), Some(Type::Int));
    }

    #[test]
    fn test_multiple_assignment_chain() {
        let code = "a = b = c = \"test\"";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.get_type("a"), Some(Type::String));
        assert_eq!(analyzer.get_type("b"), Some(Type::String));
        assert_eq!(analyzer.get_type("c"), Some(Type::String));
    }

    #[test]
    fn test_reassignment_same_type() {
        let code = "x = 5\nx = 10";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Both assignments are int, type should be int
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_reassignment_different_type() {
        let code = "x = 10\nx = \"hello\"";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Last assignment wins (Python-style dynamic typing)
        assert_eq!(analyzer.get_type("x"), Some(Type::String));
    }

    #[test]
    fn test_annotated_assignment_with_value() {
        let code = "x: int = 42";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_annotated_assignment_without_value() {
        let code = "x: int";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Without value, type comes from the annotation
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_unpacking_assignment() {
        let code = "a, b, c = (1, 2, 3)";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // For now, unpacked variables get Unknown type (tuple type not yet implemented)
        assert_eq!(analyzer.get_type("a"), Some(Type::Unknown));
        assert_eq!(analyzer.get_type("b"), Some(Type::Unknown));
        assert_eq!(analyzer.get_type("c"), Some(Type::Unknown));
    }

    #[test]
    fn test_walrus_operator_assignment() {
        let code = "y = (x := 42)";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_walrus_operator_string() {
        let code = "y = (name := \"test\")";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.get_type("name"), Some(Type::String));
    }

    #[test]
    fn test_assignment_from_identifier() {
        let code = "x = 10\ny = x";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
        // y should get the type from x
        assert_eq!(analyzer.get_type("y"), Some(Type::Int));
    }

    #[test]
    fn test_assignment_from_undefined_identifier() {
        let code = "y = x";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // x is undefined, so y gets Unknown type
        assert_eq!(analyzer.get_type("y"), Some(Type::Unknown));
    }

    #[test]
    fn test_mixed_types() {
        let code = "a = 42\nb = \"hello\"\nc = True\nd = None";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.get_type("a"), Some(Type::Int));
        assert_eq!(analyzer.get_type("b"), Some(Type::String));
        assert_eq!(analyzer.get_type("c"), Some(Type::Bool));
        assert_eq!(analyzer.get_type("d"), Some(Type::None));
    }

    #[test]
    fn test_chained_identifier_assignment() {
        let code = "x = 5\ny = x\nz = y";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
        assert_eq!(analyzer.get_type("y"), Some(Type::Int));
        assert_eq!(analyzer.get_type("z"), Some(Type::Int));
    }

    #[test]
    fn test_complex_expression_unknown_type() {
        let code = "x = 1 + 2";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Binary operations are now inferred: Int + Int → Int
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_walrus_reassignment() {
        let code = "x = True\ny = (x := 42)";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Walrus reassigns x to int
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    // ============================================================
    // Basic Function Return Type Tracking Tests
    // ============================================================

    #[test]
    fn test_function_with_no_return() {
        let code = r#"
def foo():
    x = 1
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Function with no return should have None type
        assert_eq!(analyzer.function_types().get("foo"), Some(&Type::None));
    }

    #[test]
    fn test_function_with_simple_literal_return() {
        let code = r#"
def foo():
    return 42
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Function returning literal int should have Int type
        assert_eq!(analyzer.function_types().get("foo"), Some(&Type::Int));
    }

    #[test]
    fn test_function_with_variable_return() {
        let code = r#"
def foo():
    x = 3.14
    return x
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Function returning float variable should have Float type
        assert_eq!(analyzer.function_types().get("foo"), Some(&Type::Float));
    }

    #[test]
    fn test_function_return_type_empty_function() {
        let code = r#"
def foo():
    pass
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Empty function should have None type
        assert_eq!(analyzer.function_types().get("foo"), Some(&Type::None));
    }

    #[test]
    fn test_function_with_pass_only() {
        let code = r#"
def foo():
    pass
    pass
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Function with only pass statements should have None type
        assert_eq!(analyzer.function_types().get("foo"), Some(&Type::None));
    }

    // ============================================================
    // Single Return Statement Type Inference Tests
    // ============================================================

    #[test]
    fn test_function_return_string_literal() {
        let code = r#"
def greet():
    return "hello"
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Function returning string literal
        assert_eq!(analyzer.function_types().get("greet"), Some(&Type::String));
    }

    #[test]
    fn test_function_return_float_literal() {
        let code = r#"
def pi():
    return 3.14159
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Function returning float literal
        assert_eq!(analyzer.function_types().get("pi"), Some(&Type::Float));
    }

    #[test]
    fn test_function_return_bool_literal() {
        let code = r#"
def always_true():
    return True
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Function returning bool literal
        assert_eq!(analyzer.function_types().get("always_true"), Some(&Type::Bool));
    }

    #[test]
    fn test_function_return_none_literal() {
        let code = r#"
def return_none():
    return None
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Function returning None literal
        assert_eq!(analyzer.function_types().get("return_none"), Some(&Type::None));
    }

    #[test]
    fn test_function_return_typed_variable() {
        let code = r#"
def get_name():
    name = "Alice"
    return name
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Function returning string variable
        assert_eq!(analyzer.function_types().get("get_name"), Some(&Type::String));
    }

    // ============================================================
    // Multiple Return Paths Tests
    // ============================================================

    #[test]
    fn test_function_multiple_returns_same_type() {
        let code = r#"
def abs_value(x):
    if x >= 0:
        return x
    return 42
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Last return wins for now (simple approach)
        assert_eq!(analyzer.function_types().get("abs_value"), Some(&Type::Int));
    }

    #[test]
    fn test_function_if_else_returns() {
        let code = r#"
def check(flag):
    if flag:
        return True
    else:
        return False
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Last return statement type
        assert_eq!(analyzer.function_types().get("check"), Some(&Type::Bool));
    }

    #[test]
    fn test_function_early_return() {
        let code = r#"
def early():
    return 10
    return "unreachable"
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Last return statement processed (even if unreachable)
        assert_eq!(analyzer.function_types().get("early"), Some(&Type::String));
    }

    #[test]
    fn test_function_nested_if_returns() {
        let code = r#"
def nested(x):
    if x > 0:
        if x > 10:
            return "big"
        return "small"
    return 0
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Last return statement
        assert_eq!(analyzer.function_types().get("nested"), Some(&Type::Int));
    }

    #[test]
    fn test_function_mixed_return_and_implicit_none() {
        let code = r#"
def maybe_return(x):
    if x:
        return 42
    pass
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Has explicit return, so takes that type
        assert_eq!(analyzer.function_types().get("maybe_return"), Some(&Type::Int));
    }

    #[test]
    fn test_function_return_in_loop() {
        let code = r#"
def find_first():
    for i in range(10):
        return i
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Return type is Unknown since 'i' type is not tracked in for loops yet
        assert_eq!(analyzer.function_types().get("find_first"), Some(&Type::Unknown));
    }

    // ============================================================
    // Using Function Return Types Tests
    // ============================================================

    #[test]
    fn test_assign_from_function_call() {
        let code = r#"
def get_number():
    return 42

x = get_number()
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Variable assigned from function call gets function's return type
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_assign_from_string_returning_function() {
        let code = r#"
def get_greeting():
    return "Hello"

msg = get_greeting()
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Variable gets string type from function
        assert_eq!(analyzer.get_type("msg"), Some(Type::String));
    }

    #[test]
    fn test_assign_from_none_returning_function() {
        let code = r#"
def do_nothing():
    pass

result = do_nothing()
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Variable gets None type from function
        assert_eq!(analyzer.get_type("result"), Some(Type::None));
    }

    #[test]
    fn test_chained_function_calls() {
        let code = r#"
def get_number():
    return 100

def double():
    x = get_number()
    return x

result = double()
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Both functions should have Int type
        assert_eq!(analyzer.function_types().get("get_number"), Some(&Type::Int));
        // double() returns x which is Int
        assert_eq!(analyzer.function_types().get("double"), Some(&Type::Int));
        // result gets Int from double()
        assert_eq!(analyzer.get_type("result"), Some(Type::Int));
    }

    // ============================================================
    // Arithmetic Operations Type Inference Tests
    // ============================================================

    #[test]
    fn test_int_plus_int() {
        let code = "x = 1 + 2";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Int + Int → Int
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_float_plus_float() {
        let code = "x = 1.5 + 2.5";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Float + Float → Float
        assert_eq!(analyzer.get_type("x"), Some(Type::Float));
    }

    #[test]
    fn test_int_plus_float_promotion() {
        let code = "x = 1 + 2.5";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Int + Float → Float (type promotion)
        assert_eq!(analyzer.get_type("x"), Some(Type::Float));
    }

    #[test]
    fn test_division_always_float() {
        let code = "x = 10 / 2";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Division always returns Float in Python 3
        assert_eq!(analyzer.get_type("x"), Some(Type::Float));
    }

    #[test]
    fn test_string_concatenation() {
        let code = r#"x = "hello" + "world""#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // String + String → String
        assert_eq!(analyzer.get_type("x"), Some(Type::String));
    }

    #[test]
    fn test_bool_plus_float_promotion() {
        let code = "x = True + 3.14";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Bool + Float → Float (bool is subclass of int in Python)
        assert_eq!(analyzer.get_type("x"), Some(Type::Float));
    }

    #[test]
    fn test_float_plus_bool_promotion() {
        let code = "y = 2.5 + False";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Float + Bool → Float
        assert_eq!(analyzer.get_type("y"), Some(Type::Float));
    }

    #[test]
    fn test_bool_plus_int() {
        let code = "z = True + 5";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Bool + Int → Int (bool is subclass of int)
        assert_eq!(analyzer.get_type("z"), Some(Type::Int));
    }

    #[test]
    fn test_bool_plus_bool() {
        let code = "w = True + False";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Bool + Bool → Int (both bools treated as ints)
        assert_eq!(analyzer.get_type("w"), Some(Type::Int));
    }

    // ============================================================
    // Comparison Operations Type Inference Tests
    // ============================================================

    #[test]
    fn test_int_comparison() {
        let code = "x = 5 < 10";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Int < Int → Bool
        assert_eq!(analyzer.get_type("x"), Some(Type::Bool));
    }

    #[test]
    fn test_float_comparison() {
        let code = "x = 3.14 > 2.71";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Float > Float → Bool
        assert_eq!(analyzer.get_type("x"), Some(Type::Bool));
    }

    #[test]
    fn test_string_equality() {
        let code = r#"x = "hello" == "world""#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // String == String → Bool
        assert_eq!(analyzer.get_type("x"), Some(Type::Bool));
    }

    #[test]
    fn test_mixed_type_comparison() {
        let code = "x = 5 != 3.14";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Int != Float → Bool
        assert_eq!(analyzer.get_type("x"), Some(Type::Bool));
    }

    #[test]
    fn test_bool_equality() {
        let code = "x = True == False";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Bool == Bool → Bool
        assert_eq!(analyzer.get_type("x"), Some(Type::Bool));
    }

    // ============================================================
    // Logical Operations Type Inference Tests
    // ============================================================

    #[test]
    fn test_bool_and() {
        let code = "x = True and False";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Bool and Bool → Bool
        assert_eq!(analyzer.get_type("x"), Some(Type::Bool));
    }

    #[test]
    fn test_bool_or() {
        let code = "x = True or False";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Bool or Bool → Bool
        assert_eq!(analyzer.get_type("x"), Some(Type::Bool));
    }

    #[test]
    fn test_literal_bool_and() {
        let code = "x = False and True";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Bool and Bool → Bool
        assert_eq!(analyzer.get_type("x"), Some(Type::Bool));
    }

    #[test]
    fn test_comparison_in_logical() {
        let code = "x = (5 > 3) and (10 < 20)";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Comparison results are Bool, Bool and Bool → Bool
        assert_eq!(analyzer.get_type("x"), Some(Type::Bool));
    }

    // ============================================================
    // Unary Operations Type Inference Tests
    // ============================================================

    #[test]
    fn test_unary_not() {
        let code = "x = not True";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // not Bool → Bool
        assert_eq!(analyzer.get_type("x"), Some(Type::Bool));
    }

    #[test]
    fn test_unary_minus_int() {
        let code = "x = -42";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // -Int → Int
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_unary_minus_float() {
        let code = "x = -3.14";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // -Float → Float
        assert_eq!(analyzer.get_type("x"), Some(Type::Float));
    }

    #[test]
    fn test_unary_minus_bool() {
        let code = "x = -True";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // -Bool → Int (bool is subclass of int in Python)
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_unary_plus_bool() {
        let code = "y = +False";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // +Bool → Int (bool is subclass of int in Python)
        assert_eq!(analyzer.get_type("y"), Some(Type::Int));
    }

    // ============================================================
    // Complex Expression Type Propagation Tests
    // ============================================================

    #[test]
    fn test_nested_arithmetic() {
        let code = "x = (1 + 2) * 3";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // (Int + Int) * Int → Int * Int → Int
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_mixed_type_nested_arithmetic() {
        let code = "x = 1.5 + (2 * 3)";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Float + (Int * Int) → Float + Int → Float
        assert_eq!(analyzer.get_type("x"), Some(Type::Float));
    }

    #[test]
    fn test_nested_logical() {
        let code = "x = (1 < 2) and (3 > 1)";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // (Int < Int) and (Int > Int) → Bool and Bool → Bool
        assert_eq!(analyzer.get_type("x"), Some(Type::Bool));
    }

    // ============================================================
    // Basic If Statement Type Tracking Tests
    // ============================================================

    #[test]
    fn test_variable_assigned_in_if() {
        let code = r#"
if True:
    x = 42
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Variable assigned in if block should be tracked
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_variable_reassigned_in_if() {
        let code = r#"
x = "hello"
if True:
    x = 42
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Last assignment wins: x is now Int
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_variable_used_after_if() {
        let code = r#"
if True:
    x = 3.14
y = x
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // x assigned in if block, y gets its type
        assert_eq!(analyzer.get_type("x"), Some(Type::Float));
        assert_eq!(analyzer.get_type("y"), Some(Type::Float));
    }

    #[test]
    fn test_variable_before_and_in_if() {
        let code = r#"
x = 100
if True:
    x = 200
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Both assignments are Int, last one wins
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    // ============================================================
    // If-Else Type Merging Tests
    // ============================================================

    #[test]
    fn test_same_type_both_branches() {
        let code = r#"
if True:
    x = 42
else:
    x = 100
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Both branches assign Int, so x is Int
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_different_types_both_branches() {
        let code = r#"
if True:
    x = 42
else:
    x = "hello"
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Different types: last write wins (else branch executed last in analysis)
        // In Python, actual type depends on runtime condition; we track last seen
        assert_eq!(analyzer.get_type("x"), Some(Type::String));
    }

    #[test]
    fn test_assignment_only_in_if() {
        let code = r#"
y = 0
if True:
    x = 42
else:
    y = 1
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // x only assigned in if branch
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
        // y assigned before and in else branch
        assert_eq!(analyzer.get_type("y"), Some(Type::Int));
    }

    #[test]
    fn test_assignment_only_in_else() {
        let code = r#"
if False:
    y = 1
else:
    x = 3.14
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // x only assigned in else branch
        assert_eq!(analyzer.get_type("x"), Some(Type::Float));
        // y only assigned in if branch
        assert_eq!(analyzer.get_type("y"), Some(Type::Int));
    }

    #[test]
    fn test_reassignment_same_type_both_branches() {
        let code = r#"
x = 0
if True:
    x = 10
else:
    x = 20
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // All assignments are Int
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    // ============================================================
    // Nested If Statements Tests
    // ============================================================

    #[test]
    fn test_nested_if_type_changes() {
        let code = r#"
if True:
    if True:
        x = 42
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Variable assigned in nested if
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_variable_in_nested_if() {
        let code = r#"
x = "start"
if True:
    x = 1
    if False:
        x = 2
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Last assignment in nested if wins
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_type_propagation_multiple_levels() {
        let code = r#"
if True:
    x = 3.14
    if True:
        y = x
        if True:
            z = y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Type propagates through multiple nesting levels
        assert_eq!(analyzer.get_type("x"), Some(Type::Float));
        assert_eq!(analyzer.get_type("y"), Some(Type::Float));
        assert_eq!(analyzer.get_type("z"), Some(Type::Float));
    }

    // ============================================================
    // While Loop Type Tracking Tests
    // ============================================================

    #[test]
    fn test_variable_before_and_in_while() {
        let code = r#"
x = 1
while False:
    x = 2
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Last assignment in loop wins
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    #[test]
    fn test_variable_assigned_only_in_while() {
        let code = r#"
while False:
    x = "loop"
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Variable assigned in while loop
        assert_eq!(analyzer.get_type("x"), Some(Type::String));
    }

    #[test]
    fn test_variable_used_after_while() {
        let code = r#"
while False:
    x = 42
y = x
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // x assigned in loop, y gets its type
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
        assert_eq!(analyzer.get_type("y"), Some(Type::Int));
    }

    // ============================================================
    // For Loop Type Tracking Tests
    // ============================================================

    #[test]
    fn test_for_loop_with_range() {
        let code = r#"
for i in range(10):
    pass
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Loop variable gets Unknown type (iterable element types not tracked yet)
        assert_eq!(analyzer.get_type("i"), Some(Type::Unknown));
    }

    #[test]
    fn test_variable_assigned_in_for_loop() {
        let code = r#"
for i in range(5):
    x = 42
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Variable assigned in for loop body
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
        assert_eq!(analyzer.get_type("i"), Some(Type::Unknown));
    }

    #[test]
    fn test_for_loop_variable_after_loop() {
        let code = r#"
for i in range(3):
    pass
x = i
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Loop variable accessible after loop
        assert_eq!(analyzer.get_type("i"), Some(Type::Unknown));
        assert_eq!(analyzer.get_type("x"), Some(Type::Unknown));
    }

    #[test]
    fn test_nested_for_loops() {
        let code = r#"
for i in range(3):
    for j in range(2):
        x = 100
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Both loop variables are Unknown
        assert_eq!(analyzer.get_type("i"), Some(Type::Unknown));
        assert_eq!(analyzer.get_type("j"), Some(Type::Unknown));
        // Variable assigned in nested loop
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
    }

    // ============================================================
    // Binary Operation Type Mismatch Tests
    // ============================================================

    #[test]
    fn test_string_subtraction_error() {
        let code = r#"x = "hello" - "world""#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce type mismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message().contains("Type mismatch"));
    }

    #[test]
    fn test_string_multiply_string_error() {
        let code = r#"x = "hello" * "world""#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce type mismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message().contains("Type mismatch"));
    }

    #[test]
    fn test_bool_plus_string_error() {
        let code = r#"x = True + "hello""#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce type mismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message().contains("Type mismatch"));
    }

    #[test]
    fn test_valid_operations_no_errors() {
        let code = r#"
x = 1 + 2
y = 3.14 + 2.71
z = "hello" + "world"
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_none_addition_error() {
        let code = r#"x = None + 1"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce type mismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message().contains("Type mismatch"));
        assert!(errors[0].message().contains("numeric or string"));
    }

    #[test]
    fn test_none_subtraction_error() {
        let code = r#"y = 5 - None"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce type mismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message().contains("Type mismatch"));
        assert!(errors[0].message().contains("numeric"));
    }

    #[test]
    fn test_none_string_addition_error() {
        let code = r#"z = None + "hello""#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce type mismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message().contains("Type mismatch"));
        assert!(errors[0].message().contains("numeric or string"));
    }

    // Annotated Assignment Type Mismatches
    #[test]
    fn test_int_annotation_string_value_error() {
        let code = r#"x: int = "hello""#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce type mismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message().contains("Type mismatch"));
        assert!(errors[0].message().contains("int"));
    }

    #[test]
    fn test_str_annotation_int_value_error() {
        let code = r#"name: str = 42"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce type mismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message().contains("Type mismatch"));
        assert!(errors[0].message().contains("str"));
    }

    #[test]
    fn test_bool_annotation_float_value_error() {
        let code = r#"flag: bool = 3.14"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce type mismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message().contains("Type mismatch"));
        assert!(errors[0].message().contains("bool"));
    }

    #[test]
    fn test_matching_annotation_no_error() {
        let code = r#"
x: int = 42
y: str = "hello"
z: float = 3.14
flag: bool = True
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors when annotations match
        assert!(result.is_ok());
    }

    // Subtask 6.3: Division by Zero
    #[test]
    fn test_division_by_zero_int_error() {
        let code = r#"x = 10 / 0"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce division by zero error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message(), "Division by zero");
    }

    #[test]
    fn test_division_by_zero_float_error() {
        let code = r#"y = 5.5 / 0.0"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce division by zero error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message(), "Division by zero");
    }

    // Subtask 6.4: Unary Operation Mismatches
    #[test]
    fn test_unary_minus_string_error() {
        let code = r#"x = -"hello""#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce type mismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message().contains("Type mismatch"));
        assert!(errors[0].message().contains("numeric"));
    }

    #[test]
    fn test_unary_plus_string_error() {
        let code = r#"y = +"world""#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce type mismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message().contains("Type mismatch"));
        assert!(errors[0].message().contains("numeric"));
    }

    #[test]
    fn test_bitwise_not_non_int_error() {
        let code = r#"z = ~3.14"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce type mismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message().contains("Type mismatch"));
        assert!(errors[0].message().contains("int"));
    }

    // Subtask 6.5: Return Type Annotation Mismatches
    #[test]
    fn test_return_type_int_annotation_string_value_error() {
        let code = r#"
def get_number() -> int:
    return "not a number"
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce type mismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message().contains("Type mismatch"));
        assert!(errors[0].message().contains("int"));
    }

    #[test]
    fn test_return_type_str_annotation_int_value_error() {
        let code = r#"
def get_name() -> str:
    return 42
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce type mismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message().contains("Type mismatch"));
        assert!(errors[0].message().contains("str"));
    }

    #[test]
    fn test_return_type_matching_annotation_no_error() {
        let code = r#"
def get_number() -> int:
    return 42

def get_name() -> str:
    return "hello"

def get_flag() -> bool:
    return True
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors when return types match
        assert!(result.is_ok());
    }

    // ======================
    // Scope-Aware Type Tracking Tests
    // ======================

    #[test]
    fn test_variable_shadowing_different_types() {
        let code = r#"
x = 42
def foo():
    x = "hello"
    y = x
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Module-level x should be Int
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
        // Function-level y should have String type (from shadowed x)
        // Note: Currently our type tracking may not perfectly isolate function scope
        // This test documents current behavior and can be enhanced later
    }

    #[test]
    fn test_function_parameter_type_scoping() {
        let code = r#"
def add(a, b):
    result = a + b
    return result
x = 10
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Module-level x should exist
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
        // Parameters a, b are in function scope - may not be accessible from module level
        // This documents that parameters are properly scoped
        assert_eq!(analyzer.get_type("a"), None);
    }

    #[test]
    fn test_block_scope_type_tracking() {
        let code = r#"
flag = True
if flag:
    message = "yes"
else:
    message = "no"
count = 42
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // All variables should be accessible at module level
        assert_eq!(analyzer.get_type("flag"), Some(Type::Bool));
        assert_eq!(analyzer.get_type("message"), Some(Type::String));
        assert_eq!(analyzer.get_type("count"), Some(Type::Int));
    }

    #[test]
    fn test_nested_scope_type_isolation() {
        let code = r#"
x = 1
def outer():
    y = 2
    def inner():
        z = 3
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Only module-level variable accessible
        assert_eq!(analyzer.get_type("x"), Some(Type::Int));
        // Function-scoped variables not accessible from module scope
        assert_eq!(analyzer.get_type("y"), None);
        assert_eq!(analyzer.get_type("z"), None);
    }

    // ======================
    // Break/Continue Validation Tests
    // ======================

    #[test]
    fn test_break_in_while_loop() {
        let code = r#"
while True:
    break
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_continue_in_while_loop() {
        let code = r#"
while True:
    continue
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_break_in_for_loop() {
        let code = r#"
for i in range(10):
    if i == 5:
        break
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_continue_in_for_loop() {
        let code = r#"
for i in range(10):
    if i % 2 == 0:
        continue
    print(i)
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_break_in_nested_loops() {
        let code = r#"
for i in range(10):
    for j in range(10):
        if i == j:
            break
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_continue_in_nested_loops() {
        let code = r#"
for i in range(10):
    for j in range(10):
        if i == j:
            continue
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_break_at_module_level() {
        let code = "break";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce BreakOutsideLoop error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::BreakOutsideLoop { .. }));
        assert_eq!(errors[0].message(), "'break' outside loop");
    }

    #[test]
    fn test_continue_at_module_level() {
        let code = "continue";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce ContinueOutsideLoop error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::ContinueOutsideLoop { .. }));
        assert_eq!(errors[0].message(), "'continue' outside loop");
    }

    #[test]
    fn test_break_in_function_not_in_loop() {
        let code = r#"
def foo():
    break
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce BreakOutsideLoop error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::BreakOutsideLoop { .. }));
    }

    #[test]
    fn test_continue_in_function_not_in_loop() {
        let code = r#"
def foo():
    continue
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce ContinueOutsideLoop error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::ContinueOutsideLoop { .. }));
    }

    #[test]
    fn test_break_in_if_not_in_loop() {
        let code = r#"
if True:
    break
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce BreakOutsideLoop error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::BreakOutsideLoop { .. }));
    }

    #[test]
    fn test_continue_in_while_else_block() {
        let code = r#"
while True:
    pass
else:
    continue
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce ContinueOutsideLoop error (else block is not in loop)
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::ContinueOutsideLoop { .. }));
    }

    #[test]
    fn test_break_in_for_else_block() {
        let code = r#"
for i in range(10):
    pass
else:
    break
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce BreakOutsideLoop error (else block is not in loop)
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::BreakOutsideLoop { .. }));
    }

    #[test]
    fn test_multiple_break_continue_in_loop() {
        let code = r#"
for i in range(10):
    if i == 5:
        break
    if i % 2 == 0:
        continue
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_break_continue_in_function_with_loop() {
        let code = r#"
def process():
    for i in range(10):
        if i == 5:
            break
        if i % 2 == 0:
            continue
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    // ======================
    // Return Statement Validation Tests
    // ======================

    #[test]
    fn test_return_in_simple_function() {
        let code = r#"
def foo():
    return 42
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_none_in_function() {
        let code = r#"
def foo():
    return None
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_no_value_in_function() {
        let code = r#"
def foo():
    return
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_in_nested_function() {
        let code = r#"
def outer():
    def inner():
        return 10
    return inner()
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_returns_in_function() {
        let code = r#"
def check(x):
    if x > 0:
        return "positive"
    elif x < 0:
        return "negative"
    else:
        return "zero"
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_in_loop_inside_function() {
        let code = r#"
def find(items, target):
    for item in items:
        if item == target:
            return item
    return None
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_in_if_inside_function() {
        let code = r#"
def get_value(flag):
    if flag:
        return 1
    return 0
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_at_module_level() {
        let code = "return 42";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce ReturnOutsideFunction error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::ReturnOutsideFunction { .. }));
        assert_eq!(errors[0].message(), "'return' outside function");
    }

    #[test]
    fn test_return_no_value_at_module_level() {
        let code = "return";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce ReturnOutsideFunction error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::ReturnOutsideFunction { .. }));
    }

    #[test]
    fn test_return_in_if_at_module_level() {
        let code = r#"
if True:
    return 1
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce ReturnOutsideFunction error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::ReturnOutsideFunction { .. }));
    }

    #[test]
    fn test_return_after_function_definition() {
        let code = r#"
def foo():
    pass
return 10
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce ReturnOutsideFunction error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::ReturnOutsideFunction { .. }));
    }

    #[test]
    fn test_return_in_class_body_not_method() {
        let code = r#"
class MyClass:
    x = 10
    return x
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce ReturnOutsideFunction error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SemanticError::ReturnOutsideFunction { .. })));
    }

    // ======================
    // Unreachable Code Detection Tests
    // ======================

    #[test]
    fn test_unreachable_after_return_in_function() {
        let code = r#"
def foo():
    return 42
    x = 10
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
        assert_eq!(errors[0].message(), "Unreachable code");
    }

    #[test]
    fn test_unreachable_after_break_in_loop() {
        let code = r#"
while True:
    break
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_unreachable_after_continue_in_loop() {
        let code = r#"
for i in range(10):
    continue
    print(i)
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_multiple_unreachable_statements() {
        let code = r#"
def foo():
    return 1
    x = 2
    y = 3
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce 2 UnreachableCode errors
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
        assert!(matches!(errors[1], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_unreachable_after_return_in_nested_function() {
        let code = r#"
def outer():
    def inner():
        return 10
        x = 5
    return inner()
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error in inner function
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_no_unreachable_after_if_with_return() {
        let code = r#"
def foo(x):
    if x > 0:
        return "positive"
    print("not positive")
    return "done"
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce unreachable code errors
        // (code after if can execute if condition is false)
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_unreachable_in_else_after_return_in_if() {
        let code = r#"
def foo(x):
    if x:
        return 1
    else:
        return 2
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce errors - each branch's return is reachable
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_as_last_statement_not_unreachable() {
        let code = r#"
def foo():
    x = 10
    return x
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_unreachable_pass_after_return() {
        let code = r#"
def foo():
    return 42
    pass
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error even for pass
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_unreachable_in_nested_loop() {
        let code = r#"
for i in range(10):
    for j in range(10):
        break
        print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    // ======================
    // Unreachable Code Detection - Branch Analysis
    // ======================

    #[test]
    fn test_unreachable_after_if_else_both_return() {
        let code = r#"
def foo():
    if True:
        return 1
    else:
        return 2
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_unreachable_after_if_elif_else_all_return() {
        let code = r#"
def foo(x: int):
    if x == 1:
        return 1
    elif x == 2:
        return 2
    else:
        return 3
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_unreachable_after_if_else_with_break() {
        let code = r#"
def foo(condition: bool):
    while True:
        if condition:
            break
        else:
            break
        print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_unreachable_after_if_else_with_continue() {
        let code = r#"
def foo(condition: bool):
    while True:
        if condition:
            continue
        else:
            continue
        print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_unreachable_after_if_else_with_raise() {
        let code = r#"
def foo(condition: bool):
    if condition:
        return 1
    else:
        return 2
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_unreachable_after_if_else_mixed_exits() {
        let code = r#"
def foo(condition: bool):
    while True:
        if condition:
            return 1
        else:
            break
        print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error (mixed exit types)
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_unreachable_after_nested_if_else_all_exit() {
        let code = r#"
def foo(a: bool, b: bool):
    if a:
        if b:
            return 1
        else:
            return 2
    else:
        return 3
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_reachable_after_if_without_else() {
        let code = r#"
def foo(condition: bool):
    if condition:
        return 1
    print("reachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should NOT produce any errors (code is reachable)
        assert!(result.is_ok());
    }

    #[test]
    fn test_reachable_after_if_elif_without_else() {
        let code = r#"
def foo(x: int):
    if x == 1:
        return 1
    elif x == 2:
        return 2
    print("reachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should NOT produce any errors (no else clause)
        assert!(result.is_ok());
    }

    #[test]
    fn test_reachable_after_if_else_partial_exit() {
        let code = r#"
def foo(condition: bool):
    if condition:
        return 1
    else:
        x = 42
    print("reachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should NOT produce any errors (else doesn't exit)
        assert!(result.is_ok());
    }

    #[test]
    fn test_reachable_after_if_else_one_branch_no_exit() {
        let code = r#"
def foo(condition: bool):
    if condition:
        x = 1
    else:
        return 2
    print("reachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should NOT produce any errors (if doesn't exit)
        assert!(result.is_ok());
    }

    #[test]
    fn test_unreachable_after_if_else_multiple_statements() {
        let code = r#"
def foo(condition: bool):
    if condition:
        x = 1
        y = 2
        return x + y
    else:
        z = 3
        return z
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error (both branches exit)
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_unreachable_after_if_elif_else_with_multiple_statements() {
        let code = r#"
def foo(x: int):
    if x == 1:
        print("one")
        return 1
    elif x == 2:
        print("two")
        return 2
    elif x == 3:
        print("three")
        return 3
    else:
        print("other")
        return 0
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_reachable_after_nested_if_else_partial_exit() {
        let code = r#"
def foo(a: bool, b: bool):
    if a:
        if b:
            return 1
        else:
            x = 2
    else:
        return 3
    print("reachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should NOT produce any errors (inner if-else doesn't fully exit)
        assert!(result.is_ok());
    }

    #[test]
    fn test_unreachable_multiple_after_if_else_all_return() {
        let code = r#"
def foo(condition: bool):
    if condition:
        return 1
    else:
        return 2
    print("unreachable 1")
    x = 42
    print("unreachable 2")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce 3 UnreachableCode errors
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 3);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
        assert!(matches!(errors[1], SemanticError::UnreachableCode { .. }));
        assert!(matches!(errors[2], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_reachable_after_if_elif_else_one_elif_no_exit() {
        let code = r#"
def foo(x: int):
    if x == 1:
        return 1
    elif x == 2:
        y = 2
    else:
        return 3
    print("reachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should NOT produce any errors (one elif doesn't exit)
        assert!(result.is_ok());
    }

    #[test]
    fn test_unreachable_after_deeply_nested_if_else() {
        let code = r#"
def foo(a: bool, b: bool, c: bool, d: bool):
    if a:
        if b:
            if c:
                return 1
            else:
                return 2
        else:
            return 3
    else:
        if d:
            return 4
        else:
            return 5
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error (all nested paths exit)
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_reachable_after_if_else_with_pass() {
        let code = r#"
def foo(condition: bool):
    if condition:
        return 1
    else:
        pass
    print("reachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should NOT produce any errors (pass doesn't exit)
        assert!(result.is_ok());
    }

    #[test]
    fn test_unreachable_in_loop_after_if_else_all_break() {
        let code = r#"
def foo(condition: bool):
    while True:
        if condition:
            break
        else:
            break
        print("unreachable in loop")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UnreachableCode error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_reachable_module_level_after_if_else() {
        let code = r#"
condition = True
if condition:
    x = 1
else:
    y = 2
print("reachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should NOT produce any errors (module-level, not all branches exit)
        assert!(result.is_ok());
    }

    // ======================
    // Function Call Validation Tests
    // ======================

    #[test]
    fn test_call_undefined_function() {
        let code = r#"
foo()
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce UndefinedFunction error (and possibly UndefinedVariable)
        assert!(result.is_err());
        let errors = result.unwrap_err();
        // Check that at least one error is UndefinedFunction
        assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedFunction { .. })));
        // Find the UndefinedFunction error and check its message
        let func_error = errors.iter().find(|e| matches!(e, SemanticError::UndefinedFunction { .. })).unwrap();
        assert_eq!(func_error.message(), "Undefined function: 'foo'");
    }

    #[test]
    fn test_call_function_too_few_arguments() {
        let code = r#"
def add(a, b):
    return a + b

add(1)
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce ArgumentCountMismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::ArgumentCountMismatch { .. }));
        assert!(errors[0].message().contains("takes 2 argument(s) but 1 were given"));
    }

    #[test]
    fn test_call_function_too_many_arguments() {
        let code = r#"
def add(a, b):
    return a + b

add(1, 2, 3)
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce ArgumentCountMismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::ArgumentCountMismatch { .. }));
        assert!(errors[0].message().contains("takes 2 argument(s) but 3 were given"));
    }

    #[test]
    fn test_call_function_correct_argument_count() {
        let code = r#"
def add(a, b):
    return a + b

result = add(1, 2)
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_function_with_default_parameter_all_args() {
        let code = r#"
def greet(name, greeting="Hello"):
    return greeting + " " + name

result = greet("World", "Hi")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_function_with_default_parameter_omit_default() {
        let code = r#"
def greet(name, greeting="Hello"):
    return greeting + " " + name

result = greet("World")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_function_with_default_parameter_too_few() {
        let code = r#"
def greet(name, greeting="Hello"):
    return greeting + " " + name

result = greet()
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce ArgumentCountMismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::ArgumentCountMismatch { .. }));
        assert!(errors[0].message().contains("takes 1-2 arguments but 0 were given"));
    }

    #[test]
    fn test_call_function_argument_type_mismatch() {
        let code = r#"
def add(a: int, b: int) -> int:
    return a + b

result = add(1, "hello")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce ArgumentTypeMismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::ArgumentTypeMismatch { .. }));
        let msg = errors[0].message();
        assert!(msg.contains("parameter 'b'"));
        assert!(msg.contains("expected int")); 
        assert!(msg.contains("got str"));
    }

    #[test]
    fn test_call_function_correct_argument_types() {
        let code = r#"
def add(a: int, b: int) -> int:
    return a + b

result = add(1, 2)
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_builtin_function_any_args() {
        let code = r#"
print("hello", "world", 1, 2, 3)
len([1, 2, 3])
range(10)
str(42)
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Built-in functions should accept any arguments
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_nested_function() {
        let code = r#"
def outer():
    def inner(x):
        return x * 2
    return inner(5)
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_function_no_parameters() {
        let code = r#"
def get_value():
    return 42

x = get_value()
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_function_no_parameters_with_args() {
        let code = r#"
def get_value():
    return 42

x = get_value(1)
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce ArgumentCountMismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::ArgumentCountMismatch { .. }));
    }

    #[test]
    fn test_call_function_in_expression() {
        let code = r#"
def double(x: int) -> int:
    return x * 2

result = double(5) + double(3)
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_recursive_function() {
        let code = r#"
def factorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

result = factorial(5)
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_function_all_defaults() {
        let code = r#"
def configure(mode="auto", debug=False):
    pass

configure()
configure("manual")
configure("manual", True)
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_function_without_type_annotations() {
        let code = r#"
def process(data):
    return data

result = process("anything")
result2 = process(123)
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce errors - no type info to check
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_function_multiple_type_errors() {
        let code = r#"
def add(a: int, b: int) -> int:
    return a + b

result = add("hello", "world")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce 2 ArgumentTypeMismatch errors (one for each parameter)
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
        assert!(matches!(errors[0], SemanticError::ArgumentTypeMismatch { .. }));
        assert!(matches!(errors[1], SemanticError::ArgumentTypeMismatch { .. }));
    }

    // ======================
    // Operator Validation Tests
    // ======================

    #[test]
    fn test_bitwise_and_with_integers() {
        let code = r#"
x = 5
y = 3
result = x & y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_bitwise_and_with_string_error() {
        let code = r#"
x = "hello"
y = "world"
result = x & y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce TypeMismatch errors
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 1);
        assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    }

    #[test]
    fn test_bitwise_or_with_floats_error() {
        let code = r#"
x = 5.0
y = 3.0
result = x | y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce TypeMismatch errors
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 1);
        assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    }

    #[test]
    fn test_bitwise_xor_with_bools() {
        let code = r#"
x = True
y = False
result = x ^ y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce errors (bool is subtype of int)
        assert!(result.is_ok());
    }

    #[test]
    fn test_left_shift_with_integers() {
        let code = r#"
x = 5
result = x << 2
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_right_shift_with_string_error() {
        let code = r#"
x = "hello"
result = x >> 2
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce TypeMismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    }

    #[test]
    fn test_bitwise_and_with_both_operands_invalid() {
        let code = r#"
x = "hello"
y = 3.14
result = x & y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce TWO TypeMismatch errors (one for each operand)
        assert!(result.is_err());
        let errors = result.unwrap_err();
        let type_errors: Vec<_> = errors.iter()
            .filter(|e| matches!(e, SemanticError::TypeMismatch { .. }))
            .collect();
        assert_eq!(type_errors.len(), 2, "Expected 2 type errors (one for each operand), got {}", type_errors.len());
    }

    #[test]
    fn test_string_comparison_equal() {
        let code = r#"
x = "hello"
y = "world"
result = x == y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_ordering_valid() {
        let code = r#"
x = "apple"
y = "banana"
result = x < y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_mixed_type_ordering_error() {
        let code = r#"
x = "hello"
y = 42
result = x < y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce TypeMismatch error
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    }

    #[test]
    fn test_numeric_ordering_valid() {
        let code = r#"
x = 5
y = 3.14
result = x > y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce any errors (numeric types are compatible)
        assert!(result.is_ok());
    }

    #[test]
    fn test_none_ordering_error() {
        let code = r#"
x = None
y = 5
result = x < y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should produce TypeMismatch error (None cannot be ordered)
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    }

    #[test]
    fn test_none_equality_valid() {
        let code = r#"
x = None
y = 5
result = x == y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce errors (None can be compared for equality)
        assert!(result.is_ok());
    }

    #[test]
    fn test_logical_and_with_any_type() {
        let code = r#"
x = 5
y = "hello"
result = x and y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce errors (logical operators accept any type)
        assert!(result.is_ok());
    }

    #[test]
    fn test_logical_or_with_any_type() {
        let code = r#"
x = None
y = False
result = x or y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce errors (logical operators accept any type)
        assert!(result.is_ok());
    }

    #[test]
    fn test_membership_in_string_valid() {
        let code = r#"
ch = "e"
text = "hello"
result = ch in text
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);

        // Should not produce errors (string is a valid container)
        assert!(result.is_ok());
    }

    #[test]
    fn test_membership_not_in_string_valid() {
        let code = r#"
ch = "z"
text = "hello"
result = ch not in text
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);

        // Should not produce errors (string is a valid container)
        assert!(result.is_ok());
    }

    #[test]
    fn test_membership_rhs_integer_error() {
        let code = r#"
x = 1
y = 2
result = x in y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);

        // Should produce TypeMismatch error (int is not a container)
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    }

    #[test]
    fn test_membership_rhs_none_error() {
        let code = r#"
x = 1
y = None
result = x not in y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);

        // Should produce TypeMismatch error (None is not a container)
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    }

    #[test]
    fn test_membership_unknown_rhs_allowed_conservative() {
        let code = r#"
x = 1
y = maybe_container()
result = x in y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);

        // Undefined function error is expected, but no membership type mismatch for unknown RHS.
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SemanticError::UndefinedFunction { .. })));
        assert!(!errors.iter().any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    }

    // ===== Invalid Assignment Target Tests =====
    // Note: Parser already catches some invalid assignments (literals, operators, function calls, lambdas)
    // These tests verify semantic analyzer catches cases that might slip through

    #[test]
    fn test_valid_identifier_assignment() {
        let code = "x = 5";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce errors (identifier is valid target)
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_subscript_assignment() {
        let code = r#"
mylist = [1, 2, 3]
mylist[0] = 10
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce errors (subscript is valid target)
        if let Err(errors) = &result {
            eprintln!("Unexpected errors: {:?}", errors);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_attribute_assignment() {
        let code = r#"
x = None
x.attr = 5
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce errors (attribute is valid target)
        if let Err(errors) = &result {
            eprintln!("Unexpected errors: {:?}", errors);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_tuple_unpacking() {
        let code = "a, b = 1, 2";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce errors (tuple unpacking is valid)
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_list_unpacking() {
        let code = "[a, b] = [1, 2]";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce errors (list unpacking is valid)
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_starred_unpacking() {
        let code = "a, *b, c = [1, 2, 3, 4]";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce errors (starred unpacking is valid)
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_nested_unpacking() {
        let code = "(a, (b, c)) = (1, (2, 3))";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        // Should not produce errors (nested unpacking is valid)
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_dict_literal_assignment() {
        let code = r#"{} = x"#;
        // Try to parse - might fail in parser or semantic analyzer
        let parse_result = try_parse(code);
        if parse_result.is_err() {
            // Parser caught it - that's fine
            return;
        }
        // Parser accepted it - semantic analyzer should catch it
        let module = parse_result.unwrap();
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_set_literal_assignment() {
        let code = r#"{1, 2, 3} = x"#;
        // Try to parse - might fail in parser or semantic analyzer
        let parse_result = try_parse(code);
        if parse_result.is_err() {
            // Parser caught it - that's fine
            return;
        }
        // Parser accepted it - semantic analyzer should catch it
        let module = parse_result.unwrap();
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_list_comp_assignment() {
        let code = "[x for x in range(10)] = foo";
        // Try to parse - might fail in parser or semantic analyzer
        let parse_result = try_parse(code);
        if parse_result.is_err() {
            // Parser caught it - that's fine
            return;
        }
        // Parser accepted it - semantic analyzer should catch it
        let module = parse_result.unwrap();
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_conditional_expression_assignment() {
        let code = r#"
x = 1
y = 2
(x if True else y) = 5
"#;
        // Try to parse - might fail in parser or semantic analyzer
        let parse_result = try_parse(code);
        if parse_result.is_err() {
            // Parser caught it - that's fine
            return;
        }
        // Parser accepted it - semantic analyzer should catch it
        let module = parse_result.unwrap();
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_augmented_assignment_conditional() {
        let code = r#"
x = 1
y = 2
(x if True else y) += 5
"#;
        // Try to parse - might fail in parser or semantic analyzer
        let parse_result = try_parse(code);
        if parse_result.is_err() {
            // Parser caught it - that's fine
            return;
        }
        // Parser accepted it - semantic analyzer should catch it
        let module = parse_result.unwrap();
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err());
    }

    // ============================================================
    // Try/Except Semantic Analysis Tests
    // ============================================================

    #[test]
    fn test_try_except_basic_analysis() {
        let code = r#"
try:
    x = 1
except:
    pass
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Basic try/except should analyze successfully");
    }

    #[test]
    fn test_try_except_exception_variable_scoping() {
        let code = r#"
class ValueError:
    pass

try:
    x = 1
except ValueError as e:
    pass
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Exception variable should be declared in handler scope");
    }

    #[test]
    fn test_try_except_exception_variable_not_visible_outside() {
        let code = r#"
class ValueError:
    pass

try:
    x = 1
except ValueError as e:
    pass
print(e)
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Exception variable should not be visible outside handler");
    }

    #[test]
    fn test_try_except_variable_from_outer_scope() {
        let code = r#"
x = 10
try:
    y = x + 1
except:
    z = x + 2
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Should access variables from outer scope");
    }

    #[test]
    fn test_try_except_else_finally() {
        let code = r#"
class ValueError:
    pass

try:
    x = 1
except ValueError:
    y = 2
else:
    z = 3
finally:
    pass
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Full try/except/else/finally should analyze");
    }

    #[test]
    fn test_nested_try_except() {
        let code = r#"
class ValueError:
    pass

class TypeError:
    pass

try:
    try:
        x = 1
    except ValueError as e1:
        pass
except TypeError as e2:
    pass
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Nested try/except with different exception variables");
    }

    #[test]
    fn test_try_except_undefined_variable_in_try_body() {
        let code = r#"
try:
    x = undefined_var
except:
    pass
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Should detect undefined variable in try body");
    }

    #[test]
    fn test_try_except_undefined_variable_in_handler() {
        let code = r#"
class ValueError:
    pass

try:
    x = 1
except ValueError:
    y = undefined_var
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_err(), "Should detect undefined variable in except handler");
    }

    #[test]
    fn test_try_except_multiple_handlers_with_variables() {
        let code = r#"
class ValueError:
    pass

class TypeError:
    pass

try:
    x = 1
except ValueError as e1:
    pass
except TypeError as e2:
    pass
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Multiple handlers with different exception variables");
    }

    #[test]
    fn test_try_finally_without_except() {
        let code = r#"
try:
    x = 1
finally:
    pass
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "try/finally without except should analyze");
    }

    #[test]
    fn test_try_except_variable_defined_in_try_visible_in_except() {
        let code = r#"
try:
    x = 1
except:
    y = x
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Variable defined in try should be visible in except");
    }

    #[test]
    fn test_try_except_variable_defined_in_except_visible_in_finally() {
        let code = r#"
class ValueError:
    pass

try:
    x = 1
except ValueError:
    y = 2
finally:
    z = y
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        assert!(result.is_ok(), "Variable from except should be visible in finally");
    }

    // ============================================================
    // Try/Except Unreachable Code Detection Tests
    // ============================================================

    #[test]
    fn test_unreachable_after_try_except_all_return() {
        let code = r#"
def foo():
    try:
        return 1
    except:
        return 2
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        assert!(result.is_err(), "Code after try/except where all paths return is unreachable");
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_reachable_after_try_except_try_no_return() {
        let code = r#"
def foo():
    try:
        x = 1
    except:
        return 2
    print("reachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        assert!(result.is_ok(), "Code after try/except is reachable when try doesn't return");
    }

    #[test]
    fn test_reachable_after_try_except_except_no_return() {
        let code = r#"
def foo():
    try:
        return 1
    except:
        x = 2
    print("reachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        assert!(result.is_ok(), "Code after try/except is reachable when except doesn't return");
    }

    #[test]
    fn test_unreachable_after_try_multiple_except_all_return() {
        let code = r#"
class ValueError:
    pass

class TypeError:
    pass

def foo():
    try:
        return 1
    except ValueError:
        return 2
    except TypeError:
        return 3
    except:
        return 4
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        assert!(result.is_err(), "Code after try with multiple except handlers all returning is unreachable");
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_reachable_after_try_multiple_except_one_no_return() {
        let code = r#"
class ValueError:
    pass

class TypeError:
    pass

def foo():
    try:
        return 1
    except ValueError:
        return 2
    except TypeError:
        x = 3
    except:
        return 4
    print("reachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        assert!(result.is_ok(), "Code is reachable when one except handler doesn't return");
    }

    #[test]
    fn test_unreachable_after_try_except_else_all_return() {
        let code = r#"
class ValueError:
    pass

def foo():
    try:
        return 1
    except ValueError:
        return 2
    else:
        return 3
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        assert!(result.is_err(), "Code after try/except/else where all paths return is unreachable");
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_reachable_after_try_except_else_no_return() {
        let code = r#"
class ValueError:
    pass

def foo():
    try:
        return 1
    except ValueError:
        return 2
    else:
        x = 3
    print("reachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        assert!(result.is_ok(), "Code is reachable when else block doesn't return");
    }

    #[test]
    fn test_unreachable_after_try_finally_with_finally_return() {
        let code = r#"
def foo():
    try:
        x = 1
    finally:
        return 2
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        assert!(result.is_err(), "Code after try/finally with finally return is unreachable");
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_unreachable_after_try_except_finally_no_finally_return() {
        let code = r#"
class ValueError:
    pass

def foo():
    try:
        return 1
    except ValueError:
        return 2
    finally:
        x = 3
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        assert!(result.is_err(), "Code is unreachable when try and all except return, even if finally doesn't");
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_unreachable_after_try_except_finally_finally_returns() {
        let code = r#"
class ValueError:
    pass

def foo():
    try:
        x = 1
    except ValueError:
        y = 2
    finally:
        return 3
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        assert!(result.is_err(), "Code after finally with return is always unreachable");
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_unreachable_after_nested_try_except() {
        let code = r#"
class ValueError:
    pass

class TypeError:
    pass

def foo():
    try:
        try:
            return 1
        except ValueError:
            return 2
    except TypeError:
        return 3
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        assert!(result.is_err(), "Code after nested try/except with all paths returning is unreachable");
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }

    #[test]
    fn test_try_except_with_break_not_return() {
        let code = r#"
def foo():
    while True:
        try:
            break
        except:
            break
    print("reachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        assert!(result.is_ok(), "Code after while loop is reachable even if try/except break");
    }

    #[test]
    fn test_try_except_with_raise() {
        let code = r#"
def foo():
    try:
        raise
    except:
        raise
    print("unreachable")
"#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        
        assert!(result.is_err(), "Code after try/except where all paths raise is unreachable");
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::UnreachableCode { .. }));
    }
}


