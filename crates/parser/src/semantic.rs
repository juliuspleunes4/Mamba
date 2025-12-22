//! Semantic Analysis
//!
//! This module performs semantic analysis on the AST, building a symbol table
//! and detecting semantic errors such as undefined variables, redeclarations, etc.

use crate::ast::{BinaryOperator, Expression, Literal, Module, Statement, UnaryOperator};
use crate::symbol_table::{ScopeKind, SymbolKind, SymbolTable};
use crate::token::SourcePosition;
use crate::types::Type;
use std::collections::HashMap;

/// Type table for tracking inferred types of variables
#[derive(Debug, Clone)]
pub struct TypeTable {
    /// Maps variable name to its inferred type
    types: HashMap<String, Type>,
}

impl TypeTable {
    /// Create a new type table
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    /// Assign a type to a variable
    pub fn assign_type(&mut self, name: String, ty: Type) {
        self.types.insert(name, ty);
    }

    /// Get the type of a variable
    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }
}

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
        }
    }
}

/// The semantic analyzer traverses the AST and builds a symbol table
pub struct SemanticAnalyzer {
    /// Symbol table tracking all declarations and scopes
    symbol_table: SymbolTable,
    /// Type table tracking inferred types
    type_table: TypeTable,
    /// Function return types
    function_types: HashMap<String, Type>,
    /// Current function being analyzed
    current_function: Option<String>,
    /// Collected semantic errors
    errors: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Self {
        let mut symbol_table = SymbolTable::new();
        let mut type_table = TypeTable::new();
        
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
        type_table.assign_type("True".to_string(), Type::Bool);
        type_table.assign_type("False".to_string(), Type::Bool);
        type_table.assign_type("None".to_string(), Type::None);
        
        Self {
            symbol_table,
            type_table,
            function_types: HashMap::new(),
            current_function: None,
            errors: Vec::new(),
        }
    }

    /// Analyze a module and return the symbol table or errors
    ///
    /// Returns Ok(symbol_table) if no errors, Err(errors) if errors found
    pub fn analyze(mut self, module: &Module) -> Result<SymbolTable, Vec<SemanticError>> {
        // Visit all statements in the module
        for statement in &module.statements {
            self.visit_statement(statement);
        }

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
        for statement in &module.statements {
            self.visit_statement(statement);
        }
        self
    }

    /// For testing: get type_table reference
    #[cfg(test)]
    pub fn type_table(&self) -> &TypeTable {
        &self.type_table
    }

    /// For testing: get function_types reference
    #[cfg(test)]
    pub fn function_types(&self) -> &HashMap<String, Type> {
        &self.function_types
    }

    /// Infer the type of an expression
    fn infer_type(&self, expr: &Expression) -> Type {
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
                self.type_table.get_type(name).cloned().unwrap_or(Type::Unknown)
            },
            Expression::Call { function, .. } => {
                // If calling a function, return its inferred return type
                if let Expression::Identifier { name, .. } = &**function {
                    self.function_types.get(name).cloned().unwrap_or(Type::Unknown)
                } else {
                    Type::Unknown
                }
            },
            Expression::BinaryOp { left, op, right, .. } => {
                let left_type = self.infer_type(left);
                let right_type = self.infer_type(right);
                self.infer_binary_op_type(op, &left_type, &right_type)
            },
            Expression::UnaryOp { op, operand, .. } => {
                let operand_type = self.infer_type(operand);
                self.infer_unary_op_type(op, &operand_type)
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
                    (Type::String, Type::String) => Type::String,
                    _ => Type::Unknown,
                }
            },
            Subtract | Multiply | Modulo | Power | FloorDivide => {
                match (left, right) {
                    (Type::Int, Type::Int) => Type::Int,
                    (Type::Float, Type::Float) => Type::Float,
                    (Type::Int, Type::Float) | (Type::Float, Type::Int) => Type::Float,
                    _ => Type::Unknown,
                }
            },
            Divide => {
                // In Python 3, division always returns float
                match (left, right) {
                    (Type::Int, Type::Int) | 
                    (Type::Float, Type::Float) |
                    (Type::Int, Type::Float) | 
                    (Type::Float, Type::Int) => Type::Float,
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
            // Other operations return Unknown for now
            _ => Type::Unknown,
        }
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
                self.type_table.assign_type(name.clone(), typ.clone());
            },
            Expression::Tuple { elements, .. } | Expression::List { elements, .. } => {
                // For unpacking, all variables get the same type (for now)
                for elem in elements {
                    self.assign_type_to_names(elem, typ);
                }
            },
            _ => {
                // Other target types (subscript, attribute) - skip for now
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
                
                // Infer the type of the value
                let value_type = self.infer_type(value);
                
                // Extract and declare all target variables, then assign type
                for target in targets {
                    self.extract_and_declare_names(target, position);
                    self.assign_type_to_names(target, &value_type);
                }
            }

            // AnnAssignment - track typed variable declarations and infer types
            Statement::AnnAssignment { target, value, position, .. } => {
                // Infer type from value if present, otherwise Unknown
                let inferred_type = if let Some(val) = value {
                    self.visit_expression(val);
                    self.infer_type(val)
                } else {
                    Type::Unknown
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
                self.type_table.assign_type(target.clone(), inferred_type);
            }

            // AugmentedAssignment - check variable exists before augmenting
            Statement::AugmentedAssignment { target, value, position, .. } => {
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
            Statement::FunctionDef { name, parameters, body, position, .. } => {
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
                
                // Initialize function return type to None
                self.function_types.insert(name.clone(), Type::None);

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

                // Analyze function body
                for statement in body {
                    self.visit_statement(statement);
                }

                // Exit function scope
                self.symbol_table.exit_scope();
                
                // Restore previous function context
                self.current_function = prev_function;
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

                // Analyze class body
                for statement in body {
                    self.visit_statement(statement);
                }

                // Exit class scope
                self.symbol_table.exit_scope();
            }

            // If - no new scope in Python, just visit all parts
            Statement::If { condition, then_block, elif_blocks, else_block, .. } => {
                // Visit condition
                self.visit_expression(condition);
                
                // Visit then block
                for statement in then_block {
                    self.visit_statement(statement);
                }
                
                // Visit elif blocks
                for (elif_condition, elif_body) in elif_blocks {
                    self.visit_expression(elif_condition);
                    for statement in elif_body {
                        self.visit_statement(statement);
                    }
                }
                
                // Visit else block
                if let Some(else_body) = else_block {
                    for statement in else_body {
                        self.visit_statement(statement);
                    }
                }
            }

            // While - no new scope in Python, just visit condition and body
            Statement::While { condition, body, else_block, .. } => {
                // Visit condition
                self.visit_expression(condition);
                
                // Visit body
                for statement in body {
                    self.visit_statement(statement);
                }
                
                // Visit else block if present
                if let Some(else_body) = else_block {
                    for statement in else_body {
                        self.visit_statement(statement);
                    }
                }
            }

            // For - declare loop variable in current scope, no new scope
            Statement::For { target, iter, body, else_block, position } => {
                // Visit iterator expression first
                self.visit_expression(iter);
                
                // Declare loop variable(s) in current scope
                self.extract_and_declare_names(target, position);
                
                // Visit body
                for statement in body {
                    self.visit_statement(statement);
                }
                
                // Visit else block if present
                if let Some(else_body) = else_block {
                    for statement in else_body {
                        self.visit_statement(statement);
                    }
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
            Statement::Return { value, .. } => {
                if let Some(expr) = value {
                    self.visit_expression(expr);
                    
                    // Infer return type and update function type
                    if let Some(func_name) = &self.current_function {
                        let return_type = self.infer_type(expr);
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

            // Statements with no expressions to visit
            Statement::Pass(_)
            | Statement::Break(_)
            | Statement::Continue(_) => {
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
            Expression::Call { function, arguments, .. } => {
                self.visit_expression(function);
                for arg in arguments {
                    self.visit_expression(arg);
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
                self.type_table.assign_type(target.clone(), value_type);
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

    /// Helper to parse code and create an analyzer
    fn parse(code: &str) -> Module {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().expect("Tokenize should succeed");
        let mut parser = Parser::new(tokens);
        parser.parse().expect("Parse should succeed")
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
    fn test_type_table_storage() {
        let mut type_table = TypeTable::new();
        
        type_table.assign_type("x".to_string(), Type::Int);
        type_table.assign_type("y".to_string(), Type::String);
        type_table.assign_type("z".to_string(), Type::Bool);
        
        assert_eq!(type_table.get_type("x"), Some(&Type::Int));
        assert_eq!(type_table.get_type("y"), Some(&Type::String));
        assert_eq!(type_table.get_type("z"), Some(&Type::Bool));
        assert_eq!(type_table.get_type("undefined"), None);
    }

    #[test]
    fn test_builtin_constant_types() {
        let analyzer = SemanticAnalyzer::new();
        
        assert_eq!(analyzer.type_table.get_type("True"), Some(&Type::Bool));
        assert_eq!(analyzer.type_table.get_type("False"), Some(&Type::Bool));
        assert_eq!(analyzer.type_table.get_type("None"), Some(&Type::None));
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
        
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Int));
    }

    #[test]
    fn test_simple_float_assignment() {
        let code = "y = 3.14";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.type_table().get_type("y"), Some(&Type::Float));
    }

    #[test]
    fn test_simple_string_assignment() {
        let code = "name = \"Alice\"";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.type_table().get_type("name"), Some(&Type::String));
    }

    #[test]
    fn test_simple_bool_assignment() {
        let code = "flag = True";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.type_table().get_type("flag"), Some(&Type::Bool));
    }

    #[test]
    fn test_simple_none_assignment() {
        let code = "value = None";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.type_table().get_type("value"), Some(&Type::None));
    }

    #[test]
    fn test_multiple_assignment_same_type() {
        let code = "x = y = 10";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Int));
        assert_eq!(analyzer.type_table().get_type("y"), Some(&Type::Int));
    }

    #[test]
    fn test_multiple_assignment_chain() {
        let code = "a = b = c = \"test\"";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.type_table().get_type("a"), Some(&Type::String));
        assert_eq!(analyzer.type_table().get_type("b"), Some(&Type::String));
        assert_eq!(analyzer.type_table().get_type("c"), Some(&Type::String));
    }

    #[test]
    fn test_reassignment_same_type() {
        let code = "x = 5\nx = 10";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Both assignments are int, type should be int
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Int));
    }

    #[test]
    fn test_reassignment_different_type() {
        let code = "x = 10\nx = \"hello\"";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Last assignment wins (Python-style dynamic typing)
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::String));
    }

    #[test]
    fn test_annotated_assignment_with_value() {
        let code = "x: int = 42";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Int));
    }

    #[test]
    fn test_annotated_assignment_without_value() {
        let code = "x: int";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Without value, type is Unknown (we don't parse annotations yet)
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Unknown));
    }

    #[test]
    fn test_unpacking_assignment() {
        let code = "a, b, c = (1, 2, 3)";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // For now, unpacked variables get Unknown type (tuple type not yet implemented)
        assert_eq!(analyzer.type_table().get_type("a"), Some(&Type::Unknown));
        assert_eq!(analyzer.type_table().get_type("b"), Some(&Type::Unknown));
        assert_eq!(analyzer.type_table().get_type("c"), Some(&Type::Unknown));
    }

    #[test]
    fn test_walrus_operator_assignment() {
        let code = "y = (x := 42)";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Int));
    }

    #[test]
    fn test_walrus_operator_string() {
        let code = "y = (name := \"test\")";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.type_table().get_type("name"), Some(&Type::String));
    }

    #[test]
    fn test_assignment_from_identifier() {
        let code = "x = 10\ny = x";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Int));
        // y should get the type from x
        assert_eq!(analyzer.type_table().get_type("y"), Some(&Type::Int));
    }

    #[test]
    fn test_assignment_from_undefined_identifier() {
        let code = "y = x";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // x is undefined, so y gets Unknown type
        assert_eq!(analyzer.type_table().get_type("y"), Some(&Type::Unknown));
    }

    #[test]
    fn test_mixed_types() {
        let code = "a = 42\nb = \"hello\"\nc = True\nd = None";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.type_table().get_type("a"), Some(&Type::Int));
        assert_eq!(analyzer.type_table().get_type("b"), Some(&Type::String));
        assert_eq!(analyzer.type_table().get_type("c"), Some(&Type::Bool));
        assert_eq!(analyzer.type_table().get_type("d"), Some(&Type::None));
    }

    #[test]
    fn test_chained_identifier_assignment() {
        let code = "x = 5\ny = x\nz = y";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Int));
        assert_eq!(analyzer.type_table().get_type("y"), Some(&Type::Int));
        assert_eq!(analyzer.type_table().get_type("z"), Some(&Type::Int));
    }

    #[test]
    fn test_complex_expression_unknown_type() {
        let code = "x = 1 + 2";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Binary operations are now inferred: Int + Int → Int
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Int));
    }

    #[test]
    fn test_walrus_reassignment() {
        let code = "x = True\ny = (x := 42)";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Walrus reassigns x to int
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Int));
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
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Int));
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
        assert_eq!(analyzer.type_table().get_type("msg"), Some(&Type::String));
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
        assert_eq!(analyzer.type_table().get_type("result"), Some(&Type::None));
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
        assert_eq!(analyzer.type_table().get_type("result"), Some(&Type::Int));
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
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Int));
    }

    #[test]
    fn test_float_plus_float() {
        let code = "x = 1.5 + 2.5";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Float + Float → Float
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Float));
    }

    #[test]
    fn test_int_plus_float_promotion() {
        let code = "x = 1 + 2.5";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Int + Float → Float (type promotion)
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Float));
    }

    #[test]
    fn test_division_always_float() {
        let code = "x = 10 / 2";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Division always returns Float in Python 3
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Float));
    }

    #[test]
    fn test_string_concatenation() {
        let code = r#"x = "hello" + "world""#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // String + String → String
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::String));
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
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Bool));
    }

    #[test]
    fn test_float_comparison() {
        let code = "x = 3.14 > 2.71";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Float > Float → Bool
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Bool));
    }

    #[test]
    fn test_string_equality() {
        let code = r#"x = "hello" == "world""#;
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // String == String → Bool
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Bool));
    }

    #[test]
    fn test_mixed_type_comparison() {
        let code = "x = 5 != 3.14";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Int != Float → Bool
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Bool));
    }

    #[test]
    fn test_bool_equality() {
        let code = "x = True == False";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Bool == Bool → Bool
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Bool));
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
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Bool));
    }

    #[test]
    fn test_bool_or() {
        let code = "x = True or False";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Bool or Bool → Bool
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Bool));
    }

    #[test]
    fn test_literal_bool_and() {
        let code = "x = False and True";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Bool and Bool → Bool
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Bool));
    }

    #[test]
    fn test_comparison_in_logical() {
        let code = "x = (5 > 3) and (10 < 20)";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Comparison results are Bool, Bool and Bool → Bool
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Bool));
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
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Bool));
    }

    #[test]
    fn test_unary_minus_int() {
        let code = "x = -42";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // -Int → Int
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Int));
    }

    #[test]
    fn test_unary_minus_float() {
        let code = "x = -3.14";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // -Float → Float
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Float));
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
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Int));
    }

    #[test]
    fn test_mixed_type_nested_arithmetic() {
        let code = "x = 1.5 + (2 * 3)";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // Float + (Int * Int) → Float + Int → Float
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Float));
    }

    #[test]
    fn test_nested_logical() {
        let code = "x = (1 < 2) and (3 > 1)";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let analyzer = analyzer.analyze_with_types(&module);
        
        // (Int < Int) and (Int > Int) → Bool and Bool → Bool
        assert_eq!(analyzer.type_table().get_type("x"), Some(&Type::Bool));
    }
}

