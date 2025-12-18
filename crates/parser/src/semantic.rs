//! Semantic Analysis
//!
//! This module performs semantic analysis on the AST, building a symbol table
//! and detecting semantic errors such as undefined variables, redeclarations, etc.

use crate::ast::{Expression, Module, Statement};
use crate::symbol_table::{ScopeKind, SymbolKind, SymbolTable};
use crate::token::SourcePosition;

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
}

impl SemanticError {
    /// Get the position of the error
    pub fn position(&self) -> &SourcePosition {
        match self {
            SemanticError::UndefinedVariable { position, .. } => position,
            SemanticError::Redeclaration { second_position, .. } => second_position,
            SemanticError::InvalidScope { position, .. } => position,
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
        }
    }
}

/// The semantic analyzer traverses the AST and builds a symbol table
pub struct SemanticAnalyzer {
    /// Symbol table tracking all declarations and scopes
    symbol_table: SymbolTable,
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
        let _ = symbol_table.declare("None".to_string(), SymbolKind::Variable, builtin_pos);
        
        Self {
            symbol_table,
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

    /// Visit a statement and perform semantic analysis
    fn visit_statement(&mut self, statement: &Statement) {
        match statement {
            // Assignment - track variable declarations
            Statement::Assignment { targets, value, position } => {
                // Visit the value expression first
                self.visit_expression(value);
                
                // Extract and declare all target variables
                for target in targets {
                    self.extract_and_declare_names(target, position);
                }
            }

            // AnnAssignment - track typed variable declarations
            Statement::AnnAssignment { target, value, position, .. } => {
                // Visit the value expression if present
                if let Some(val) = value {
                    self.visit_expression(val);
                }
                
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

            // TODO: Global - mark variables as global
            Statement::Global { .. } => {
                // TODO: Track global declarations
            }

            // TODO: Nonlocal - mark variables as nonlocal
            Statement::Nonlocal { .. } => {
                // TODO: Track nonlocal declarations
            }

            // Other statements that don't affect symbol table
            Statement::Return { .. }
            | Statement::Del { .. }
            | Statement::Assert { .. }
            | Statement::Pass(_)
            | Statement::Break(_)
            | Statement::Continue(_)
            | Statement::Raise { .. } => {
                // TODO: Visit child expressions if any
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

            // Assignment expression (walrus operator) - declare and visit
            Expression::AssignmentExpr { target, value, position } => {
                self.visit_expression(value);
                // Declare the target variable
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
                // Declare single variable
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
        // In Python, walrus operator at module level would redeclare
        // But in current implementation, it's treated as assignment
        // This test verifies current behavior - walrus declares in current scope
        let code = "x = 10\ny = (x := 20)\n";
        let module = parse(code);
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&module);
        // Walrus operator on existing variable - in Python this is allowed
        // It rebinds the variable, which our current implementation treats as redeclaration
        assert!(result.is_err(), "Walrus operator redeclaring variable should fail in current implementation");
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::Redeclaration { name, .. } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected Redeclaration error"),
        }
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
}

