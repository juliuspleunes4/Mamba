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
        Self {
            symbol_table: SymbolTable::new(),
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

            // TODO: ClassDef - track class declarations
            Statement::ClassDef { .. } => {
                // TODO: Declare class in current scope
                // TODO: Enter new class scope
                // TODO: Visit body
                // TODO: Exit class scope
            }

            // TODO: If - handle scoped blocks
            Statement::If { .. } => {
                // TODO: Visit condition
                // TODO: Visit body (potentially new scope)
                // TODO: Visit orelse
            }

            // TODO: While - handle scoped blocks
            Statement::While { .. } => {
                // TODO: Visit condition
                // TODO: Visit body (potentially new scope)
            }

            // TODO: For - track loop variable
            Statement::For { .. } => {
                // TODO: Visit target
                // TODO: Visit iter
                // TODO: Declare loop variable
                // TODO: Visit body
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
            // TODO: Identifier - check if variable is defined
            Expression::Identifier { .. } => {
                // TODO: Check if name exists in symbol table
            }

            // TODO: Lambda - track lambda parameters
            Expression::Lambda { .. } => {
                // TODO: Enter new scope
                // TODO: Declare parameters
                // TODO: Visit body
                // TODO: Exit scope
            }

            // TODO: ListComp/SetComp/DictComp - handle comprehension scopes
            Expression::ListComp { .. }
            | Expression::SetComp { .. }
            | Expression::DictComp { .. } => {
                // TODO: Enter new scope
                // TODO: Visit generators (declare loop variables)
                // TODO: Visit element/key/value
                // TODO: Exit scope
            }

            // Literals - no semantic analysis needed
            Expression::Literal(_) => {}

            // TODO: Compound expressions - visit children
            Expression::BinaryOp { .. }
            | Expression::UnaryOp { .. }
            | Expression::Call { .. }
            | Expression::Attribute { .. }
            | Expression::Subscript { .. }
            | Expression::List { .. }
            | Expression::Tuple { .. }
            | Expression::Set { .. }
            | Expression::Dict { .. }
            | Expression::Starred { .. }
            | Expression::Parenthesized { .. }
            | Expression::Conditional { .. }
            | Expression::AssignmentExpr { .. }
            | Expression::GeneratorExpr { .. } => {
                // TODO: Visit all child expressions
            }
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
}
