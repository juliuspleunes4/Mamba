// Control Flow Graph (CFG) implementation
// Represents all possible execution paths through a function

use crate::ast::Statement;
use crate::token::SourcePosition;
use std::collections::HashMap;

/// Unique identifier for a basic block
pub type BlockId = usize;

/// Type of basic block in the CFG
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockKind {
    /// Function entry point
    Entry,
    /// Function exit (return, raise without handler)
    Exit,
    /// Regular statement block
    Normal,
    /// Conditional branch point (if/elif condition)
    Conditional,
    /// Loop header (condition evaluation)
    LoopHeader,
    /// Loop body
    LoopBody,
    /// Exception handler (except block)
    ExceptionHandler,
}

/// A basic block in the control flow graph
/// Contains a sequence of statements with single entry/exit
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Unique identifier for this block
    pub id: BlockId,
    
    /// The kind of block
    pub kind: BlockKind,
    
    /// Statements in this block
    pub statements: Vec<Statement>,
    
    /// Blocks that can follow this block
    pub successors: Vec<BlockId>,
    
    /// Blocks that can precede this block
    pub predecessors: Vec<BlockId>,
    
    /// Source position for error reporting
    pub position: SourcePosition,
}

impl BasicBlock {
    /// Create a new basic block
    pub fn new(id: BlockId, kind: BlockKind, position: SourcePosition) -> Self {
        Self {
            id,
            kind,
            statements: Vec::new(),
            successors: Vec::new(),
            predecessors: Vec::new(),
            position,
        }
    }
    
    /// Add a statement to this block
    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }
    
    /// Add a successor block
    pub fn add_successor(&mut self, successor_id: BlockId) {
        if !self.successors.contains(&successor_id) {
            self.successors.push(successor_id);
        }
    }
    
    /// Add a predecessor block
    pub fn add_predecessor(&mut self, predecessor_id: BlockId) {
        if !self.predecessors.contains(&predecessor_id) {
            self.predecessors.push(predecessor_id);
        }
    }
    
    /// Check if this block is empty (no statements)
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }
    
    /// Check if this block has any successors
    pub fn has_successors(&self) -> bool {
        !self.successors.is_empty()
    }
    
    /// Check if this block has any predecessors
    pub fn has_predecessors(&self) -> bool {
        !self.predecessors.is_empty()
    }
}

/// Control Flow Graph for a function
/// Represents all possible execution paths
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// All basic blocks in the CFG
    blocks: HashMap<BlockId, BasicBlock>,
    
    /// Entry block ID (function start)
    entry_block: BlockId,
    
    /// Exit block IDs (all possible function exits)
    exit_blocks: Vec<BlockId>,
    
    /// Counter for generating unique block IDs
    block_counter: usize,
}

impl ControlFlowGraph {
    /// Create a new CFG with an entry block
    pub fn new() -> Self {
        let mut cfg = Self {
            blocks: HashMap::new(),
            entry_block: 0,
            exit_blocks: Vec::new(),
            block_counter: 0,
        };
        
        // Create entry block
        let entry = BasicBlock::new(
            0,
            BlockKind::Entry,
            SourcePosition::new(0, 0, 0),
        );
        cfg.blocks.insert(0, entry);
        cfg.block_counter = 1;
        
        cfg
    }
    
    /// Create a new basic block and return its ID
    pub fn new_block(&mut self, kind: BlockKind, position: SourcePosition) -> BlockId {
        let id = self.block_counter;
        self.block_counter += 1;
        
        let block = BasicBlock::new(id, kind, position);
        self.blocks.insert(id, block);
        
        id
    }
    
    /// Get a reference to a block
    pub fn get_block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }
    
    /// Get a mutable reference to a block
    pub fn get_block_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(&id)
    }
    
    /// Add an edge from one block to another
    pub fn add_edge(&mut self, from: BlockId, to: BlockId) {
        // Add to successors of source block
        if let Some(from_block) = self.blocks.get_mut(&from) {
            from_block.add_successor(to);
        }
        
        // Add to predecessors of target block
        if let Some(to_block) = self.blocks.get_mut(&to) {
            to_block.add_predecessor(from);
        }
    }
    
    /// Remove an edge from one block to another
    pub fn remove_edge(&mut self, from: BlockId, to: BlockId) {
        // Remove from successors of source block
        if let Some(from_block) = self.blocks.get_mut(&from) {
            from_block.successors.retain(|&id| id != to);
        }
        
        // Remove from predecessors of target block
        if let Some(to_block) = self.blocks.get_mut(&to) {
            to_block.predecessors.retain(|&id| id != from);
        }
    }
    
    /// Get the entry block ID
    pub fn entry(&self) -> BlockId {
        self.entry_block
    }
    
    /// Get all exit block IDs
    pub fn exits(&self) -> &[BlockId] {
        &self.exit_blocks
    }
    
    /// Mark a block as an exit block
    pub fn add_exit_block(&mut self, id: BlockId) {
        if !self.exit_blocks.contains(&id) {
            self.exit_blocks.push(id);
        }
    }
    
    /// Get all block IDs in the CFG
    pub fn block_ids(&self) -> Vec<BlockId> {
        self.blocks.keys().copied().collect()
    }
    
    /// Get the number of blocks in the CFG
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
    
    /// Check if a block exists
    pub fn has_block(&self, id: BlockId) -> bool {
        self.blocks.contains_key(&id)
    }
}

impl Default for ControlFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// CFG Builder - constructs control flow graphs from AST statements
pub struct CFGBuilder {
    /// The CFG being built
    cfg: ControlFlowGraph,
    
    /// The current block being populated with statements
    current_block: BlockId,
    
    /// Stack of exit blocks (for functions)
    exit_block: Option<BlockId>,
    
    /// Stack of loop break targets (for break statements)
    break_targets: Vec<BlockId>,
    
    /// Stack of loop continue targets (for continue statements)
    continue_targets: Vec<BlockId>,
}

impl CFGBuilder {
    /// Create a new CFG builder
    fn new() -> Self {
        let cfg = ControlFlowGraph::new();
        let entry = cfg.entry();
        
        Self {
            cfg,
            current_block: entry,
            exit_block: None,
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
        }
    }
    
    /// Build a CFG from a function definition
    pub fn build_function_cfg(function: &Statement) -> Result<ControlFlowGraph, String> {
        if let Statement::FunctionDef { body, position, .. } = function {
            let mut builder = CFGBuilder::new();
            
            // Create a normal block to start adding statements
            let first_block = builder.cfg.new_block(BlockKind::Normal, position.clone());
            builder.cfg.add_edge(builder.current_block, first_block);
            builder.current_block = first_block;
            
            // Create exit block
            let exit = builder.cfg.new_block(BlockKind::Exit, position.clone());
            builder.exit_block = Some(exit);
            builder.cfg.add_exit_block(exit);
            
            // Process function body
            for statement in body {
                builder.process_statement(statement)?;
            }
            
            // If current block doesn't end with a return and is reachable, connect to exit
            if let Some(current) = builder.cfg.get_block(builder.current_block) {
                // Only connect to exit if block has no successors and has predecessors (is reachable)
                if current.has_successors() == false && current.has_predecessors() {
                    builder.cfg.add_edge(builder.current_block, exit);
                }
            }
            
            Ok(builder.cfg)
        } else {
            Err("Expected FunctionDef statement".to_string())
        }
    }
    
    /// Process a single statement
    fn process_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            // Linear statements - just add to current block
            Statement::Assignment { .. }
            | Statement::AugmentedAssignment { .. }
            | Statement::AnnAssignment { .. }
            | Statement::Expression { .. }
            | Statement::Pass(_) => {
                self.add_statement_to_current_block(statement.clone());
                Ok(())
            }
            
            // Return statement - add edge to exit and start new block
            Statement::Return { position, .. } => {
                self.add_statement_to_current_block(statement.clone());
                
                if let Some(exit) = self.exit_block {
                    self.cfg.add_edge(self.current_block, exit);
                }
                
                // Start new block for any unreachable code after return
                let new_block = self.cfg.new_block(BlockKind::Normal, position.clone());
                self.current_block = new_block;
                
                Ok(())
            }
            
            // Raise statement - add edge to exit (unhandled exception)
            Statement::Raise { position, .. } => {
                self.add_statement_to_current_block(statement.clone());
                
                if let Some(exit) = self.exit_block {
                    self.cfg.add_edge(self.current_block, exit);
                }
                
                // Start new block for any unreachable code after raise
                let new_block = self.cfg.new_block(BlockKind::Normal, position.clone());
                self.current_block = new_block;
                
                Ok(())
            }
            
            // Break/Continue - these should already be validated by semantic analyzer
            Statement::Break(_) => {
                if self.break_targets.is_empty() {
                    Err("Break statement outside loop (should be caught by semantic analyzer)".to_string())
                } else {
                    Ok(())
                }
            }
            
            Statement::Continue(_) => {
                if self.continue_targets.is_empty() {
                    Err("Continue statement outside loop (should be caught by semantic analyzer)".to_string())
                } else {
                    Ok(())
                }
            }
            
            // Control flow statements - to be implemented in later sessions
            Statement::If { .. }
            | Statement::While { .. }
            | Statement::For { .. }
            | Statement::Try { .. } => {
                Err("Control flow statements not yet supported in CFG builder".to_string())
            }
            
            // Other statements - treat as linear for now
            Statement::Import { .. }
            | Statement::FromImport { .. }
            | Statement::Global { .. }
            | Statement::Nonlocal { .. }
            | Statement::Assert { .. }
            | Statement::Del { .. } => {
                self.add_statement_to_current_block(statement.clone());
                Ok(())
            }
            
            // Function and class definitions - treat as statements
            Statement::FunctionDef { .. }
            | Statement::ClassDef { .. } => {
                self.add_statement_to_current_block(statement.clone());
                Ok(())
            }
        }
    }
    
    /// Add a statement to the current block
    fn add_statement_to_current_block(&mut self, statement: Statement) {
        if let Some(block) = self.cfg.get_block_mut(self.current_block) {
            block.add_statement(statement);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_cfg() {
        let cfg = ControlFlowGraph::new();
        
        assert_eq!(cfg.block_count(), 1); // Entry block
        assert_eq!(cfg.entry(), 0);
        assert_eq!(cfg.exits().len(), 0);
    }
    
    #[test]
    fn test_create_block() {
        let mut cfg = ControlFlowGraph::new();
        
        let block1 = cfg.new_block(BlockKind::Normal, SourcePosition::new(1, 0, 0));
        let block2 = cfg.new_block(BlockKind::Normal, SourcePosition::new(2, 0, 0));
        
        assert_eq!(block1, 1);
        assert_eq!(block2, 2);
        assert_eq!(cfg.block_count(), 3); // Entry + 2 new blocks
    }
    
    #[test]
    fn test_add_edge() {
        let mut cfg = ControlFlowGraph::new();
        
        let block1 = cfg.new_block(BlockKind::Normal, SourcePosition::new(1, 0, 0));
        let block2 = cfg.new_block(BlockKind::Normal, SourcePosition::new(2, 0, 0));
        
        cfg.add_edge(block1, block2);
        
        let b1 = cfg.get_block(block1).unwrap();
        assert_eq!(b1.successors, vec![block2]);
        
        let b2 = cfg.get_block(block2).unwrap();
        assert_eq!(b2.predecessors, vec![block1]);
    }
    
    #[test]
    fn test_remove_edge() {
        let mut cfg = ControlFlowGraph::new();
        
        let block1 = cfg.new_block(BlockKind::Normal, SourcePosition::new(1, 0, 0));
        let block2 = cfg.new_block(BlockKind::Normal, SourcePosition::new(2, 0, 0));
        
        cfg.add_edge(block1, block2);
        cfg.remove_edge(block1, block2);
        
        let b1 = cfg.get_block(block1).unwrap();
        assert_eq!(b1.successors.len(), 0);
        
        let b2 = cfg.get_block(block2).unwrap();
        assert_eq!(b2.predecessors.len(), 0);
    }
    
    #[test]
    fn test_add_exit_block() {
        let mut cfg = ControlFlowGraph::new();
        
        let exit1 = cfg.new_block(BlockKind::Exit, SourcePosition::new(1, 0, 0));
        let exit2 = cfg.new_block(BlockKind::Exit, SourcePosition::new(2, 0, 0));
        
        cfg.add_exit_block(exit1);
        cfg.add_exit_block(exit2);
        
        assert_eq!(cfg.exits().len(), 2);
        assert!(cfg.exits().contains(&exit1));
        assert!(cfg.exits().contains(&exit2));
    }
    
    #[test]
    fn test_basic_block_operations() {
        let mut block = BasicBlock::new(1, BlockKind::Normal, SourcePosition::new(1, 0, 0));
        
        assert!(block.is_empty());
        assert!(!block.has_successors());
        assert!(!block.has_predecessors());
        
        // Add a statement
        let stmt = Statement::Pass(SourcePosition::new(1, 0, 0));
        block.add_statement(stmt);
        
        assert!(!block.is_empty());
        assert_eq!(block.statements.len(), 1);
        
        // Add successors and predecessors
        block.add_successor(2);
        block.add_predecessor(0);
        
        assert!(block.has_successors());
        assert!(block.has_predecessors());
        assert_eq!(block.successors, vec![2]);
        assert_eq!(block.predecessors, vec![0]);
    }
    
    #[test]
    fn test_duplicate_edges_ignored() {
        let mut cfg = ControlFlowGraph::new();
        
        let block1 = cfg.new_block(BlockKind::Normal, SourcePosition::new(1, 0, 0));
        let block2 = cfg.new_block(BlockKind::Normal, SourcePosition::new(2, 0, 0));
        
        // Add same edge twice
        cfg.add_edge(block1, block2);
        cfg.add_edge(block1, block2);
        
        let b1 = cfg.get_block(block1).unwrap();
        assert_eq!(b1.successors.len(), 1); // Should only have one successor
        
        let b2 = cfg.get_block(block2).unwrap();
        assert_eq!(b2.predecessors.len(), 1); // Should only have one predecessor
    }
    
    #[test]
    fn test_simple_cfg_construction() {
        let mut cfg = ControlFlowGraph::new();
        
        // Create a simple linear CFG: entry -> block1 -> block2 -> exit
        let block1 = cfg.new_block(BlockKind::Normal, SourcePosition::new(1, 0, 0));
        let block2 = cfg.new_block(BlockKind::Normal, SourcePosition::new(2, 0, 0));
        let exit = cfg.new_block(BlockKind::Exit, SourcePosition::new(3, 0, 0));
        
        cfg.add_edge(cfg.entry(), block1);
        cfg.add_edge(block1, block2);
        cfg.add_edge(block2, exit);
        cfg.add_exit_block(exit);
        
        // Verify structure
        assert_eq!(cfg.block_count(), 4); // entry + 3 blocks
        assert_eq!(cfg.exits().len(), 1);
        
        // Check entry -> block1
        let entry_block = cfg.get_block(cfg.entry()).unwrap();
        assert_eq!(entry_block.successors, vec![block1]);
        
        // Check block1 -> block2
        let b1 = cfg.get_block(block1).unwrap();
        assert_eq!(b1.successors, vec![block2]);
        assert_eq!(b1.predecessors, vec![cfg.entry()]);
        
        // Check block2 -> exit
        let b2 = cfg.get_block(block2).unwrap();
        assert_eq!(b2.successors, vec![exit]);
        assert_eq!(b2.predecessors, vec![block1]);
        
        // Check exit
        let exit_block = cfg.get_block(exit).unwrap();
        assert_eq!(exit_block.successors.len(), 0);
        assert_eq!(exit_block.predecessors, vec![block2]);
    }
    
    #[test]
    fn test_branching_cfg() {
        let mut cfg = ControlFlowGraph::new();
        
        // Create a branching CFG:
        //   entry -> condition
        //   condition -> then_block (true)
        //   condition -> else_block (false)
        //   then_block -> merge
        //   else_block -> merge
        //   merge -> exit
        
        let condition = cfg.new_block(BlockKind::Conditional, SourcePosition::new(1, 0, 0));
        let then_block = cfg.new_block(BlockKind::Normal, SourcePosition::new(2, 0, 0));
        let else_block = cfg.new_block(BlockKind::Normal, SourcePosition::new(3, 0, 0));
        let merge = cfg.new_block(BlockKind::Normal, SourcePosition::new(4, 0, 0));
        let exit = cfg.new_block(BlockKind::Exit, SourcePosition::new(5, 0, 0));
        
        cfg.add_edge(cfg.entry(), condition);
        cfg.add_edge(condition, then_block);
        cfg.add_edge(condition, else_block);
        cfg.add_edge(then_block, merge);
        cfg.add_edge(else_block, merge);
        cfg.add_edge(merge, exit);
        cfg.add_exit_block(exit);
        
        // Verify branching structure
        let cond = cfg.get_block(condition).unwrap();
        assert_eq!(cond.successors.len(), 2);
        assert!(cond.successors.contains(&then_block));
        assert!(cond.successors.contains(&else_block));
        
        // Verify merge point
        let merge_block = cfg.get_block(merge).unwrap();
        assert_eq!(merge_block.predecessors.len(), 2);
        assert!(merge_block.predecessors.contains(&then_block));
        assert!(merge_block.predecessors.contains(&else_block));
    }
    
    // ===== CFG Builder Tests =====
    
    use crate::ast::Expression;  // Only needed for test data
    
    #[test]
    fn test_build_empty_function_cfg() {
        // def foo():
        //     pass
        let func = Statement::FunctionDef {
            name: "foo".to_string(),
            parameters: Vec::new(),
            body: vec![Statement::Pass(SourcePosition::new(2, 0, 0))],
            decorators: Vec::new(),
            is_async: false,
            return_type: None,
            position: SourcePosition::new(1, 0, 0),
        };
        
        let cfg = CFGBuilder::build_function_cfg(&func).unwrap();
        
        // Should have entry, exit, and one normal block for pass
        assert_eq!(cfg.block_count(), 3);
        
        // Entry block should exist
        let entry = cfg.entry();
        assert!(cfg.has_block(entry));
        
        // Entry should have successor (the normal block)
        let entry_block = cfg.get_block(entry).unwrap();
        assert_eq!(entry_block.successors.len(), 1);
        
        // Normal block should have pass statement
        let normal_block_id = entry_block.successors[0];
        let normal_block = cfg.get_block(normal_block_id).unwrap();
        assert_eq!(normal_block.statements.len(), 1);
        assert!(matches!(normal_block.statements[0], Statement::Pass(_)));
        
        // Normal block should connect to exit
        assert_eq!(normal_block.successors.len(), 1);
        let exit_id = normal_block.successors[0];
        let exit_block = cfg.get_block(exit_id).unwrap();
        assert_eq!(exit_block.kind, BlockKind::Exit);
    }
    
    #[test]
    fn test_build_function_with_assignments() {
        // def foo():
        //     x = 1
        //     y = 2
        //     z = x + y
        let func = Statement::FunctionDef {
            name: "foo".to_string(),
            parameters: Vec::new(),
            body: vec![
                Statement::Assignment {
                    targets: Vec::new(),
                    value: Expression::Identifier {
                        name: "dummy".to_string(),
                        position: SourcePosition::new(2, 0, 0),
                    },
                    position: SourcePosition::new(2, 0, 0),
                },
                Statement::Assignment {
                    targets: Vec::new(),
                    value: Expression::Identifier {
                        name: "dummy".to_string(),
                        position: SourcePosition::new(3, 0, 0),
                    },
                    position: SourcePosition::new(3, 0, 0),
                },
                Statement::Assignment {
                    targets: Vec::new(),
                    value: Expression::Identifier {
                        name: "dummy".to_string(),
                        position: SourcePosition::new(4, 0, 0),
                    },
                    position: SourcePosition::new(4, 0, 0),
                },
            ],
            decorators: Vec::new(),
            is_async: false,
            return_type: None,
            position: SourcePosition::new(1, 0, 0),
        };
        
        let cfg = CFGBuilder::build_function_cfg(&func).unwrap();
        
        // Should have entry, exit, and one normal block
        assert_eq!(cfg.block_count(), 3);
        
        // Entry should connect to normal block
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        assert_eq!(entry_block.successors.len(), 1);
        
        // Normal block should have 3 assignment statements
        let normal_block_id = entry_block.successors[0];
        let normal_block = cfg.get_block(normal_block_id).unwrap();
        assert_eq!(normal_block.statements.len(), 3);
        
        // All statements should be assignments
        for stmt in &normal_block.statements {
            assert!(matches!(stmt, Statement::Assignment { .. }));
        }
        
        // Normal block should connect to exit
        assert_eq!(normal_block.successors.len(), 1);
    }
    
    #[test]
    fn test_build_function_with_return() {
        // def foo():
        //     x = 1
        //     return x
        //     y = 2  # unreachable
        let func = Statement::FunctionDef {
            name: "foo".to_string(),
            parameters: Vec::new(),
            body: vec![
                Statement::Assignment {
                    targets: Vec::new(),
                    value: Expression::Identifier {
                        name: "dummy".to_string(),
                        position: SourcePosition::new(2, 0, 0),
                    },
                    position: SourcePosition::new(2, 0, 0),
                },
                Statement::Return {
                    value: None,
                    position: SourcePosition::new(3, 0, 0),
                },
                Statement::Assignment {
                    targets: Vec::new(),
                    value: Expression::Identifier {
                        name: "dummy".to_string(),
                        position: SourcePosition::new(4, 0, 0),
                    },
                    position: SourcePosition::new(4, 0, 0),
                },
            ],
            decorators: Vec::new(),
            is_async: false,
            return_type: None,
            position: SourcePosition::new(1, 0, 0),
        };
        
        let cfg = CFGBuilder::build_function_cfg(&func).unwrap();
        
        // Should have entry, exit, normal block, and unreachable block
        assert_eq!(cfg.block_count(), 4);
        
        // Entry should connect to normal block
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        assert_eq!(entry_block.successors.len(), 1);
        
        // Normal block should have assignment and return
        let normal_block_id = entry_block.successors[0];
        let normal_block = cfg.get_block(normal_block_id).unwrap();
        assert_eq!(normal_block.statements.len(), 2);
        assert!(matches!(normal_block.statements[0], Statement::Assignment { .. }));
        assert!(matches!(normal_block.statements[1], Statement::Return { .. }));
        
        // Normal block should connect to exit
        assert_eq!(normal_block.successors.len(), 1);
        let exit_id = normal_block.successors[0];
        let exit_block = cfg.get_block(exit_id).unwrap();
        assert_eq!(exit_block.kind, BlockKind::Exit);
        
        // Unreachable block should exist with one assignment
        let unreachable_id = cfg.block_ids()
            .into_iter()
            .find(|&id| id != entry && id != normal_block_id && id != exit_id)
            .expect("Should have unreachable block");
        let unreachable_block = cfg.get_block(unreachable_id).unwrap();
        assert_eq!(unreachable_block.statements.len(), 1);
        assert!(matches!(unreachable_block.statements[0], Statement::Assignment { .. }));
        
        // Unreachable block should have no successors
        assert_eq!(unreachable_block.successors.len(), 0);
    }
    
    #[test]
    fn test_build_function_with_multiple_returns() {
        // def foo():
        //     if condition:
        //         return 1
        //     return 2
        // Note: We can't build if statements yet, so just test multiple sequential returns
        // def foo():
        //     return 1
        //     return 2
        let func = Statement::FunctionDef {
            name: "foo".to_string(),
            parameters: Vec::new(),
            body: vec![
                Statement::Return {
                    value: None,
                    position: SourcePosition::new(2, 0, 0),
                },
                Statement::Return {
                    value: None,
                    position: SourcePosition::new(3, 0, 0),
                },
            ],
            decorators: Vec::new(),
            is_async: false,
            return_type: None,
            position: SourcePosition::new(1, 0, 0),
        };
        
        let cfg = CFGBuilder::build_function_cfg(&func).unwrap();
        
        // Should have: entry, normal (first return), exit, unreachable1 (second return), unreachable2 (empty)
        // Note: The second return creates an extra unreachable block, which is acceptable for now
        assert_eq!(cfg.block_count(), 5);
        
        // Entry should connect to first block
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        assert_eq!(entry_block.successors.len(), 1);
        
        // First block should have first return
        let block1_id = entry_block.successors[0];
        let block1 = cfg.get_block(block1_id).unwrap();
        assert_eq!(block1.statements.len(), 1);
        assert!(matches!(block1.statements[0], Statement::Return { .. }));
        
        // First block should connect to exit
        assert_eq!(block1.successors.len(), 1);
        
        // Second return should be in unreachable block
        let exit_id = block1.successors[0];
        
        // Find the unreachable blocks (there will be 2: one with second return, one empty)
        let unreachable_ids: Vec<BlockId> = cfg.block_ids()
            .into_iter()
            .filter(|&id| id != entry && id != block1_id && id != exit_id)
            .collect();
        assert_eq!(unreachable_ids.len(), 2);
        
        // Find the block with the second return statement
        let block2_id = unreachable_ids.iter()
            .find(|&&id| {
                cfg.get_block(id).unwrap().statements.len() == 1
            })
            .copied()
            .expect("Should have block with second return");
        
        let block2 = cfg.get_block(block2_id).unwrap();
        assert!(matches!(block2.statements[0], Statement::Return { .. }));
        
        // Second return block connects to exit (it's still a return statement)
        assert_eq!(block2.successors.len(), 1);
        assert_eq!(block2.successors[0], exit_id);
    }
    
    #[test]
    fn test_build_function_with_raise() {
        // def foo():
        //     x = 1
        //     raise Exception()
        //     y = 2  # unreachable
        let func = Statement::FunctionDef {
            name: "foo".to_string(),
            parameters: Vec::new(),
            body: vec![
                Statement::Assignment {
                    targets: Vec::new(),
                    value: Expression::Identifier {
                        name: "dummy".to_string(),
                        position: SourcePosition::new(2, 0, 0),
                    },
                    position: SourcePosition::new(2, 0, 0),
                },
                Statement::Raise {
                    exception: None,
                    position: SourcePosition::new(3, 0, 0),
                },
                Statement::Assignment {
                    targets: Vec::new(),
                    value: Expression::Identifier {
                        name: "dummy".to_string(),
                        position: SourcePosition::new(4, 0, 0),
                    },
                    position: SourcePosition::new(4, 0, 0),
                },
            ],
            decorators: Vec::new(),
            is_async: false,
            return_type: None,
            position: SourcePosition::new(1, 0, 0),
        };
        
        let cfg = CFGBuilder::build_function_cfg(&func).unwrap();
        
        // Should have entry, exit, normal block, and unreachable block
        assert_eq!(cfg.block_count(), 4);
        
        // Entry should connect to normal block
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        assert_eq!(entry_block.successors.len(), 1);
        
        // Normal block should have assignment and raise
        let normal_block_id = entry_block.successors[0];
        let normal_block = cfg.get_block(normal_block_id).unwrap();
        assert_eq!(normal_block.statements.len(), 2);
        assert!(matches!(normal_block.statements[0], Statement::Assignment { .. }));
        assert!(matches!(normal_block.statements[1], Statement::Raise { .. }));
        
        // Normal block should connect to exit (exception exits function)
        assert_eq!(normal_block.successors.len(), 1);
        let exit_id = normal_block.successors[0];
        let exit_block = cfg.get_block(exit_id).unwrap();
        assert_eq!(exit_block.kind, BlockKind::Exit);
        
        // Unreachable block should exist with one assignment
        let unreachable_id = cfg.block_ids()
            .into_iter()
            .find(|&id| id != entry && id != normal_block_id && id != exit_id)
            .expect("Should have unreachable block");
        let unreachable_block = cfg.get_block(unreachable_id).unwrap();
        assert_eq!(unreachable_block.statements.len(), 1);
        assert!(matches!(unreachable_block.statements[0], Statement::Assignment { .. }));
        
        // Unreachable block should have no successors
        assert_eq!(unreachable_block.successors.len(), 0);
    }
    
    #[test]
    fn test_build_function_rejects_non_function() {
        // Test that build_function_cfg rejects non-function statements
        let stmt = Statement::Pass(SourcePosition::new(1, 0, 0));
        
        let result = CFGBuilder::build_function_cfg(&stmt);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Expected FunctionDef statement");
    }
    
    #[test]
    fn test_control_flow_not_yet_supported() {
        // Test that control flow statements return errors (to be implemented later)
        // def foo():
        //     if True:
        //         pass
        let func = Statement::FunctionDef {
            name: "foo".to_string(),
            parameters: Vec::new(),
            body: vec![
                Statement::If {
                    condition: Expression::Identifier {
                        name: "True".to_string(),
                        position: SourcePosition::new(2, 0, 0),
                    },
                    then_block: vec![Statement::Pass(SourcePosition::new(3, 0, 0))],
                    elif_blocks: Vec::new(),
                    else_block: None,
                    position: SourcePosition::new(2, 0, 0),
                },
            ],
            decorators: Vec::new(),
            is_async: false,
            return_type: None,
            position: SourcePosition::new(1, 0, 0),
        };
        
        let result = CFGBuilder::build_function_cfg(&func);
        assert!(result.is_err());
        assert!(result.err().unwrap().contains("Control flow statements not yet supported"));
    }
}
