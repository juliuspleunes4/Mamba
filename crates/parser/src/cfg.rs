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
}
