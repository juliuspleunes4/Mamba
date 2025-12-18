//! Symbol Table for Semantic Analysis
//!
//! This module implements the symbol table used for tracking declarations,
//! managing scopes, and performing semantic analysis on Mamba code.

use crate::token::SourcePosition;
use std::collections::HashMap;

/// Unique identifier for a scope
pub type ScopeId = usize;

/// The kind of symbol (what the identifier represents)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// A variable (includes loop variables)
    Variable,
    /// A function definition
    Function,
    /// A class definition
    Class,
    /// A function parameter
    Parameter,
}

/// A symbol represents a declared identifier in the code
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// The identifier name
    pub name: String,
    /// What kind of symbol this is
    pub kind: SymbolKind,
    /// Where it was declared
    pub position: SourcePosition,
    /// Which scope it belongs to
    pub scope_id: ScopeId,
    /// Whether this variable is captured by a nested function (for closures)
    pub is_captured: bool,
    /// Whether this variable was declared with `global` keyword
    pub is_global: bool,
    /// Whether this variable was declared with `nonlocal` keyword
    pub is_nonlocal: bool,
}

impl Symbol {
    /// Create a new symbol
    pub fn new(name: String, kind: SymbolKind, position: SourcePosition, scope_id: ScopeId) -> Self {
        Self {
            name,
            kind,
            position,
            scope_id,
            is_captured: false,
            is_global: false,
            is_nonlocal: false,
        }
    }
    
    /// Mark this symbol as captured by a nested function
    pub fn mark_captured(&mut self) {
        self.is_captured = true;
    }
    
    /// Mark this symbol as global
    pub fn mark_global(&mut self) {
        self.is_global = true;
    }
    
    /// Mark this symbol as nonlocal
    pub fn mark_nonlocal(&mut self) {
        self.is_nonlocal = true;
    }
}

/// The kind of scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// Module/file-level scope
    Module,
    /// Function scope
    Function,
    /// Class scope
    Class,
    /// Block scope (if/while/for)
    Block,
}

/// A scope represents a lexical scope where symbols can be declared
#[derive(Debug, Clone)]
pub struct Scope {
    /// Unique identifier for this scope
    pub id: ScopeId,
    /// What kind of scope this is
    pub kind: ScopeKind,
    /// Parent scope (None for module scope)
    pub parent: Option<ScopeId>,
    /// Symbols declared in this scope
    symbols: HashMap<String, Symbol>,
    /// Child scopes
    pub children: Vec<ScopeId>,
}

impl Scope {
    /// Create a new scope
    pub fn new(id: ScopeId, kind: ScopeKind, parent: Option<ScopeId>) -> Self {
        Self {
            id,
            kind,
            parent,
            symbols: HashMap::new(),
            children: Vec::new(),
        }
    }

    /// Insert a symbol into this scope
    ///
    /// Returns true if inserted successfully, false if symbol already exists
    pub fn insert(&mut self, symbol: Symbol) -> bool {
        if self.symbols.contains_key(&symbol.name) {
            return false;
        }
        self.symbols.insert(symbol.name.clone(), symbol);
        true
    }

    /// Look up a symbol in this scope only (not parent scopes)
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// Check if a symbol exists in this scope
    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Get all symbols in this scope
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.values()
    }

    /// Add a child scope
    pub fn add_child(&mut self, child_id: ScopeId) {
        self.children.push(child_id);
    }
}

/// The symbol table manages all scopes and symbols in a program
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// All scopes indexed by their ID
    scopes: HashMap<ScopeId, Scope>,
    /// The currently active scope
    current_scope: ScopeId,
    /// Counter for generating unique scope IDs
    next_scope_id: ScopeId,
}

impl SymbolTable {
    /// Create a new symbol table with a root module scope
    pub fn new() -> Self {
        let mut scopes = HashMap::new();
        let root_scope = Scope::new(0, ScopeKind::Module, None);
        scopes.insert(0, root_scope);

        Self {
            scopes,
            current_scope: 0,
            next_scope_id: 1,
        }
    }

    /// Get the current scope ID
    pub fn current_scope_id(&self) -> ScopeId {
        self.current_scope
    }

    /// Get a reference to a scope by ID
    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(&id)
    }

    /// Get the current scope
    pub fn current_scope(&self) -> &Scope {
        self.scopes.get(&self.current_scope).expect("Current scope should always exist")
    }

    /// Enter a new scope (create child scope and make it current)
    pub fn enter_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let new_id = self.next_scope_id;
        self.next_scope_id += 1;

        let parent_id = self.current_scope;
        let new_scope = Scope::new(new_id, kind, Some(parent_id));

        // Add as child to current scope
        if let Some(parent) = self.scopes.get_mut(&parent_id) {
            parent.add_child(new_id);
        }

        self.scopes.insert(new_id, new_scope);
        self.current_scope = new_id;
        new_id
    }

    /// Exit current scope and return to parent
    ///
    /// Returns true if exited successfully, false if already at root
    pub fn exit_scope(&mut self) -> bool {
        if let Some(scope) = self.scopes.get(&self.current_scope) {
            if let Some(parent_id) = scope.parent {
                self.current_scope = parent_id;
                return true;
            }
        }
        false
    }

    /// Declare a new symbol in the current scope
    ///
    /// Returns Ok(()) if successful, Err with the existing symbol if already declared
    pub fn declare(
        &mut self,
        name: String,
        kind: SymbolKind,
        position: SourcePosition,
    ) -> Result<(), Symbol> {
        let scope_id = self.current_scope;
        let symbol = Symbol::new(name.clone(), kind, position, scope_id);

        if let Some(scope) = self.scopes.get_mut(&scope_id) {
            if scope.insert(symbol.clone()) {
                Ok(())
            } else {
                // Symbol already exists, return it as error
                Err(scope.lookup(&name).unwrap().clone())
            }
        } else {
            // This should never happen - current_scope should always be valid
            Ok(())
        }
    }

    /// Look up a symbol in current scope and all parent scopes
    ///
    /// Returns the symbol if found, None otherwise
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut current_id = self.current_scope;

        loop {
            if let Some(scope) = self.scopes.get(&current_id) {
                if let Some(symbol) = scope.lookup(name) {
                    return Some(symbol);
                }

                // Try parent scope
                if let Some(parent_id) = scope.parent {
                    current_id = parent_id;
                } else {
                    // Reached root, symbol not found
                    return None;
                }
            } else {
                return None;
            }
        }
    }

    /// Look up a symbol in the current scope only (not parent scopes)
    pub fn lookup_current_scope(&self, name: &str) -> Option<&Symbol> {
        self.scopes.get(&self.current_scope)?.lookup(name)
    }
    
    /// Mark a symbol in the current scope as global
    pub fn mark_global(&mut self, name: &str) -> bool {
        if let Some(scope) = self.scopes.get_mut(&self.current_scope) {
            if let Some(symbol) = scope.symbols.get_mut(name) {
                symbol.mark_global();
                return true;
            }
        }
        false
    }
    
    /// Mark a symbol in the current scope as nonlocal
    pub fn mark_nonlocal(&mut self, name: &str) -> bool {
        if let Some(scope) = self.scopes.get_mut(&self.current_scope) {
            if let Some(symbol) = scope.symbols.get_mut(name) {
                symbol.mark_nonlocal();
                return true;
            }
        }
        false
    }
    
    /// Get the kind of the current scope
    pub fn current_scope_kind(&self) -> ScopeKind {
        self.scopes.get(&self.current_scope)
            .map(|s| s.kind)
            .unwrap_or(ScopeKind::Module)
    }
    
    /// Look up a symbol in enclosing function scopes (excluding module scope)
    /// Used for nonlocal declarations
    pub fn lookup_in_enclosing_function_scopes(&self, name: &str) -> Option<&Symbol> {
        let mut current_id = self.current_scope;
        
        // Skip current scope, look in parents
        if let Some(scope) = self.scopes.get(&current_id) {
            if let Some(parent_id) = scope.parent {
                current_id = parent_id;
            } else {
                return None;
            }
        }
        
        // Search through enclosing scopes, stopping at module level
        loop {
            if let Some(scope) = self.scopes.get(&current_id) {
                // Don't search in module scope for nonlocal
                if scope.kind == ScopeKind::Module {
                    return None;
                }
                
                if let Some(symbol) = scope.lookup(name) {
                    return Some(symbol);
                }
                
                // Try parent scope
                if let Some(parent_id) = scope.parent {
                    current_id = parent_id;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
    }

    /// Get all scopes
    pub fn scopes(&self) -> &HashMap<ScopeId, Scope> {
        &self.scopes
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(line: usize, col: usize) -> SourcePosition {
        SourcePosition { line, column: col, offset: 0 }
    }

    #[test]
    fn test_symbol_creation() {
        let symbol = Symbol::new("x".to_string(), SymbolKind::Variable, pos(1, 1), 0);
        assert_eq!(symbol.name, "x");
        assert_eq!(symbol.kind, SymbolKind::Variable);
        assert_eq!(symbol.scope_id, 0);
    }

    #[test]
    fn test_scope_insert_and_lookup() {
        let mut scope = Scope::new(0, ScopeKind::Module, None);
        let symbol = Symbol::new("x".to_string(), SymbolKind::Variable, pos(1, 1), 0);

        assert!(scope.insert(symbol.clone()));
        assert!(scope.contains("x"));
        assert_eq!(scope.lookup("x"), Some(&symbol));
    }

    #[test]
    fn test_scope_duplicate_insert() {
        let mut scope = Scope::new(0, ScopeKind::Module, None);
        let symbol1 = Symbol::new("x".to_string(), SymbolKind::Variable, pos(1, 1), 0);
        let symbol2 = Symbol::new("x".to_string(), SymbolKind::Variable, pos(2, 1), 0);

        assert!(scope.insert(symbol1));
        assert!(!scope.insert(symbol2)); // Should fail
    }

    #[test]
    fn test_symbol_table_creation() {
        let table = SymbolTable::new();
        assert_eq!(table.current_scope_id(), 0);
        assert_eq!(table.current_scope().kind, ScopeKind::Module);
    }

    #[test]
    fn test_symbol_table_declare() {
        let mut table = SymbolTable::new();

        assert!(table.declare("x".to_string(), SymbolKind::Variable, pos(1, 1)).is_ok());
        assert!(table.lookup("x").is_some());
        assert_eq!(table.lookup("x").unwrap().name, "x");
    }

    #[test]
    fn test_symbol_table_redeclaration() {
        let mut table = SymbolTable::new();

        assert!(table.declare("x".to_string(), SymbolKind::Variable, pos(1, 1)).is_ok());
        let result = table.declare("x".to_string(), SymbolKind::Variable, pos(2, 1));
        assert!(result.is_err());
    }

    #[test]
    fn test_symbol_table_enter_exit_scope() {
        let mut table = SymbolTable::new();

        // Enter function scope
        let func_scope_id = table.enter_scope(ScopeKind::Function);
        assert_eq!(table.current_scope_id(), func_scope_id);
        assert_eq!(table.current_scope().kind, ScopeKind::Function);

        // Exit back to module scope
        assert!(table.exit_scope());
        assert_eq!(table.current_scope_id(), 0);
        assert_eq!(table.current_scope().kind, ScopeKind::Module);
    }

    #[test]
    fn test_symbol_table_nested_scopes() {
        let mut table = SymbolTable::new();

        // Declare in module scope
        table.declare("x".to_string(), SymbolKind::Variable, pos(1, 1)).unwrap();

        // Enter function scope
        table.enter_scope(ScopeKind::Function);

        // Can still see parent scope variable
        assert!(table.lookup("x").is_some());

        // Declare new variable in function scope
        table.declare("y".to_string(), SymbolKind::Variable, pos(2, 1)).unwrap();
        assert!(table.lookup("y").is_some());

        // Exit function scope
        table.exit_scope();

        // Can still see x
        assert!(table.lookup("x").is_some());
        // But not y (function scope variable)
        assert!(table.lookup("y").is_none());
    }

    #[test]
    fn test_symbol_table_shadowing() {
        let mut table = SymbolTable::new();

        // Declare x in module scope
        table.declare("x".to_string(), SymbolKind::Variable, pos(1, 1)).unwrap();
        let module_x = table.lookup("x").unwrap().clone();

        // Enter function scope
        table.enter_scope(ScopeKind::Function);

        // Shadow x in function scope (should be allowed)
        table.declare("x".to_string(), SymbolKind::Variable, pos(2, 1)).unwrap();
        let function_x = table.lookup("x").unwrap();

        // Should find the function scope x, not module x
        assert_ne!(function_x.position, module_x.position);
        assert_eq!(function_x.position.line, 2);

        // Exit function scope
        table.exit_scope();

        // Should now find module scope x again
        let found_x = table.lookup("x").unwrap();
        assert_eq!(found_x.position, module_x.position);
    }

    #[test]
    fn test_symbol_table_lookup_current_scope_only() {
        let mut table = SymbolTable::new();

        // Declare in module scope
        table.declare("x".to_string(), SymbolKind::Variable, pos(1, 1)).unwrap();

        // Enter function scope
        table.enter_scope(ScopeKind::Function);

        // x is visible through lookup
        assert!(table.lookup("x").is_some());

        // But not in current scope only
        assert!(table.lookup_current_scope("x").is_none());

        // Declare y in function scope
        table.declare("y".to_string(), SymbolKind::Variable, pos(2, 1)).unwrap();

        // y is visible in current scope
        assert!(table.lookup_current_scope("y").is_some());
    }

    #[test]
    fn test_deeply_nested_scopes() {
        let mut table = SymbolTable::new();

        table.declare("a".to_string(), SymbolKind::Variable, pos(1, 1)).unwrap();

        table.enter_scope(ScopeKind::Function);
        table.declare("b".to_string(), SymbolKind::Variable, pos(2, 1)).unwrap();

        table.enter_scope(ScopeKind::Block);
        table.declare("c".to_string(), SymbolKind::Variable, pos(3, 1)).unwrap();

        // All variables should be visible
        assert!(table.lookup("a").is_some());
        assert!(table.lookup("b").is_some());
        assert!(table.lookup("c").is_some());

        // Exit inner block
        table.exit_scope();
        assert!(table.lookup("a").is_some());
        assert!(table.lookup("b").is_some());
        assert!(table.lookup("c").is_none()); // c is out of scope

        // Exit function
        table.exit_scope();
        assert!(table.lookup("a").is_some());
        assert!(table.lookup("b").is_none()); // b is out of scope
        assert!(table.lookup("c").is_none());
    }
}
