// Control Flow Graph (CFG) implementation
// Represents all possible execution paths through a function

use crate::ast::{Statement, Expression};
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
            Statement::Break(position) => {
                if self.break_targets.is_empty() {
                    Err("Break statement outside loop (should be caught by semantic analyzer)".to_string())
                } else {
                    // Add statement to current block
                    self.add_statement_to_current_block(statement.clone());
                    
                    // Add edge to break target (loop exit)
                    if let Some(&target) = self.break_targets.last() {
                        self.cfg.add_edge(self.current_block, target);
                    }
                    
                    // Start new block for unreachable code after break
                    let new_block = self.cfg.new_block(BlockKind::Normal, position.clone());
                    self.current_block = new_block;
                    
                    Ok(())
                }
            }
            
            Statement::Continue(position) => {
                if self.continue_targets.is_empty() {
                    Err("Continue statement outside loop (should be caught by semantic analyzer)".to_string())
                } else {
                    // Add statement to current block
                    self.add_statement_to_current_block(statement.clone());
                    
                    // Add edge to continue target (loop header)
                    if let Some(&target) = self.continue_targets.last() {
                        self.cfg.add_edge(self.current_block, target);
                    }
                    
                    // Start new block for unreachable code after continue
                    let new_block = self.cfg.new_block(BlockKind::Normal, position.clone());
                    self.current_block = new_block;
                    
                    Ok(())
                }
            }
            
            // Control flow statements - to be implemented in later sessions
            Statement::If { condition, then_block, elif_blocks, else_block, position } => {
                self.process_if_statement(condition, then_block, elif_blocks, else_block, position)
            }
            
            Statement::While { condition, body, else_block, position } => {
                self.process_while_statement(condition, body, else_block, position)
            }
            
            Statement::For { target, iter, body, else_block, position } => {
                self.process_for_statement(target, iter, body, else_block, position)
            }
            
            Statement::Try { .. } => {
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
    
    /// Process an if/elif/else statement
    fn process_if_statement(
        &mut self,
        condition: &Expression,
        then_block: &[Statement],
        elif_blocks: &[(Expression, Vec<Statement>)],
        else_block: &Option<Vec<Statement>>,
        position: &SourcePosition,
    ) -> Result<(), String> {
        // Add condition evaluation to current block as an expression statement
        let condition_stmt = Statement::Expression(condition.clone());
        self.add_statement_to_current_block(condition_stmt);
        
        let condition_block = self.current_block;
        
        // Create then-block
        let then_block_id = self.cfg.new_block(BlockKind::Normal, position.clone());
        self.cfg.add_edge(condition_block, then_block_id);
        self.current_block = then_block_id;
        
        // Process then-block statements
        for stmt in then_block {
            self.process_statement(stmt)?;
        }
        let then_exit = self.current_block;
        
        // Collect all branch exits
        let mut all_branch_exits = vec![then_exit];
        
        // Handle elif and else chains
        let has_else = if !elif_blocks.is_empty() || else_block.is_some() {
            let (elif_exits, has_else_block) = self.process_elif_else_chain(condition_block, elif_blocks, else_block, position)?;
            all_branch_exits.extend(elif_exits);
            has_else_block
        } else {
            // No elif or else - false branch goes directly to merge
            false
        };
        
        // Create merge block
        let merge_block = self.cfg.new_block(BlockKind::Normal, position.clone());
        
        // Connect all branch exits to merge (if they don't already have successors)
        for exit_id in all_branch_exits {
            if let Some(block_ref) = self.cfg.get_block(exit_id) {
                if !block_ref.has_successors() {
                    self.cfg.add_edge(exit_id, merge_block);
                }
            }
        }
        
        // If no else, connect condition directly to merge (false branch)
        if !has_else {
            self.cfg.add_edge(condition_block, merge_block);
        }
        
        // Continue with merge block
        self.current_block = merge_block;
        Ok(())
    }
    
    /// Process elif/else chain, returns (branch_exits, has_else)
    fn process_elif_else_chain(
        &mut self,
        mut previous_condition_block: BlockId,
        elif_blocks: &[(Expression, Vec<Statement>)],
        else_block: &Option<Vec<Statement>>,
        position: &SourcePosition,
    ) -> Result<(Vec<BlockId>, bool), String> {
        let mut branch_exits = Vec::new();
        
        // Process each elif
        for (elif_condition, elif_body) in elif_blocks {
            // Create elif condition block
            let elif_condition_block = self.cfg.new_block(BlockKind::Conditional, position.clone());
            
            // Previous false branch goes to this elif condition
            self.cfg.add_edge(previous_condition_block, elif_condition_block);
            
            // Add condition evaluation
            let condition_stmt = Statement::Expression(elif_condition.clone());
            if let Some(block) = self.cfg.get_block_mut(elif_condition_block) {
                block.add_statement(condition_stmt);
            }
            
            // Create elif then-block
            let elif_then_block = self.cfg.new_block(BlockKind::Normal, position.clone());
            self.cfg.add_edge(elif_condition_block, elif_then_block);
            self.current_block = elif_then_block;
            
            // Process elif body
            for stmt in elif_body {
                self.process_statement(stmt)?;
            }
            
            branch_exits.push(self.current_block);
            previous_condition_block = elif_condition_block;
        }
        
        // Process else block if it exists
        if let Some(else_body) = else_block {
            let else_block_id = self.cfg.new_block(BlockKind::Normal, position.clone());
            
            // Last condition's false branch goes to else
            self.cfg.add_edge(previous_condition_block, else_block_id);
            self.current_block = else_block_id;
            
            // Process else body
            for stmt in else_body {
                self.process_statement(stmt)?;
            }
            
            branch_exits.push(self.current_block);
            Ok((branch_exits, true))
        } else {
            // No else block
            Ok((branch_exits, false))
        }
    }
    
    /// Process a while loop statement
    fn process_while_statement(
        &mut self,
        condition: &Expression,
        body: &[Statement],
        else_block: &Option<Vec<Statement>>,
        position: &SourcePosition,
    ) -> Result<(), String> {
        // Create loop header block for condition evaluation
        let header = self.cfg.new_block(BlockKind::LoopHeader, position.clone());
        
        // Add edge from current block to header
        self.cfg.add_edge(self.current_block, header);
        
        // Add condition evaluation to header block
        if let Some(header_block) = self.cfg.get_block_mut(header) {
            header_block.add_statement(Statement::Expression(condition.clone()));
        }
        
        // Create loop body block
        let body_block = self.cfg.new_block(BlockKind::LoopBody, position.clone());
        
        // Add edge from header to body (true branch)
        self.cfg.add_edge(header, body_block);
        
        // Create loop exit block (merge point after loop)
        let exit_block = self.cfg.new_block(BlockKind::Normal, position.clone());
        
        // Process else block if present
        if let Some(else_stmts) = else_block {
            // Create else block
            let else_blk = self.cfg.new_block(BlockKind::Normal, position.clone());
            
            // False branch from header goes to else
            self.cfg.add_edge(header, else_blk);
            
            // Process else statements
            self.current_block = else_blk;
            for statement in else_stmts {
                self.process_statement(statement)?;
            }
            
            // Connect else to exit if reachable
            if let Some(current) = self.cfg.get_block(self.current_block) {
                if !current.has_successors() {
                    self.cfg.add_edge(self.current_block, exit_block);
                }
            }
        } else {
            // No else block - false branch goes directly to exit
            self.cfg.add_edge(header, exit_block);
        }
        
        // Push loop context for break/continue
        self.break_targets.push(exit_block);
        self.continue_targets.push(header);
        
        // Process loop body
        self.current_block = body_block;
        for statement in body {
            self.process_statement(statement)?;
        }
        
        // Add back-edge from body to header (if body is reachable and doesn't exit)
        if let Some(current) = self.cfg.get_block(self.current_block) {
            if !current.has_successors() {
                self.cfg.add_edge(self.current_block, header);
            }
        }
        
        // Pop loop context
        self.break_targets.pop();
        self.continue_targets.pop();
        
        // Continue execution after the loop
        self.current_block = exit_block;
        
        Ok(())
    }
    
    /// Process a for loop statement
    fn process_for_statement(
        &mut self,
        target: &Expression,
        iter: &Expression,
        body: &[Statement],
        else_block: &Option<Vec<Statement>>,
        position: &SourcePosition,
    ) -> Result<(), String> {
        // Add iterator evaluation to current block
        self.add_statement_to_current_block(Statement::Expression(iter.clone()));
        
        // Create loop header block for iteration check and target assignment
        let header = self.cfg.new_block(BlockKind::LoopHeader, position.clone());
        
        // Add edge from current block to header
        self.cfg.add_edge(self.current_block, header);
        
        // Add target assignment to header block (loop variable)
        if let Some(header_block) = self.cfg.get_block_mut(header) {
            // Create assignment: target = next(iter)
            header_block.add_statement(Statement::Assignment {
                targets: vec![target.clone()],
                value: target.clone(), // Simplified - represents getting next value
                position: position.clone(),
            });
        }
        
        // Create loop body block
        let body_block = self.cfg.new_block(BlockKind::LoopBody, position.clone());
        
        // Add edge from header to body (has next item)
        self.cfg.add_edge(header, body_block);
        
        // Create loop exit block (merge point after loop)
        let exit_block = self.cfg.new_block(BlockKind::Normal, position.clone());
        
        // Process else block if present
        if let Some(else_stmts) = else_block {
            // Create else block
            let else_blk = self.cfg.new_block(BlockKind::Normal, position.clone());
            
            // No more items branch from header goes to else
            self.cfg.add_edge(header, else_blk);
            
            // Process else statements
            self.current_block = else_blk;
            for statement in else_stmts {
                self.process_statement(statement)?;
            }
            
            // Connect else to exit if reachable
            if let Some(current) = self.cfg.get_block(self.current_block) {
                if !current.has_successors() {
                    self.cfg.add_edge(self.current_block, exit_block);
                }
            }
        } else {
            // No else block - no more items goes directly to exit
            self.cfg.add_edge(header, exit_block);
        }
        
        // Push loop context for break/continue
        self.break_targets.push(exit_block);
        self.continue_targets.push(header);
        
        // Process loop body
        self.current_block = body_block;
        for statement in body {
            self.process_statement(statement)?;
        }
        
        // Add back-edge from body to header (if body is reachable and doesn't exit)
        if let Some(current) = self.cfg.get_block(self.current_block) {
            if !current.has_successors() {
                self.cfg.add_edge(self.current_block, header);
            }
        }
        
        // Pop loop context
        self.break_targets.pop();
        self.continue_targets.pop();
        
        // Continue execution after the loop
        self.current_block = exit_block;
        
        Ok(())
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
    fn test_simple_if_no_else() {
        // def foo():
        //     x = 1
        //     if True:
        //         y = 2
        //     z = 3
        let func = Statement::FunctionDef {
            name: "foo".to_string(),
            parameters: Vec::new(),
            body: vec![
                Statement::Assignment {
                    targets: Vec::new(),
                    value: Expression::Identifier {
                        name: "one".to_string(),
                        position: SourcePosition::new(2, 0, 0),
                    },
                    position: SourcePosition::new(2, 0, 0),
                },
                Statement::If {
                    condition: Expression::Identifier {
                        name: "True".to_string(),
                        position: SourcePosition::new(3, 0, 0),
                    },
                    then_block: vec![Statement::Assignment {
                        targets: Vec::new(),
                        value: Expression::Identifier {
                            name: "two".to_string(),
                            position: SourcePosition::new(4, 0, 0),
                        },
                        position: SourcePosition::new(4, 0, 0),
                    }],
                    elif_blocks: Vec::new(),
                    else_block: None,
                    position: SourcePosition::new(3, 0, 0),
                },
                Statement::Assignment {
                    targets: Vec::new(),
                    value: Expression::Identifier {
                        name: "three".to_string(),
                        position: SourcePosition::new(5, 0, 0),
                    },
                    position: SourcePosition::new(5, 0, 0),
                },
            ],
            decorators: Vec::new(),
            is_async: false,
            return_type: None,
            position: SourcePosition::new(1, 0, 0),
        };
        
        let cfg = CFGBuilder::build_function_cfg(&func).unwrap();
        
        // Should have: entry, normal(x=1, if cond), then(y=2), merge(z=3), exit
        assert_eq!(cfg.block_count(), 5);
        
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        assert_eq!(entry_block.successors.len(), 1);
        
        // First normal block has x=1 and condition
        let first_normal = entry_block.successors[0];
        let first_normal_block = cfg.get_block(first_normal).unwrap();
        assert_eq!(first_normal_block.statements.len(), 2); // x=1, condition
        assert_eq!(first_normal_block.successors.len(), 2); // then and merge
        
        // Find then and merge blocks
        let successors = &first_normal_block.successors;
        let then_id = successors[0];
        let then_block = cfg.get_block(then_id).unwrap();
        assert_eq!(then_block.statements.len(), 1); // y=2
        
        // Then block should connect to merge
        assert_eq!(then_block.successors.len(), 1);
        let merge_from_then = then_block.successors[0];
        
        // Merge block should have z=3
        let merge_block = cfg.get_block(merge_from_then).unwrap();
        assert_eq!(merge_block.statements.len(), 1); // z=3
        
        // Merge should connect to exit
        assert_eq!(merge_block.successors.len(), 1);
        
        // Merge should have 2 predecessors: then and condition (false branch)
        assert_eq!(merge_block.predecessors.len(), 2);
    }
    
    #[test]
    fn test_if_else() {
        // def foo():
        //     if True:
        //         x = 1
        //     else:
        //         x = 2
        //     y = 3
        let func = Statement::FunctionDef {
            name: "foo".to_string(),
            parameters: Vec::new(),
            body: vec![
                Statement::If {
                    condition: Expression::Identifier {
                        name: "True".to_string(),
                        position: SourcePosition::new(2, 0, 0),
                    },
                    then_block: vec![Statement::Assignment {
                        targets: Vec::new(),
                        value: Expression::Identifier {
                            name: "one".to_string(),
                            position: SourcePosition::new(3, 0, 0),
                        },
                        position: SourcePosition::new(3, 0, 0),
                    }],
                    elif_blocks: Vec::new(),
                    else_block: Some(vec![Statement::Assignment {
                        targets: Vec::new(),
                        value: Expression::Identifier {
                            name: "two".to_string(),
                            position: SourcePosition::new(5, 0, 0),
                        },
                        position: SourcePosition::new(5, 0, 0),
                    }]),
                    position: SourcePosition::new(2, 0, 0),
                },
                Statement::Assignment {
                    targets: Vec::new(),
                    value: Expression::Identifier {
                        name: "three".to_string(),
                        position: SourcePosition::new(6, 0, 0),
                    },
                    position: SourcePosition::new(6, 0, 0),
                },
            ],
            decorators: Vec::new(),
            is_async: false,
            return_type: None,
            position: SourcePosition::new(1, 0, 0),
        };
        
        let cfg = CFGBuilder::build_function_cfg(&func).unwrap();
        
        // Should have: entry, normal(condition), then(x=1), else(x=2), merge(y=3), exit
        assert_eq!(cfg.block_count(), 6);
        
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        assert_eq!(entry_block.successors.len(), 1);
        
        // Condition block
        let condition_block_id = entry_block.successors[0];
        let condition_block = cfg.get_block(condition_block_id).unwrap();
        assert_eq!(condition_block.statements.len(), 1); // condition
        assert_eq!(condition_block.successors.len(), 2); // then and else
        
        let then_id = condition_block.successors[0];
        let then_block = cfg.get_block(then_id).unwrap();
        assert_eq!(then_block.statements.len(), 1); // x=1
        assert_eq!(then_block.successors.len(), 1); // to merge
        
        let else_id = condition_block.successors[1];
        let else_block = cfg.get_block(else_id).unwrap();
        assert_eq!(else_block.statements.len(), 1); // x=2
        assert_eq!(else_block.successors.len(), 1); // to merge
        
        // Both should go to same merge
        let merge_id = then_block.successors[0];
        assert_eq!(else_block.successors[0], merge_id);
        
        let merge_block = cfg.get_block(merge_id).unwrap();
        assert_eq!(merge_block.statements.len(), 1); // y=3
        assert_eq!(merge_block.predecessors.len(), 2); // from then and else
    }
    
    #[test]
    fn test_if_elif_else() {
        // def foo():
        //     if cond1:
        //         x = 1
        //     elif cond2:
        //         x = 2
        //     else:
        //         x = 3
        //     y = 4
        let func = Statement::FunctionDef {
            name: "foo".to_string(),
            parameters: Vec::new(),
            body: vec![
                Statement::If {
                    condition: Expression::Identifier {
                        name: "cond1".to_string(),
                        position: SourcePosition::new(2, 0, 0),
                    },
                    then_block: vec![Statement::Assignment {
                        targets: Vec::new(),
                        value: Expression::Identifier {
                            name: "one".to_string(),
                            position: SourcePosition::new(3, 0, 0),
                        },
                        position: SourcePosition::new(3, 0, 0),
                    }],
                    elif_blocks: vec![(
                        Expression::Identifier {
                            name: "cond2".to_string(),
                            position: SourcePosition::new(4, 0, 0),
                        },
                        vec![Statement::Assignment {
                            targets: Vec::new(),
                            value: Expression::Identifier {
                                name: "two".to_string(),
                                position: SourcePosition::new(5, 0, 0),
                            },
                            position: SourcePosition::new(5, 0, 0),
                        }],
                    )],
                    else_block: Some(vec![Statement::Assignment {
                        targets: Vec::new(),
                        value: Expression::Identifier {
                            name: "three".to_string(),
                            position: SourcePosition::new(7, 0, 0),
                        },
                        position: SourcePosition::new(7, 0, 0),
                    }]),
                    position: SourcePosition::new(2, 0, 0),
                },
                Statement::Assignment {
                    targets: Vec::new(),
                    value: Expression::Identifier {
                        name: "four".to_string(),
                        position: SourcePosition::new(8, 0, 0),
                    },
                    position: SourcePosition::new(8, 0, 0),
                },
            ],
            decorators: Vec::new(),
            is_async: false,
            return_type: None,
            position: SourcePosition::new(1, 0, 0),
        };
        
        let cfg = CFGBuilder::build_function_cfg(&func).unwrap();
        
        // Should have: entry, normal(if cond), then, elif_cond, elif_then, else, merge(y=4), exit
        assert_eq!(cfg.block_count(), 8);
        
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        
        // First condition block
        let if_cond_id = entry_block.successors[0];
        let if_cond = cfg.get_block(if_cond_id).unwrap();
        assert_eq!(if_cond.successors.len(), 2); // then and elif
        
        // Then block
        let then_id = if_cond.successors[0];
        let then_block = cfg.get_block(then_id).unwrap();
        assert_eq!(then_block.statements.len(), 1); // x=1
        
        // Elif condition block (false branch from if)
        let elif_cond_id = if_cond.successors[1];
        let elif_cond = cfg.get_block(elif_cond_id).unwrap();
        assert_eq!(elif_cond.kind, BlockKind::Conditional);
        assert_eq!(elif_cond.statements.len(), 1); // elif condition
        assert_eq!(elif_cond.successors.len(), 2); // elif then and else
        
        // Elif then block
        let elif_then_id = elif_cond.successors[0];
        let elif_then = cfg.get_block(elif_then_id).unwrap();
        assert_eq!(elif_then.statements.len(), 1); // x=2
        
        // Else block
        let else_id = elif_cond.successors[1];
        let else_block = cfg.get_block(else_id).unwrap();
        assert_eq!(else_block.statements.len(), 1); // x=3
        
        // All should connect to merge
        let merge_id = then_block.successors[0];
        assert_eq!(elif_then.successors[0], merge_id);
        assert_eq!(else_block.successors[0], merge_id);
        
        let merge_block = cfg.get_block(merge_id).unwrap();
        assert_eq!(merge_block.statements.len(), 1); // y=4
        assert_eq!(merge_block.predecessors.len(), 3); // then, elif_then, else
    }
    
    #[test]
    fn test_nested_if() {
        // def foo():
        //     if outer:
        //         if inner:
        //             x = 1
        //         y = 2
        //     z = 3
        let func = Statement::FunctionDef {
            name: "foo".to_string(),
            parameters: Vec::new(),
            body: vec![
                Statement::If {
                    condition: Expression::Identifier {
                        name: "outer".to_string(),
                        position: SourcePosition::new(2, 0, 0),
                    },
                    then_block: vec![
                        Statement::If {
                            condition: Expression::Identifier {
                                name: "inner".to_string(),
                                position: SourcePosition::new(3, 0, 0),
                            },
                            then_block: vec![Statement::Assignment {
                                targets: Vec::new(),
                                value: Expression::Identifier {
                                    name: "one".to_string(),
                                    position: SourcePosition::new(4, 0, 0),
                                },
                                position: SourcePosition::new(4, 0, 0),
                            }],
                            elif_blocks: Vec::new(),
                            else_block: None,
                            position: SourcePosition::new(3, 0, 0),
                        },
                        Statement::Assignment {
                            targets: Vec::new(),
                            value: Expression::Identifier {
                                name: "two".to_string(),
                                position: SourcePosition::new(5, 0, 0),
                            },
                            position: SourcePosition::new(5, 0, 0),
                        },
                    ],
                    elif_blocks: Vec::new(),
                    else_block: None,
                    position: SourcePosition::new(2, 0, 0),
                },
                Statement::Assignment {
                    targets: Vec::new(),
                    value: Expression::Identifier {
                        name: "three".to_string(),
                        position: SourcePosition::new(6, 0, 0),
                    },
                    position: SourcePosition::new(6, 0, 0),
                },
            ],
            decorators: Vec::new(),
            is_async: false,
            return_type: None,
            position: SourcePosition::new(1, 0, 0),
        };
        
        let cfg = CFGBuilder::build_function_cfg(&func).unwrap();
        
        // Complex structure with nested ifs
        // Entry -> outer_cond -> outer_then(inner_cond) -> inner_then -> inner_merge(y=2) -> outer_merge(z=3) -> exit
        assert!(cfg.block_count() >= 7); // At least: entry, outer_cond, outer_then, inner_then, inner_merge, outer_merge, exit
        
        // Verify structure
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        assert_eq!(entry_block.successors.len(), 1);
    }
    
    #[test]
    fn test_if_with_return_in_then() {
        // def foo():
        //     if True:
        //         return 1
        //     x = 2
        let func = Statement::FunctionDef {
            name: "foo".to_string(),
            parameters: Vec::new(),
            body: vec![
                Statement::If {
                    condition: Expression::Identifier {
                        name: "True".to_string(),
                        position: SourcePosition::new(2, 0, 0),
                    },
                    then_block: vec![Statement::Return {
                        value: None,
                        position: SourcePosition::new(3, 0, 0),
                    }],
                    elif_blocks: Vec::new(),
                    else_block: None,
                    position: SourcePosition::new(2, 0, 0),
                },
                Statement::Assignment {
                    targets: Vec::new(),
                    value: Expression::Identifier {
                        name: "two".to_string(),
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
        
        // Then block should connect to exit (return)
        // Merge should still be created and have x=2
        // False branch goes to merge
        assert!(cfg.block_count() >= 5);
        
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        let condition_id = entry_block.successors[0];
        let condition_block = cfg.get_block(condition_id).unwrap();
        
        // Then block
        let then_id = condition_block.successors[0];
        let then_block = cfg.get_block(then_id).unwrap();
        assert!(matches!(then_block.statements.last().unwrap(), Statement::Return { .. }));
        
        // Then connects to exit (via return)
        let exit_blocks = cfg.exits();
        assert!(then_block.successors.iter().any(|&s| exit_blocks.contains(&s)));
    }
    
    #[test]
    fn test_if_else_both_return() {
        // def foo():
        //     if True:
        //         return 1
        //     else:
        //         return 2
        //     x = 3  # unreachable
        let func = Statement::FunctionDef {
            name: "foo".to_string(),
            parameters: Vec::new(),
            body: vec![
                Statement::If {
                    condition: Expression::Identifier {
                        name: "True".to_string(),
                        position: SourcePosition::new(2, 0, 0),
                    },
                    then_block: vec![Statement::Return {
                        value: None,
                        position: SourcePosition::new(3, 0, 0),
                    }],
                    elif_blocks: Vec::new(),
                    else_block: Some(vec![Statement::Return {
                        value: None,
                        position: SourcePosition::new(5, 0, 0),
                    }]),
                    position: SourcePosition::new(2, 0, 0),
                },
                Statement::Assignment {
                    targets: Vec::new(),
                    value: Expression::Identifier {
                        name: "three".to_string(),
                        position: SourcePosition::new(6, 0, 0),
                    },
                    position: SourcePosition::new(6, 0, 0),
                },
            ],
            decorators: Vec::new(),
            is_async: false,
            return_type: None,
            position: SourcePosition::new(1, 0, 0),
        };
        
        let cfg = CFGBuilder::build_function_cfg(&func).unwrap();
        
        // Both branches return, so merge block should have no predecessors (unreachable)
        // x=3 should be in an unreachable block
        assert!(cfg.block_count() >= 6);
        
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        let condition_id = entry_block.successors[0];
        let condition_block = cfg.get_block(condition_id).unwrap();
        
        let then_id = condition_block.successors[0];
        let then_block = cfg.get_block(then_id).unwrap();
        assert!(matches!(then_block.statements.last().unwrap(), Statement::Return { .. }));
        
        let else_id = condition_block.successors[1];
        let else_block = cfg.get_block(else_id).unwrap();
        assert!(matches!(else_block.statements.last().unwrap(), Statement::Return { .. }));
        
        // Both should connect to exit
        let exit_blocks = cfg.exits();
        assert!(then_block.successors.iter().any(|&s| exit_blocks.contains(&s)));
        assert!(else_block.successors.iter().any(|&s| exit_blocks.contains(&s)));
        
        // Find the unreachable block with x=3
        let unreachable_blocks: Vec<_> = cfg.block_ids()
            .into_iter()
            .filter(|&id| {
                let block = cfg.get_block(id).unwrap();
                !block.has_predecessors() && !exit_blocks.contains(&id) && id != entry
            })
            .collect();
        
        assert!(unreachable_blocks.len() >= 1);
    }
    
    // Helper functions for creating test statements
    fn create_assignment_stmt(line: usize) -> Statement {
        Statement::Assignment {
            targets: Vec::new(),
            value: Expression::Identifier {
                name: "dummy".to_string(),
                position: SourcePosition::new(line, 0, 0),
            },
            position: SourcePosition::new(line, 0, 0),
        }
    }
    
    fn create_bool_expression(value: bool, line: usize) -> Expression {
        use crate::ast::Literal;
        Expression::Literal(Literal::Boolean {
            value,
            position: SourcePosition::new(line, 0, 0),
        })
    }
    
    fn create_function(body: Vec<Statement>) -> Statement {
        Statement::FunctionDef {
            name: "test_func".to_string(),
            parameters: Vec::new(),
            body,
            decorators: Vec::new(),
            is_async: false,
            return_type: None,
            position: SourcePosition::new(1, 0, 0),
        }
    }
    
    #[test]
    fn test_simple_while_loop() {
        // def foo():
        //     x = 1
        //     while True:
        //         y = 2
        //     z = 3
        let function = create_function(vec![
            create_assignment_stmt(2),
            Statement::While {
                condition: create_bool_expression(true, 3),
                body: vec![
                    create_assignment_stmt(4),
                ],
                else_block: None,
                position: SourcePosition::new(3, 0, 0),
            },
            create_assignment_stmt(5),
        ]);
        
        let cfg = CFGBuilder::build_function_cfg(&function).unwrap();
        
        // Expected structure:
        // entry → initial_block(x=1) → header(condition) → body(y=2) → header (back-edge)
        //                                      ↓ false
        //                                   exit_block(z=3) → exit
        
        assert!(cfg.block_count() >= 6);
        
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        let initial_id = entry_block.successors[0];
        let initial_block = cfg.get_block(initial_id).unwrap();
        
        // Find assignment x=1
        assert!(matches!(initial_block.statements.first().unwrap(), Statement::Assignment { .. }));
        
        // Initial block should connect to loop header
        assert_eq!(initial_block.successors.len(), 1);
        let header_id = initial_block.successors[0];
        let header_block = cfg.get_block(header_id).unwrap();
        assert_eq!(header_block.kind, BlockKind::LoopHeader);
        
        // Header should have condition
        assert!(matches!(header_block.statements.first().unwrap(), Statement::Expression(_)));
        
        // Header should have 2 successors: body (true) and exit (false)
        assert_eq!(header_block.successors.len(), 2);
        let body_id = header_block.successors[0];
        let exit_block_id = header_block.successors[1];
        
        let body_block = cfg.get_block(body_id).unwrap();
        assert_eq!(body_block.kind, BlockKind::LoopBody);
        
        // Body should have y=2
        assert!(matches!(body_block.statements.first().unwrap(), Statement::Assignment { .. }));
        
        // Body should have back-edge to header
        assert_eq!(body_block.successors.len(), 1);
        assert_eq!(body_block.successors[0], header_id);
        
        // Exit block should have z=3
        let exit_block = cfg.get_block(exit_block_id).unwrap();
        assert!(matches!(exit_block.statements.first().unwrap(), Statement::Assignment { .. }));
    }
    
    #[test]
    fn test_while_with_break() {
        // def foo():
        //     while True:
        //         x = 1
        //         break
        //         y = 2  # unreachable
        //     z = 3
        let function = create_function(vec![
            Statement::While {
                condition: create_bool_expression(true, 2),
                body: vec![
                    create_assignment_stmt(3),
                    Statement::Break(SourcePosition::new(4, 0, 0)),
                    create_assignment_stmt(5),
                ],
                else_block: None,
                position: SourcePosition::new(2, 0, 0),
            },
            create_assignment_stmt(6),
        ]);
        
        let cfg = CFGBuilder::build_function_cfg(&function).unwrap();
        
        // Break should create edge to loop exit
        // y=2 should be in unreachable block
        
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        let initial_id = entry_block.successors[0];
        let initial_block = cfg.get_block(initial_id).unwrap();
        
        let header_id = initial_block.successors[0];
        let header_block = cfg.get_block(header_id).unwrap();
        
        let body_id = header_block.successors[0];
        let body_block = cfg.get_block(body_id).unwrap();
        
        // Body should have x=1 and break
        assert!(matches!(body_block.statements[0], Statement::Assignment { .. }));
        assert!(matches!(body_block.statements[1], Statement::Break(_)));
        
        // Body should connect to exit (via break)
        assert!(body_block.successors.len() >= 1);
        
        // Find unreachable block with y=2
        let unreachable_blocks: Vec<_> = cfg.block_ids()
            .into_iter()
            .filter(|&id| {
                let block = cfg.get_block(id).unwrap();
                !block.has_predecessors() && block.kind == BlockKind::Normal
            })
            .collect();
        
        assert!(unreachable_blocks.len() >= 1);
    }
    
    #[test]
    fn test_while_with_continue() {
        // def foo():
        //     while True:
        //         x = 1
        //         continue
        //         y = 2  # unreachable
        let function = create_function(vec![
            Statement::While {
                condition: create_bool_expression(true, 2),
                body: vec![
                    create_assignment_stmt(3),
                    Statement::Continue(SourcePosition::new(4, 0, 0)),
                    create_assignment_stmt(5),
                ],
                else_block: None,
                position: SourcePosition::new(2, 0, 0),
            },
        ]);
        
        let cfg = CFGBuilder::build_function_cfg(&function).unwrap();
        
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        let initial_id = entry_block.successors[0];
        let initial_block = cfg.get_block(initial_id).unwrap();
        
        let header_id = initial_block.successors[0];
        let header_block = cfg.get_block(header_id).unwrap();
        
        let body_id = header_block.successors[0];
        let body_block = cfg.get_block(body_id).unwrap();
        
        // Body should have x=1 and continue
        assert!(matches!(body_block.statements[0], Statement::Assignment { .. }));
        assert!(matches!(body_block.statements[1], Statement::Continue(_)));
        
        // Body should connect back to header (via continue)
        assert!(body_block.successors.contains(&header_id));
        
        // Find unreachable block with y=2
        let unreachable_blocks: Vec<_> = cfg.block_ids()
            .into_iter()
            .filter(|&id| {
                let block = cfg.get_block(id).unwrap();
                !block.has_predecessors() && block.kind == BlockKind::Normal
            })
            .collect();
        
        assert!(unreachable_blocks.len() >= 1);
    }
    
    #[test]
    fn test_while_else_no_break() {
        // def foo():
        //     while False:
        //         x = 1
        //     else:
        //         y = 2
        //     z = 3
        let function = create_function(vec![
            Statement::While {
                condition: create_bool_expression(false, 2),
                body: vec![
                    create_assignment_stmt(3),
                ],
                else_block: Some(vec![
                    create_assignment_stmt(5),
                ]),
                position: SourcePosition::new(2, 0, 0),
            },
            create_assignment_stmt(6),
        ]);
        
        let cfg = CFGBuilder::build_function_cfg(&function).unwrap();
        
        // Header false branch should go to else, which then goes to exit
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        let initial_id = entry_block.successors[0];
        let initial_block = cfg.get_block(initial_id).unwrap();
        
        let header_id = initial_block.successors[0];
        let header_block = cfg.get_block(header_id).unwrap();
        
        // Header should have 2 successors: body and else
        assert_eq!(header_block.successors.len(), 2);
        
        let _body_id = header_block.successors[0];
        let else_id = header_block.successors[1];
        
        let else_block = cfg.get_block(else_id).unwrap();
        // Else block should have y=2
        assert!(matches!(else_block.statements.first().unwrap(), Statement::Assignment { .. }));
        
        // Else should connect to exit with z=3
        assert!(else_block.successors.len() >= 1);
    }
    
    #[test]
    fn test_while_else_with_break() {
        // def foo():
        //     while True:
        //         break
        //     else:
        //         y = 2  # should not execute
        //     z = 3
        let function = create_function(vec![
            Statement::While {
                condition: create_bool_expression(true, 2),
                body: vec![
                    Statement::Break(SourcePosition::new(3, 0, 0)),
                ],
                else_block: Some(vec![
                    create_assignment_stmt(5),
                ]),
                position: SourcePosition::new(2, 0, 0),
            },
            create_assignment_stmt(6),
        ]);
        
        let cfg = CFGBuilder::build_function_cfg(&function).unwrap();
        
        // Break should go to exit, NOT to else
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        let initial_id = entry_block.successors[0];
        let initial_block = cfg.get_block(initial_id).unwrap();
        
        let header_id = initial_block.successors[0];
        let header_block = cfg.get_block(header_id).unwrap();
        
        let body_id = header_block.successors[0];
        let body_block = cfg.get_block(body_id).unwrap();
        
        // Body has break
        assert!(matches!(body_block.statements.first().unwrap(), Statement::Break(_)));
        
        // Find the exit block with z=3
        let exit_blocks: Vec<_> = cfg.block_ids()
            .into_iter()
            .filter(|&id| {
                let block = cfg.get_block(id).unwrap();
                block.kind == BlockKind::Normal && 
                block.statements.iter().any(|s| matches!(s, Statement::Assignment { .. }))
            })
            .collect();
        
        // Break should connect to one of these exits
        assert!(exit_blocks.len() >= 1);
    }
    
    #[test]
    fn test_nested_while_loops() {
        // def foo():
        //     while True:
        //         x = 1
        //         while True:
        //             y = 2
        //             break
        //         z = 3
        let function = create_function(vec![
            Statement::While {
                condition: create_bool_expression(true, 2),
                body: vec![
                    create_assignment_stmt(3),
                    Statement::While {
                        condition: create_bool_expression(true, 4),
                        body: vec![
                            create_assignment_stmt(5),
                            Statement::Break(SourcePosition::new(6, 0, 0)),
                        ],
                        else_block: None,
                        position: SourcePosition::new(4, 0, 0),
                    },
                    create_assignment_stmt(7),
                ],
                else_block: None,
                position: SourcePosition::new(2, 0, 0),
            },
        ]);
        
        let cfg = CFGBuilder::build_function_cfg(&function).unwrap();
        
        // Should have multiple loop headers
        let loop_headers: Vec<_> = cfg.block_ids()
            .into_iter()
            .filter(|&id| {
                let block = cfg.get_block(id).unwrap();
                block.kind == BlockKind::LoopHeader
            })
            .collect();
        
        assert_eq!(loop_headers.len(), 2); // outer and inner loop
        
        // Should have multiple loop bodies
        let loop_bodies: Vec<_> = cfg.block_ids()
            .into_iter()
            .filter(|&id| {
                let block = cfg.get_block(id).unwrap();
                block.kind == BlockKind::LoopBody
            })
            .collect();
        
        assert_eq!(loop_bodies.len(), 2); // outer and inner loop
    }
    
    #[test]
    fn test_simple_for_loop() {
        // def foo():
        //     x = 1
        //     for i in range(10):
        //         y = 2
        //     z = 3
        let function = create_function(vec![
            create_assignment_stmt(2),
            Statement::For {
                target: Expression::Identifier {
                    name: "i".to_string(),
                    position: SourcePosition::new(3, 0, 0),
                },
                iter: Expression::Identifier {
                    name: "range".to_string(),
                    position: SourcePosition::new(3, 0, 0),
                },
                body: vec![
                    create_assignment_stmt(4),
                ],
                else_block: None,
                position: SourcePosition::new(3, 0, 0),
            },
            create_assignment_stmt(5),
        ]);
        
        let cfg = CFGBuilder::build_function_cfg(&function).unwrap();
        
        // Expected structure similar to while:
        // entry → initial_block(x=1, iter_eval) → header(assign i) → body(y=2) → header (back-edge)
        //                                                    ↓ no more items
        //                                                 exit_block(z=3) → exit
        
        assert!(cfg.block_count() >= 6);
        
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        let initial_id = entry_block.successors[0];
        let initial_block = cfg.get_block(initial_id).unwrap();
        
        // Initial block should have x=1 and iterator evaluation
        assert!(initial_block.statements.len() >= 1);
        
        // Initial block should connect to loop header
        assert_eq!(initial_block.successors.len(), 1);
        let header_id = initial_block.successors[0];
        let header_block = cfg.get_block(header_id).unwrap();
        assert_eq!(header_block.kind, BlockKind::LoopHeader);
        
        // Header should have target assignment
        assert!(matches!(header_block.statements.first().unwrap(), Statement::Assignment { .. }));
        
        // Header should have 2 successors: body (has items) and exit (no items)
        assert_eq!(header_block.successors.len(), 2);
        let body_id = header_block.successors[0];
        let exit_block_id = header_block.successors[1];
        
        let body_block = cfg.get_block(body_id).unwrap();
        assert_eq!(body_block.kind, BlockKind::LoopBody);
        
        // Body should have y=2
        assert!(matches!(body_block.statements.first().unwrap(), Statement::Assignment { .. }));
        
        // Body should have back-edge to header
        assert_eq!(body_block.successors.len(), 1);
        assert_eq!(body_block.successors[0], header_id);
        
        // Exit block should have z=3
        let exit_block = cfg.get_block(exit_block_id).unwrap();
        assert!(matches!(exit_block.statements.first().unwrap(), Statement::Assignment { .. }));
    }
    
    #[test]
    fn test_for_with_break() {
        // def foo():
        //     for i in range(10):
        //         x = 1
        //         break
        //         y = 2  # unreachable
        //     z = 3
        let function = create_function(vec![
            Statement::For {
                target: Expression::Identifier {
                    name: "i".to_string(),
                    position: SourcePosition::new(2, 0, 0),
                },
                iter: Expression::Identifier {
                    name: "range".to_string(),
                    position: SourcePosition::new(2, 0, 0),
                },
                body: vec![
                    create_assignment_stmt(3),
                    Statement::Break(SourcePosition::new(4, 0, 0)),
                    create_assignment_stmt(5),
                ],
                else_block: None,
                position: SourcePosition::new(2, 0, 0),
            },
            create_assignment_stmt(6),
        ]);
        
        let cfg = CFGBuilder::build_function_cfg(&function).unwrap();
        
        // Break should create edge to loop exit
        // y=2 should be in unreachable block
        
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        let initial_id = entry_block.successors[0];
        let initial_block = cfg.get_block(initial_id).unwrap();
        
        let header_id = initial_block.successors[0];
        let header_block = cfg.get_block(header_id).unwrap();
        
        let body_id = header_block.successors[0];
        let body_block = cfg.get_block(body_id).unwrap();
        
        // Body should have x=1 and break
        assert!(matches!(body_block.statements[0], Statement::Assignment { .. }));
        assert!(matches!(body_block.statements[1], Statement::Break(_)));
        
        // Body should connect to exit (via break)
        assert!(body_block.successors.len() >= 1);
        
        // Find unreachable block with y=2
        let unreachable_blocks: Vec<_> = cfg.block_ids()
            .into_iter()
            .filter(|&id| {
                let block = cfg.get_block(id).unwrap();
                !block.has_predecessors() && block.kind == BlockKind::Normal
            })
            .collect();
        
        assert!(unreachable_blocks.len() >= 1);
    }
    
    #[test]
    fn test_for_with_continue() {
        // def foo():
        //     for i in range(10):
        //         x = 1
        //         continue
        //         y = 2  # unreachable
        let function = create_function(vec![
            Statement::For {
                target: Expression::Identifier {
                    name: "i".to_string(),
                    position: SourcePosition::new(2, 0, 0),
                },
                iter: Expression::Identifier {
                    name: "range".to_string(),
                    position: SourcePosition::new(2, 0, 0),
                },
                body: vec![
                    create_assignment_stmt(3),
                    Statement::Continue(SourcePosition::new(4, 0, 0)),
                    create_assignment_stmt(5),
                ],
                else_block: None,
                position: SourcePosition::new(2, 0, 0),
            },
        ]);
        
        let cfg = CFGBuilder::build_function_cfg(&function).unwrap();
        
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        let initial_id = entry_block.successors[0];
        let initial_block = cfg.get_block(initial_id).unwrap();
        
        let header_id = initial_block.successors[0];
        let header_block = cfg.get_block(header_id).unwrap();
        
        let body_id = header_block.successors[0];
        let body_block = cfg.get_block(body_id).unwrap();
        
        // Body should have x=1 and continue
        assert!(matches!(body_block.statements[0], Statement::Assignment { .. }));
        assert!(matches!(body_block.statements[1], Statement::Continue(_)));
        
        // Body should connect back to header (via continue)
        assert!(body_block.successors.contains(&header_id));
        
        // Find unreachable block with y=2
        let unreachable_blocks: Vec<_> = cfg.block_ids()
            .into_iter()
            .filter(|&id| {
                let block = cfg.get_block(id).unwrap();
                !block.has_predecessors() && block.kind == BlockKind::Normal
            })
            .collect();
        
        assert!(unreachable_blocks.len() >= 1);
    }
    
    #[test]
    fn test_for_else_no_break() {
        // def foo():
        //     for i in range(3):
        //         x = 1
        //     else:
        //         y = 2
        //     z = 3
        let function = create_function(vec![
            Statement::For {
                target: Expression::Identifier {
                    name: "i".to_string(),
                    position: SourcePosition::new(2, 0, 0),
                },
                iter: Expression::Identifier {
                    name: "range".to_string(),
                    position: SourcePosition::new(2, 0, 0),
                },
                body: vec![
                    create_assignment_stmt(3),
                ],
                else_block: Some(vec![
                    create_assignment_stmt(5),
                ]),
                position: SourcePosition::new(2, 0, 0),
            },
            create_assignment_stmt(6),
        ]);
        
        let cfg = CFGBuilder::build_function_cfg(&function).unwrap();
        
        // Header no-more-items branch should go to else, which then goes to exit
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        let initial_id = entry_block.successors[0];
        let initial_block = cfg.get_block(initial_id).unwrap();
        
        let header_id = initial_block.successors[0];
        let header_block = cfg.get_block(header_id).unwrap();
        
        // Header should have 2 successors: body and else
        assert_eq!(header_block.successors.len(), 2);
        
        let _body_id = header_block.successors[0];
        let else_id = header_block.successors[1];
        
        let else_block = cfg.get_block(else_id).unwrap();
        // Else block should have y=2
        assert!(matches!(else_block.statements.first().unwrap(), Statement::Assignment { .. }));
        
        // Else should connect to exit with z=3
        assert!(else_block.successors.len() >= 1);
    }
    
    #[test]
    fn test_for_else_with_break() {
        // def foo():
        //     for i in range(10):
        //         break
        //     else:
        //         y = 2  # should not execute
        //     z = 3
        let function = create_function(vec![
            Statement::For {
                target: Expression::Identifier {
                    name: "i".to_string(),
                    position: SourcePosition::new(2, 0, 0),
                },
                iter: Expression::Identifier {
                    name: "range".to_string(),
                    position: SourcePosition::new(2, 0, 0),
                },
                body: vec![
                    Statement::Break(SourcePosition::new(3, 0, 0)),
                ],
                else_block: Some(vec![
                    create_assignment_stmt(5),
                ]),
                position: SourcePosition::new(2, 0, 0),
            },
            create_assignment_stmt(6),
        ]);
        
        let cfg = CFGBuilder::build_function_cfg(&function).unwrap();
        
        // Break should go to exit, NOT to else
        let entry = cfg.entry();
        let entry_block = cfg.get_block(entry).unwrap();
        let initial_id = entry_block.successors[0];
        let initial_block = cfg.get_block(initial_id).unwrap();
        
        let header_id = initial_block.successors[0];
        let header_block = cfg.get_block(header_id).unwrap();
        
        let body_id = header_block.successors[0];
        let body_block = cfg.get_block(body_id).unwrap();
        
        // Body has break
        assert!(matches!(body_block.statements.first().unwrap(), Statement::Break(_)));
        
        // Find the exit block with z=3
        let exit_blocks: Vec<_> = cfg.block_ids()
            .into_iter()
            .filter(|&id| {
                let block = cfg.get_block(id).unwrap();
                block.kind == BlockKind::Normal && 
                block.statements.iter().any(|s| matches!(s, Statement::Assignment { .. }))
            })
            .collect();
        
        // Break should connect to one of these exits
        assert!(exit_blocks.len() >= 1);
    }
    
    #[test]
    fn test_nested_for_in_while() {
        // def foo():
        //     while True:
        //         x = 1
        //         for i in range(5):
        //             y = 2
        //             break
        //         z = 3
        let function = create_function(vec![
            Statement::While {
                condition: create_bool_expression(true, 2),
                body: vec![
                    create_assignment_stmt(3),
                    Statement::For {
                        target: Expression::Identifier {
                            name: "i".to_string(),
                            position: SourcePosition::new(4, 0, 0),
                        },
                        iter: Expression::Identifier {
                            name: "range".to_string(),
                            position: SourcePosition::new(4, 0, 0),
                        },
                        body: vec![
                            create_assignment_stmt(5),
                            Statement::Break(SourcePosition::new(6, 0, 0)),
                        ],
                        else_block: None,
                        position: SourcePosition::new(4, 0, 0),
                    },
                    create_assignment_stmt(7),
                ],
                else_block: None,
                position: SourcePosition::new(2, 0, 0),
            },
        ]);
        
        let cfg = CFGBuilder::build_function_cfg(&function).unwrap();
        
        // Should have multiple loop headers (while and for)
        let loop_headers: Vec<_> = cfg.block_ids()
            .into_iter()
            .filter(|&id| {
                let block = cfg.get_block(id).unwrap();
                block.kind == BlockKind::LoopHeader
            })
            .collect();
        
        assert_eq!(loop_headers.len(), 2); // while and for loop
        
        // Should have multiple loop bodies
        let loop_bodies: Vec<_> = cfg.block_ids()
            .into_iter()
            .filter(|&id| {
                let block = cfg.get_block(id).unwrap();
                block.kind == BlockKind::LoopBody
            })
            .collect();
        
        assert_eq!(loop_bodies.len(), 2); // while and for loop
    }
}
