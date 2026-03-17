# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),  
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Phase 5.1: Built-in Functions (Tranche 1)**
  - Added builtin lowering module: `crates/transpiler/src/builtins.rs`
  - Wired builtin call transpilation into expression call handling
  - Implemented built-ins in this tranche:
    * `print`, `len`, `range`, `type`
    * `str`, `int`, `float`, `bool`, `abs`
    * `min`, `max`, `sum`
    * `reversed`, `enumerate`, `zip`, `map`, `filter`, `all`, `any`
  - Added builtin-focused unit tests in:
    * `crates/transpiler/src/builtins.rs`
    * `crates/transpiler/src/expression.rs`
  - Updated existing statement/module expectations where `print(...)` now lowers
    to Rust `println!` output formatting
  - Validation:
    * `cargo test -p mamba-transpiler` passing (unit + integration suites)

- **Phase 4.9: Transpiler Testing** ✅
  - Added dedicated integration suite: `crates/transpiler/tests/phase_4_9_integration.rs`
  - Added coverage for all 4.9 goals:
    * Unit-surface smoke coverage across `CodeGenerator`, `ExpressionTranspiler`,
      `StatementTranspiler`, and `ModuleTranspiler`
    * Complex nested module transpilation path validation
    * Generated Rust compilation checks via `rustc`
    * Runtime behavior verification by compiling generated code as test binaries
      and asserting expected function behavior (`assert_eq!`)
  - Added portable test helpers for temporary workspace creation, code emission,
    `rustc` invocation, and binary execution
  - `cargo test -p mamba-transpiler` now runs both unit and integration tests,
    including compile-and-run validation of generated Rust artifacts

- **Phase 4.8: Main Function Generation** ✅
  - Added `module` transpiler module with `ModuleTranspiler`
  - Implemented full-module lowering pipeline to generate a valid Rust entry point
  - Top-level handling behavior:
    * Function definitions are emitted at module scope
    * Non-function top-level statements are wrapped in `fn main()`
    * Empty modules still emit a valid `fn main() {}` entry point
  - Added top-level variable handling:
    * First simple identifier assignment at module top-level lowers to `let mut`
    * Subsequent assignments to the same identifier lower as regular reassignments
    * Annotated top-level assignments remain declaration-oriented and are tracked
  - Added 4 dedicated module transpilation tests covering:
    * main-wrapper generation
    * top-level function hoisting
    * top-level variable declaration/reassignment behavior
    * empty-module entry point generation

- **Phase 4.7: Advanced Transpilation** ✅
  - Extended `ExpressionTranspiler` with:
    * lambda expression lowering to Rust closures (`|args| expr`)
    * list-comprehension lowering to iterator map/filter pipelines collecting to `Vec<_>`
  - Added comprehension validation for this phase scope:
    * supports single-generator list comprehensions
    * emits explicit error for multi-generator comprehensions (deferred)
  - Extended `StatementTranspiler` function lowering with:
    * basic decorator handling (emits decorator metadata comments above function)
    * recursive call preservation for non-tail recursive forms
    * direct self tail-recursion optimization for eligible
      `return self(...args...)` function bodies by rewriting to explicit loops
  - Added 7 dedicated advanced transpilation tests covering:
    * lambda and closure lowering
    * list-comprehension map/filter output
    * unsupported multi-generator comprehension behavior
    * decorator emission
    * recursive call output
    * tail recursion optimization output

- **Phase 4.6: Function Transpilation** ✅
  - Extended `StatementTranspiler` to support `FunctionDef` lowering
  - Implemented function signature generation for:
    * sync and async functions (`fn` / `async fn`)
    * annotated return types (defaulting to `()` when absent)
    * positional/regular/keyword-only parameters
    * varargs (`Vec<T>`) and kwargs (`HashMap<String, T>`) parameters
  - Implemented default-parameter lowering by using `Option<T>` in signatures
    and function-entry initialization (`name.unwrap_or(default)`)
  - Added parameter base-type resolution from annotations with fallback literal
    inference for default values
  - Function body transpilation now reuses recursive statement/block lowering,
    preserving nested control-flow semantics and multiple return statements
  - Added 5 dedicated function transpilation unit tests covering signatures,
    defaults, async functions, varargs/kwargs, and multi-return bodies

- **Phase 4.5: Control Flow Transpilation** ✅
  - Extended `StatementTranspiler` with control-flow lowering
  - Implemented transpilation for:
    * `if` statements
    * `if-else`
    * `if-elif-else`
    * `while` loops
    * `for` loops using iterator pattern (`for target in iter`)
    * Nested control flow (recursive statement/block lowering)
  - Added loop-`else` support for `while` and `for` by emitting scoped
    break-tracking flags and post-loop guard blocks (`if !loop_broke`)
  - Updated `break` lowering to set loop break flags when loop-`else` semantics
    are active, preserving expected Python behavior
  - Added `transpile_statements()` helper for block-level transpilation
  - Added 8 dedicated control-flow unit tests covering all 4.5 checklist items

- **Phase 4.4: Statement Transpilation** ✅
  - Added `statement` module in transpiler crate
  - Introduced `StatementTranspiler` for core statement lowering
  - Implemented statement support for:
    * Variable declarations (`let mut`) via annotation and initializer helpers
    * Assignments (single target)
    * Augmented assignments (`+=`, `-=`, `*=`, `/=`, `//=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`)
    * Power augmented assignment lowered to explicit `target = target.pow(value)` form
    * Expression statements
    * Return statements (`return;` and `return value;`)
    * Pass statements (no-op output)
    * Break/continue statements
  - Added explicit error handling for:
    * Empty assignment target lists
    * Multi-target assignment (deferred to future phase)
    * Unsupported statement kinds for phase scope
  - Added 9 dedicated unit tests for statement transpilation paths and failures

- **Phase 4.3: Expression Transpilation** ✅
  - Added `expression` module in transpiler crate
  - Introduced `ExpressionTranspiler` with recursive expression lowering
  - Implemented transpilation coverage for:
    * Literals (`int`, `float`, `str`, `bool`, `None`, `Ellipsis`)
    * Identifiers
    * Binary/unary expressions
    * Comparisons and logical operators
    * Function calls
    * Parenthesized expressions
    * Tuples
    * Lists to `vec![...]`
    * Dicts to `std::collections::HashMap::from([...])`
    * Subscript operations
    * Attribute access
  - Added mapping for membership operators:
    * `a in b` -> `b.contains(&a)`
    * `a not in b` -> `!b.contains(&a)`
  - Added explicit unsupported-expression error handling for non-implemented
    expression kinds
  - Added 14 focused unit tests for expression output and failure paths

- **Phase 4.2: Basic Type Mapping** ✅
  - Added `type_mapping` module in transpiler crate
  - Introduced `TypeMapper` with configurable integer width strategy:
    * `IntWidth::I32` for `int -> i32`
    * `IntWidth::I64` for `int -> i64` (default)
  - Added `RustType` representation and rendering helpers for Rust output:
    * `i32`, `i64`, `f64`, `String`, `bool`, `()`, `Option<T>`, unknown (`_`)
  - Implemented semantic type mapping:
    * `int -> i32/i64`
    * `float -> f64`
    * `str -> String`
    * `bool -> bool`
    * `None -> Option<()>` (context-free baseline for `Option<T>`)
  - Added type annotation helpers:
    * Maps annotation expressions (`int`, `float`, `str`, `bool`, `None`)
    * Supports parenthesized annotations
    * Supports `Option[T]` generic form (including nested options)
    * Renders mapped annotations as Rust type strings
  - Added robust error handling for unknown/unsupported annotation shapes
  - Added 11 dedicated unit tests for happy paths, nested generics, and failures

- **Phase 4.1: Code Generation Infrastructure** ✅
  - Added `CodeGenerator` core infrastructure in transpiler crate
  - Implemented output buffer management and indentation tracking
  - Added helper APIs:
    * `emit()` / `emit_line()` / `emit_empty_line()`
    * `indent()` / `dedent()`
    * `open_block()` / `close_block()`
    * `clear()` / `as_str()` / `into_string()`
  - Added a template system for common patterns with:
    * template registration
    * placeholder rendering via `{{key}}`
    * template emission to output
    * default templates for function signatures and let bindings
  - Added focused transpiler unit tests (8 total) for formatting behavior,
    template rendering, and error paths

- **Phase 3.6: Semantic Testing** ✅
  - Completed semantic testing closeout audit and checklist alignment
  - Verified broad semantic coverage for:
    * Scope resolution
    * Type inference accuracy
    * Semantic error detection
    * Edge cases (unreachable, control flow, assignment targets, operators)
  - Confirmed semantic suites and full workspace test runs pass

- **Phase 3.4: Control Flow Graph (CFG) - Session 8 Integration & Validation** ✅
  - Integrated CFG reachability into semantic analysis for function bodies
  - Added CFG-driven unreachable diagnostics for unreachable top-level statements in functions
  - Preserved nested sequential unreachable detection to avoid duplicate/overlapping diagnostics
  - Added deterministic ordering and deduplication of CFG-derived unreachable positions for stable tests
  - Validation completed:
    * semantic unreachable suite: 25/25 passing
    * CFG suite: 70/70 passing
    * parser crate regression checks: passing

- **Phase 3.4: Control Flow Graph (CFG) - Session 7 Liveness Follow-up** ✅
  - Implemented CFG liveness analysis and unused-variable detection
  - Added liveness methods:
    * `compute_live_variables()` - backward dataflow live-out sets per block
    * `find_unused_variables()` - identifies definitions never live after statement
  - Added internal extraction helpers for def/use analysis:
    * target definition extraction for assignment-like statements
    * target/use expression traversal for identifiers in nested expressions
  - Added 5 liveness-focused CFG tests:
    * `test_liveness_linear_chain_no_unused`
    * `test_liveness_detects_unused_variable`
    * `test_liveness_augmented_assignment_uses_target`
    * `test_liveness_branch_uses_variable`
    * `test_live_variables_empty_at_exit_block`
  - CFG test suite increased from 65 to 70 tests, all passing

- **Phase 3.5: Deferred Semantic Validation** ✅
  - Implemented semantic validation for membership operators `in` and `not in`
  - Added conservative RHS validation with current type system:
    * Accepts container-like/unknown RHS types to avoid false positives
    * Rejects known scalar RHS types (`int`, `float`, `bool`, `None`) with `TypeMismatch`
  - Membership expression result type is now inferred as `bool`
  - Added semantic tests for:
    * Valid string membership (`in`, `not in`)
    * Invalid scalar RHS membership (int/None)
    * Conservative unknown RHS handling (no false-positive type mismatch)

- **Phase 3.5: Deferred Semantic Validation** ✅ (partial)
  - Added explicit semantic tests for all supported augmented assignment operators
  - New coverage includes: `+=`, `-=`, `*=`, `/=`, `//=`, `%=`, `**=`, `&=`, `|=`, `^=`, `>>=`, `<<=`
  - Added negative coverage to verify undefined-variable errors for each augmented operator variant
  - Files updated:
    * `crates/parser/src/semantic.rs` - added 2 new tests
    * `docs/ROADMAP.md` - marked augmented assignment explicit test task as complete

- **Phase 3.4: Control Flow Graph (CFG) - Session 7** ✅ (431 tests passing)
  - Implemented advanced CFG analysis: dominance analysis and visualization
  - Dominance Analysis Methods:
    * `compute_dominators()` - Iterative dominance computation
      - Returns HashMap<BlockId, HashSet<BlockId>> of all dominators per block
      - Uses iterative dataflow algorithm until convergence
      - Entry block dominates only itself, all others start dominated by all blocks
      - For each block B: dom(B) = {B} ∪ (∩ dom(pred) for all predecessors)
      - Handles loops, branches, and complex control flow correctly
    * `compute_immediate_dominators()` - Finds immediate dominator for each block
      - Returns HashMap<BlockId, Option<BlockId>> mapping blocks to their idom
      - Immediate dominator is unique closest strict dominator
      - Strict dominators = all dominators except block itself
      - idom is strict dominator not dominated by any other strict dominator
      - Entry block has no immediate dominator (returns None)
    * `compute_dominator_tree()` - Builds dominator tree structure
      - Returns HashMap<BlockId, Vec<BlockId>> representing tree relationships
      - Inverts idom relationship: parent → children
      - Each block maps to its children in dominator tree
      - Useful for visualizing dominance relationships
    * `dominates(x, y)` - Convenience method for dominance check
      - Returns bool indicating if block x dominates block y
      - Uses compute_dominators() internally
      - Simplifies dominance queries in client code
  - DOT Visualization:
    * `to_dot()` - Generates GraphViz DOT format for CFG visualization
      - Returns String in DOT format for rendering with GraphViz
      - Includes block IDs, block kinds, and statement counts
      - Color-codes blocks by kind:
        - Entry blocks: lightgreen
        - Exit blocks: lightcoral
        - Loop headers: lightyellow
        - Conditional blocks: lightblue
        - Exception handlers: orange
        - Normal blocks: white
      - Shows all edges between blocks (control flow)
      - Can be saved to .dot file and rendered: `dot -Tpng cfg.dot -o cfg.png`
  - Test Coverage (12 new tests, 65 total CFG tests):
    * Dominance Tests (7):
      - test_dominance_linear_code: Sequential code dominance properties
      - test_dominance_if_else: Branch and merge dominance
      - test_dominance_loop: Loop header dominates body
      - test_immediate_dominators: Entry has no idom, all others have one
      - test_dominator_tree: Tree structure validation
      - test_dominance_nested_if: Nested conditional dominance
      - test_dominance_complex_control_flow: Loop with break
    * DOT Visualization Tests (5):
      - test_dot_simple_linear: Basic DOT generation
      - test_dot_with_conditional: Branching visualization
      - test_dot_with_loop: Loop structure with back edges
      - test_dot_block_colors: Color coding verification
      - test_dot_statement_counts: Block label content
  - Implementation Details:
    * Dominance analysis uses standard compiler theory algorithms
    * Iterative dataflow for dominators (not Lengauer-Tarjan for simplicity)
    * DOT format compatible with GraphViz tools (dot, neato, circo, etc.)
    * Foundation for future optimizations (dead code elimination, loop invariant motion)
    * Liveness analysis deferred to future session (complex AST interactions)
  - Notes:
    * Session 7 originally included liveness analysis, but simplified for now
    * Liveness requires extensive AST field name handling across many node types
    * Current implementation focuses on dominance and visualization
    * Liveness can be added in future sessions as optional enhancement

- **Phase 3.4: Control Flow Graph (CFG) - Session 6** ✅ (419 tests passing)
  - Implemented CFG reachability analysis to detect unreachable code
  - Reachability Analysis Methods:
    * `compute_reachable_blocks()` - DFS-based reachability computation
      - Uses HashSet to track visited blocks efficiently
      - Stack-based depth-first search from entry block
      - Follows all successor edges to find reachable blocks
      - Returns HashSet<BlockId> of all reachable blocks
    * `find_unreachable_blocks()` - Identifies unreachable blocks
      - Computes reachable blocks first
      - Filters all blocks to find unreachable ones
      - Returns Vec<BlockId> of unreachable block IDs
    * `is_block_reachable()` - Convenience method for single block check
      - Returns bool indicating if specific block is reachable
      - Useful for targeted reachability queries
  - Algorithm Design:
    * DFS traversal starting from CFG entry block
    * HashSet for O(1) membership checking
    * Stack-based iteration (no recursion)
    * Handles all control flow: conditionals, loops, try/except, returns, raises
  - Test Coverage (12 new tests, 53 total CFG tests):
    * test_reachability_simple_function: All blocks reachable (baseline)
    * test_reachability_unreachable_after_return: Code after return statement
    * test_reachability_unreachable_after_raise: Code after raise statement
    * test_reachability_unreachable_after_break: Code after break in loop
    * test_reachability_unreachable_after_continue: Code after continue
    * test_reachability_if_else_all_return: Both branches return → after unreachable
    * test_reachability_if_one_branch_returns: One branch returns → after reachable
    * test_reachability_try_except_all_return: All handlers return → after unreachable
    * test_reachability_nested_unreachable: Multiple nested unreachable blocks
    * test_reachability_complex_loop_scenario: Nested loops with multiple exits
    * test_reachability_is_block_reachable: Test convenience method directly
    * test_reachability_try_finally_reachable: Finally block always reachable
  - Implementation Details:
    * Reachability is structural property of CFG graph
    * Does not require semantic analysis or type information
    * Foundation for unreachable code warnings in semantic analyzer
    * Correctly handles all exit points (return, raise, break, continue)

- **Phase 3.4: Control Flow Graph (CFG) - Session 5** ✅ (407 tests passing)
  - Implemented CFG builder for try/except/else/finally blocks with exception handling
  - Try/Except Processing:
    * `process_try_statement()` - Main handler for exception control flow
      - Creates try block for protected code
      - Creates handler blocks for each except clause
      - Creates optional else block (executes if no exception)
      - Creates optional finally block (always executes)
      - Creates merge block (after try/except)
      - Exception paths: try → handlers (on exception)
      - Normal path: try → else (if present) → merge
      - All paths converge: merge → finally (if present)
  - Exception Handler Context:
    * Added exception_handlers field to CFGBuilder (Vec<BlockId>)
    * Stack-based exception handler tracking for nested try blocks
    * Handlers saved/restored when entering/exiting try blocks
  - Raise Statement Handling:
    * Updated Statement::Raise to check for exception handlers
    * If in try block: connects to all exception handlers
    * If not in try: connects to function exit (unhandled exception)
    * Creates unreachable block after raise
  - Exception Flow Semantics:
    * Try block connects to all handler blocks via exception edges
    * Else block only executes if no exception raised
    * Finally block always executes from all paths
    * Handlers and else both connect to merge block
    * Nested try blocks correctly maintain handler stack
  - Test Coverage (8 new tests, 41 total CFG tests):
    * test_simple_try_except: Basic try with single handler
    * test_try_multiple_except: Multiple exception handlers
    * test_try_except_else: Else block execution on success
    * test_try_except_finally: Finally always executes
    * test_try_except_else_finally: All components together
    * test_nested_try_blocks: Nested try with handler stacks
    * test_raise_in_try: Raise connects to handler, unreachable after
    * test_raise_without_handler: Raise to exit, unreachable after
  - Implementation Details:
    * Exception handlers stored as Vec<BlockId> for current try context
    * Raise statements connect to exception_handlers or function exit
    * Finally blocks reachable from all execution paths
    * Proper restoration of exception context on exit from try

- **Phase 3.4: Control Flow Graph (CFG) - Session 4** ✅ (399 tests passing)
  - Implemented CFG builder for while loops and for loops with break/continue
  - While Loop Processing:
    * `process_while_statement()` - Main handler for while loops
      - Creates loop header block (BlockKind::LoopHeader) for condition
      - Creates loop body block (BlockKind::LoopBody)
      - Creates loop exit block for merge after loop
      - Adds condition evaluation to header block
      - True branch: header → body → header (back-edge)
      - False branch: header → exit (or else block if present)
  - For Loop Processing (Session 4.3):
    * `process_for_statement()` - Main handler for for loops
      - Evaluates iterator in current block before loop
      - Creates loop header block (BlockKind::LoopHeader) with target assignment
      - Creates loop body block (BlockKind::LoopBody)
      - Creates loop exit block for merge after loop
      - Has next: header → body → header (back-edge)
      - No more: header → exit (or else block if present)
  - Break/Continue Handling:
    * Updated Statement::Break handler to create edge to loop exit
    * Updated Statement::Continue handler to create edge to loop header
    * Both create unreachable blocks after statement
    * Uses break_targets and continue_targets stacks for nested loops
  - While-Else and For-Else Support:
    * False/no-more branch from header goes to else block (if present)
    * Else block connects to exit after execution
    * Break bypasses else block, goes directly to exit
    * Correct semantics: else executes only if loop completes normally
  - Nested Loops:
    * Stack-based loop context tracking
    * Break/continue connect to correct loop level
    * Multiple loop headers and bodies properly created
  - Test Coverage (12 new tests, 33 total CFG tests):
    * While Loops (6 tests):
      - test_simple_while_loop: Basic while with back-edge (6+ blocks)
      - test_while_with_break: Break creates edge to exit, unreachable after
      - test_while_with_continue: Continue creates edge to header, unreachable after
      - test_while_else_no_break: Else executes when no break
      - test_while_else_with_break: Break bypasses else block
      - test_nested_while_loops: Nested loops with correct stack handling
    * For Loops (6 tests):
      - test_simple_for_loop: Basic for with iterator and back-edge
      - test_for_with_break: Break creates edge to exit, unreachable after
      - test_for_with_continue: Continue creates edge to header, unreachable after
      - test_for_else_no_break: Else executes when no break
      - test_for_else_with_break: Break bypasses else block
      - test_nested_for_in_while: Nested for inside while loop
  - Implementation Details:
    * Loop header block has BlockKind::LoopHeader
    * Loop body block has BlockKind::LoopBody
    * Back-edge from body to header for iteration
    * Push/pop loop context (header, exit) for break/continue
    * For loops: iterator evaluation before loop, target assignment in header

- **Phase 3.4: Control Flow Graph (CFG) - Session 3** ✅ (387 tests passing)
  - Implemented CFG builder for conditional statements (if/elif/else)
  - Conditional Processing:
    * `process_if_statement()` - Main handler for if statements
      - Adds condition evaluation to current block
      - Creates then-block and processes statements
      - Calls process_elif_else_chain for elif/else handling
      - Collects all branch exits (then, elif, else) in vector
      - Creates merge block for branch convergence
      - Connects reachable branches to merge
      - Handles false branch to merge if no else block
    * `process_elif_else_chain()` - Handles elif/else chains
      - Creates elif condition blocks (BlockKind::Conditional)
      - Chains conditions: if condition false → elif condition
      - Creates elif then-blocks for each elif statement
      - Processes else block if present
      - Returns Vec<BlockId> of all branch exits and has_else flag
  - Branching Patterns:
    * Simple if (no else): condition → then → merge
    * If-else: condition → then/else → merge
    * If-elif-else: chained conditions → multiple branches → merge
    * Nested if: recursive processing with correct block structure
  - Edge Cases Handled:
    * Return/raise in branches: unreachable detection
    * All branches exit: merge block still created (unreachable)
    * Mixed reachability: only reachable branches connect to merge
  - Test Coverage (5 new tests, 21 total CFG tests):
    * test_simple_if_no_else: Basic if without else (5 blocks)
    * test_if_else: If with else block (6 blocks)
    * test_if_elif_else: Full elif chain (8 blocks)
    * test_nested_if: Nested conditionals (7+ blocks)
    * test_if_with_return_in_then: Return in then branch
    * test_if_else_both_return: Returns in both branches (unreachable after)
  - Bug Fix:
    * Fixed elif branch tracking: changed return type from (Option<BlockId>, bool) to (Vec<BlockId>, bool)
    * All branch exits now properly connect to merge block

- **Phase 3.4: Control Flow Graph (CFG) - Session 2** ✅ (382 tests passing)
  - Implemented CFG builder for linear code (sequences without control flow)
  - CFG Builder Structure:
    * `CFGBuilder` struct with cfg, current_block, exit_block, loop targets
    * `new()` constructor initializing with entry block
    * `add_statement_to_current_block()` helper method
  - CFG Building:
    * `build_function_cfg(function)` - Entry point for building function CFGs
    * Returns `Result<ControlFlowGraph, String>` with error handling
    * Automatically creates entry, exit, and normal blocks
    * Detects unreachable code (blocks with no predecessors)
  - Statement Processing:
    * Linear statements: Assignment, AugmentedAssignment, AnnAssignment, Expression, Pass
    * Control transfer: Return (edge to exit), Raise (edge to exit)
    * Unreachable block creation after return/raise
    * Other statements: Import, FromImport, Global, Nonlocal, Assert, Del
    * Function/class definitions treated as statements
    * Error reporting for unsupported control flow (if/while/for/try)
  - Features:
    * Automatic entry → first block edge creation
    * Exit block connection for reachable code
    * Unreachable block detection (no predecessors, no exit edge)
    * Proper handling of multiple returns/raises
  - Test Coverage: 7 new CFG builder tests (16 total CFG tests)
    * Empty function with pass statement
    * Function with multiple assignments
    * Function with return in middle (unreachable code after)
    * Function with multiple returns
    * Function with raise statement
    * Rejection of non-function statements
    * Error for unsupported control flow statements
  - Foundation for Session 3: conditionals (if/elif/else)

- **Phase 3.4: Control Flow Graph (CFG) - Session 1** ✅ (375 tests passing)
  - Implemented core CFG data structures for sophisticated program analysis
  - Data Structures:
    * `BasicBlock` struct with id, kind, statements, successors, predecessors, position
    * `ControlFlowGraph` struct managing blocks, entry/exit points, block generation
    * `BlockKind` enum: Entry, Exit, Normal, Conditional, LoopHeader, LoopBody, ExceptionHandler
    * `BlockId` type alias for unique block identification
  - CFG Operations:
    * `new_block()` - Create new basic blocks with unique IDs
    * `add_edge()` - Create directed edges between blocks (bidirectional linking)
    * `remove_edge()` - Remove edges between blocks
    * `add_exit_block()` - Mark blocks as function exits
    * Query methods: `get_block()`, `block_count()`, `entry()`, `exits()`
  - Basic Block Operations:
    * `add_statement()` - Add statements to blocks
    * `add_successor()` / `add_predecessor()` - Manage edges
    * Query methods: `is_empty()`, `has_successors()`, `has_predecessors()`
  - Features:
    * Automatic entry block creation
    * Duplicate edge prevention
    * Bidirectional edge tracking (successors and predecessors)
    * Support for multiple exit blocks
  - Test Coverage: 9 comprehensive CFG tests
    * CFG creation and entry block
    * Block ID generation
    * Edge addition and removal
    * Exit block management
    * Basic block operations
    * Duplicate edge handling
    * Linear CFG construction (entry → blocks → exit)
    * Branching CFG construction (if/else with merge points)
  - Foundation for future analysis: reachability, dominance, liveness, dead code elimination

- **Phase 2: Try/Except Statement Support** ✅
  - Implemented full try/except/else/finally syntax support
  - AST Definition:
    * Added `Statement::Try` variant with body, handlers, orelse, finalbody fields
    * Added `ExceptHandler` struct with exception_type, name, body, position
  - Parser Implementation:
    * Added `parse_try()` method to parse try/except/else/finally statements
    * Added `parse_except_handler()` method to parse exception handlers
    * Supports bare `except:` and typed `except ExceptionType:` handlers
    * Supports exception variable binding with `as` clause
    * Validates `else` requires at least one except handler
    * Validates at least one except or finally required
  - Semantic Analysis:
    * Implemented `visit_try_statement()` with proper exception variable scoping
    * Exception variables (from `as` clause) scoped to handler block only
    * Detects undefined variables in all try/except/else/finally blocks
    * Handles redeclaration errors for exception variables
  - Unreachable Code Detection:
    * Extended `statement_always_exits()` to handle Try statements
    * Added `check_try_all_branches_exit()` for control flow analysis
    * Detects unreachable code when:
      - Try body exits AND all except handlers exit AND (no else OR else exits)
      - Finally block exits (always executes last)
    * Works with return, break, continue, raise statements
  - Test Coverage: 38 comprehensive tests
    * 14 parser tests (syntax validation, nesting, error cases)
    * 11 semantic tests (scoping, variable visibility, undefined variables)
    * 13 unreachable code tests (all exit combinations, finally behavior)

- **Phase 3.4: Advanced Control Flow Analysis - Unreachable Code Detection Phase 2** ✅ (324 tests passing)
  - Extended unreachable code detection to handle branch analysis
  - Detects unreachable code after if/else blocks where all branches exit
  - Added three helper methods working together:
    * `statement_always_exits()`: Enhanced to recursively check If statements
    * `check_if_all_branches_exit()`: Validates if/else blocks have all branches exiting
    * `block_contains_exit()`: Recursively searches for exit statements in blocks
  - Detection logic:
    * Must have else clause (all paths covered)
    * All branches (if, elif, else) must contain exit statement
    * Exit statements: return, break, continue, raise
    * Works recursively with nested if/else blocks
  - Examples detected:
    * `if cond: return 1; else: return 2; print("unreachable")` ✗
    * `if x==1: return 1; elif x==2: return 2; else: return 3; print("unreachable")` ✗
    * `while True: if cond: break; else: break; print("unreachable")` ✗
    * Nested if/else where all paths exit ✗
  - Examples NOT detected (conservative approach):
    * `if cond: return 1; print("reachable")` ✓ (no else clause)
    * `if x==1: return 1; elif x==2: return 2; print("reachable")` ✓ (no else)
    * `if cond: return 1; else: x=42; print("reachable")` ✓ (else doesn't exit)
    * Module-level if/else blocks (even if all exit) ✓
  - Added 20 comprehensive tests:
    * 13 tests for unreachable code detection (simple, elif, nested, mixed exits)
    * 7 tests for reachable code (missing else, partial exits, module-level)
  - Combines seamlessly with Phase 1 sequential unreachable detection
  - Conservative approach: only reports when certain all paths exit
  - Test count increased from 304 → 324 (+20 tests)

- **Phase 3.3: Semantic Validation - Invalid Assignment Target Validation** ✅ (320 tests passing)
  - Added comprehensive validation for assignment targets in assignments and augmented assignments
  - Added `InvalidAssignmentTarget` error variant with descriptive target type
  - Validates assignment targets recursively:
    * **Valid targets**: identifiers, tuples, lists, subscripts, attributes, starred expressions
    * **Invalid targets**: literals, function calls, operations, comparisons, lambdas, comprehensions
  - Validation rules:
    * Identifier assignments: `x = 5` ✓
    * Tuple/list unpacking: `a, b = 1, 2` or `[a, b] = [1, 2]` ✓
    * Starred unpacking: `a, *b, c = [1, 2, 3, 4]` ✓
    * Subscript assignments: `mylist[0] = 10` ✓
    * Attribute assignments: `obj.attr = 5` ✓
    * Nested unpacking recursively validated: `(a, (b, c)) = (1, (2, 3))` ✓
    * Parenthesized expressions checked recursively: `(x) = 5` ✓
  - Error cases detected:
    * Literal assignments: `5 = x`, `"hello" = x`, `True = x`, `None = x` ✗
    * Operation assignments: `x + y = 5`, `not x = False` ✗
    * Call assignments: `foo() = x` ✗
    * Collection literals: `{} = x`, `{1, 2} = x` ✗
    * Comprehensions: `[x for x in range(10)] = foo` ✗
    * Conditional expressions: `(x if True else y) = 5` ✗
    * Lambda assignments: `lambda x: x = foo` ✗
  - Augmented assignments validated same as regular: `5 += 1` ✗
  - Added 12 comprehensive tests:
    * 7 valid target tests (identifier, subscript, attribute, tuple/list/starred/nested unpacking)
    * 5 invalid target tests (dict, set, comprehension, conditional, augmented conditional)
  - Works in cooperation with parser - parser catches some invalid assignments early
  - Clear error messages: "Cannot assign to literal/operator/function call/etc."
  - **Note**: Handles all core assignment validation - future enhancements may add more specific type-based checks

- **Phase 3.3: Semantic Validation - Operator Usage Validation** ✅ (308 tests passing)
  - Extended operator type checking to cover all operator categories:
    * **Bitwise operators** (&, |, ^, <<, >>): Require integer types (int or bool)
    * **Comparison operators** (==, !=, <, <=, >, >=): Validate type compatibility
    * **Logical operators** (and, or): Accept any type (truthy/falsy semantics)
    * **Identity operators** (is, is not): Accept any type (reference comparison)
  - Comparison validation rules:
    * Equality (==, !=) operators are permissive - allow any type comparisons
    * Ordering (<, <=, >, >=) requires compatible types (no str < int mixing)
    * String ordering: both operands must be strings
    * Numeric ordering: allows int/float/bool mixing
    * None cannot be used in ordering operations
  - Added 14 comprehensive tests:
    * 4 bitwise operator tests (valid int, error with string/float, valid bool)
    * 2 shift operator tests (valid int, error with string)
    * 5 comparison tests (string equality, string ordering, mixed type error, numeric, None)
    * 2 equality tests (None comparison valid, mixed types valid)
    * 2 logical operator tests (any type valid)
  - Clear error messages: "expected int" for bitwise, "expected str/comparable types" for ordering
  - **Note**: Membership operators (in, not in) deferred until collection types are implemented

- **Phase 3.3: Semantic Validation - Function Call Argument Validation** ✅ (294 tests passing)
  - Added function signature tracking system:
    * `FunctionSignature` struct stores function metadata (name, parameters, return type)
    * `Parameter` struct tracks parameter name, type, and whether it has a default
    * `function_signatures` HashMap stores all function signatures for validation
  - Added 3 new error variants:
    * `UndefinedFunction`: Function called but not defined
    * `ArgumentCountMismatch`: Wrong number of arguments (shows min-max range for defaults)
    * `ArgumentTypeMismatch`: Argument type doesn't match parameter type annotation
  - Validates function calls:
    * Checks function exists before calling
    * Validates argument count matches parameter count (accounting for defaults)
    * Type-checks arguments when both parameter and argument types are known
  - Skips validation for built-in functions (print, len, range, etc.) - accepts any arguments
  - Added 18 comprehensive tests:
    * 4 undefined/count mismatch cases (undefined, too few, too many, no params with args)
    * 6 correct usage cases (correct count, with defaults, all defaults, no annotations)
    * 4 type checking cases (mismatch, correct types, multiple errors)
    * 4 edge cases (nested, recursive, in expressions, built-ins)
  - Clear error messages with function name, expected/actual counts and types
  - **Note**: Only validates simple positional arguments with optional defaults - keyword arguments, *args/**kwargs deferred

- **Phase 3.3: Semantic Validation - Unreachable Code Detection** ✅ (276 tests passing)
  - Added `UnreachableCode` error variant
  - Detects code that cannot be executed after exit statements (return/break/continue/raise)
  - Implements sequential statement analysis within statement blocks
  - Each if/elif/else branch analyzed independently (no cross-branch analysis)
  - Added helper methods:
    * `statement_always_exits()`: Checks if a statement is an exit statement
    * `visit_statement_list()`: Analyzes statement sequences with exit tracking
  - Updated all statement list processing: module, function body, class body, loops, if/elif/else
  - Added 10 comprehensive tests:
    * 5 detection cases (after return, break, continue in various contexts)
    * 3 non-detection cases (after if-return, else-return, last statement)
    * 2 edge cases (pass after return, nested loops)
  - Clear error message: "Unreachable code"
  - **Note**: Simple sequential analysis only - complex control flow (all branches return, try/except) deferred to Phase 3.4

- **Phase 3.3: Semantic Validation - Return Statement Validation** ✅ (266 tests passing)
  - Added `ReturnOutsideFunction` error variant
  - Validates that `return` statements only appear inside function definitions
  - Uses existing `current_function` tracking from type inference system
  - Works correctly with nested functions (each tracks its own context)
  - Added 12 comprehensive tests:
    * 7 valid cases (return in functions, nested functions, with/without values, in loops/if blocks)
    * 5 invalid cases (return at module level, in if blocks, after functions, in class bodies)
  - Clear error message: "'return' outside function"

- **Phase 3.3: Semantic Validation - Break/Continue Validation** ✅ (254 tests passing)
  - Added `loop_depth` field to SemanticAnalyzer for tracking loop nesting
  - Added `BreakOutsideLoop` and `ContinueOutsideLoop` error variants
  - Validates that `break` statements only appear inside loops (while/for)
  - Validates that `continue` statements only appear inside loops (while/for)
  - Correctly handles loop else blocks (not considered part of loop)
  - Supports nested loops with proper depth tracking
  - Added 15 comprehensive tests:
    * 6 valid cases (break/continue in while, for, nested loops)
    * 7 invalid cases (break/continue at module level, in functions, in if blocks, in else blocks)
    * 2 edge cases (multiple break/continue, function with loops)
  - Clear error messages: "'break' outside loop", "'continue' not properly in loop"

### Changed
- **Integrated type tracking into SymbolTable (Phase 3.2 Refactoring)** ✅ (239 tests passing)
  - Removed separate `TypeTable` structure that caused scope-awareness issues
  - Added `inferred_type: Type` field to `Symbol` struct in SymbolTable
  - Added `assign_type()` and `get_type()` methods to SymbolTable for scope-aware type tracking
  - Updated SemanticAnalyzer to use SymbolTable for all type operations
  - Type tracking now automatically follows scope hierarchy (module → function → block)
  - Eliminated duplicate name tracking between TypeTable and SymbolTable
  - Updated test `test_type_table_storage` → `test_symbol_table_type_storage`
  - Added 4 new scope-awareness tests (variable shadowing, parameter scoping, block scope, nested scope isolation)
  - **Architecture improvement**: Single source of truth for both declarations and types
  - **No user-facing changes**: Internal refactoring only, all existing tests pass

- **Phase 3.2: Type Inference (Basic)** ✅ COMPLETE (239 tests passing)
  
  **Task 1: Type System Foundation & Literal Type Inference** ✅
  - Type system foundation with `Type` enum (Int, Float, String, Bool, None, Unknown)
  - Type compatibility checking for Python-style dynamic typing
  - Literal type inference for all Python literal types
  - 18 comprehensive tests (6 type system + 12 type inference)
  
  **Type system**:
    - `Type` enum: Int, Float, String, Bool, None, Unknown
    - Type compatibility checking (`is_compatible_with`)
    - Display implementation for type names
    - Numeric type compatibility (Int ↔ Float, Bool ↔ Int, Bool ↔ Float)
  
  **Type inference**:
    - Literal type inference: integers, floats, strings, booleans, None
    - Built-in constant types: True (bool), False (bool), None (None)
    - `infer_type` method for expression type inference
    - Type lookup for identifiers from symbol table
  
  **Task 2: Variable Type Inference from Assignments** ✅
  - Variable type inference from all assignment forms
  - `assign_type_to_names()` helper for recursive type assignment to targets
  - Statement::Assignment: infers value type and assigns to all targets
  - Statement::AnnAssignment: infers from value or uses Unknown
  - Expression::AssignmentExpr (walrus): infers and assigns types
  - Handles multiple assignment chains, reassignment, identifier chain assignment
  - Supports Python-style dynamic typing (last assignment wins)
  - 20 comprehensive tests covering all assignment scenarios

  **Task 3: Function Return Type Inference** ✅
  - Function return type tracking with `function_types` HashMap
  - Current function context tracking during analysis
  - Return statement type inference from return expressions
  - Function call type inference (calls inherit function's return type)
  - Handles functions with no return (None type)
  - Handles multiple return paths (last return wins for now)
  - Type propagation through function calls in assignments
  - 20 comprehensive tests across 4 subtasks:
    - Basic return type tracking (5 tests)
    - Single return statement inference (5 tests)
    - Multiple return paths (6 tests)
    - Using function return types in calls (4 tests)

  **Task 4: Binary Operation Result Type Inference** ✅
  - Binary operation type inference for all arithmetic, comparison, and logical operators
  - Unary operation type inference (not, -, +, ~)
  - Type promotion rules (Int + Float → Float)
  - Python 3 division always returns Float (Int / Int → Float)
  - String concatenation (String + String → String)
  - Comparison operations always return Bool
  - Logical operations (and, or) for Bool types
  - Parenthesized expression handling
  - Helper methods: `infer_binary_op_type()` and `infer_unary_op_type()`
  - 20 comprehensive tests across 5 subtasks:
    - Arithmetic operations (5 tests)
    - Comparison operations (5 tests)
    - Logical operations (4 tests)
    - Unary operations (3 tests)
    - Complex nested expressions (3 tests)

  **Task 5: Track Type Through Control Flow** ✅
  - Type tracking through if/else statements
  - Type tracking through while and for loops
  - For-loop variables assigned Unknown type (iterable element types not tracked yet)
  - "Last write wins" approach for type assignments in branches
  - Variables assigned in any branch are tracked and accessible after
  - Type propagation through nested control structures
  - Comprehensive testing of all control flow patterns
  - 19 comprehensive tests across 5 subtasks:
    - Basic if statement tracking (4 tests)
    - If-else type merging (5 tests)
    - Nested if statements (3 tests)
    - While loop tracking (3 tests)
    - For loop tracking (4 tests)

  **Task 6: Type Mismatch Detection** ✅
  - Type mismatch error detection for invalid operations
  - `TypeMismatch` variant added to `SemanticError` enum
  - `check_binary_op_types()` validates binary operation operands before inference
  - `check_unary_op_types()` validates unary operation operands
  - `parse_type_annotation()` helper converts annotation expressions to Type enum
  - Division by zero detection for Divide and FloorDivide operations
  - Type annotation checking for annotated assignments
  - Return type annotation checking for function returns
  - Expected return type tracking with `expected_return_type` field
  - Conservative approach: only reports errors when both types are known (not Unknown)
  - 16 comprehensive tests across 5 subtasks:
    - Binary operation mismatches (4 tests)
    - Annotated assignment mismatches (4 tests)
    - Division by zero (2 tests)
    - Unary operation mismatches (3 tests)
    - Return type annotation mismatches (3 tests)

- **Phase 3.1: Symbol Table & Semantic Analysis** ✅ Complete (113 tests passing)
  - Complete symbol table implementation with scope hierarchy management
  - Semantic analyzer with visitor pattern for comprehensive AST analysis
  - Full variable and function tracking across all scopes
  - Undefined variable detection with detailed error reporting
  - Redeclaration detection with proper shadowing support
  - Nested scope support for functions, classes, and control flow
  - Closure tracking with global/nonlocal declarations
  - Built-in functions and constants pre-declared
  - Expression visiting in Return, Assert, Del, Raise statements
  - Walrus operator (`:=`) reassignment support
  - 113 comprehensive tests (11 symbol table + 102 semantic analyzer)
  
  **Core symbol table data structures**:
    - `SymbolKind` enum: Variable, Function, Class, Parameter
    - `Symbol` struct: tracks name, kind, position, scope, capture/global/nonlocal flags
    - `ScopeKind` enum: Module, Function, Class, Block
    - `Scope` struct: HashMap-based symbol storage with parent/child tracking
    - `SymbolTable`: manages scope hierarchy with enter/exit/declare/lookup methods
  
  **Semantic analyzer features**:
    - `SemanticAnalyzer` struct with symbol table and error tracking
    - `SemanticError` enum: UndefinedVariable, Redeclaration, InvalidScope, NonlocalAtModuleLevel, NonlocalNotFound, GlobalAtModuleLevel
    - AST visitor pattern for statements and expressions
    - Module analysis with error reporting
  - 7 semantic analyzer smoke tests
  - Variable declaration tracking:
    - Simple assignments (x = 5)
    - Multiple assignments (x = y = 10)
    - Tuple/list unpacking (a, b = 1, 2)
    - Nested unpacking ((a, (b, c)) = ...)
    - Starred unpacking (a, *rest = ...)
    - Annotated assignments (x: int = 5)
    - Augmented assignments (x += 1) with undefined variable detection
    - Redeclaration detection in same scope
  - 19 additional variable tracking tests (26 total)
  - Function definition tracking:
    - Function declarations with SymbolKind::Function
    - Function scope management (enter/exit)
    - Parameter declarations with SymbolKind::Parameter
    - Nested function support
    - Function redeclaration detection
    - Duplicate parameter detection
    - Support for async functions, decorators, return types, default parameters
  - 15 additional function tests (41 total)
  - Variable usage detection:
    - Identifier lookup with UndefinedVariable errors
    - Recursive expression visiting (BinaryOp, UnaryOp, Call, Subscript, Attribute)
    - Collection expressions (List, Tuple, Dict, Set)
    - Conditional expressions
    - Walrus operator (assignment expressions)
    - Nested scope variable access
    - Parameter usage in functions
  - 16 additional usage detection tests (57 total)
  - Redeclaration detection and shadowing:
    - Redeclaration errors in same scope (variables, functions, parameters)
    - Proper shadowing support across nested scopes
    - Parameter vs body variable conflict detection
    - Function/variable name conflicts
    - Mixed type redeclarations
    - Walrus operator redeclaration handling
  - 11 additional redeclaration/shadowing tests (68 total)
  - Nested scope support:
    - Control flow statements (if/while/for) don't create new scopes
    - Variables in if/while/for blocks persist in enclosing scope
    - For loop variables accessible after loop
    - Tuple unpacking in for loops
    - Class definitions create new scopes (with proper isolation)
    - Class name declaration and redeclaration detection
    - Nested classes in functions and methods in classes
    - Deeply nested scope combinations
    - If/elif/else, while-else, for-else handling
    - Built-in functions and constants (print, range, len, str, int, float, bool, list, dict, set, tuple, True, False, None)
  - 14 additional nested scope tests (82 total)
  - Closure tracking and global/nonlocal:
    - Global declarations in functions (modify module-level variables)
    - Global at module level (allowed, redundant)
    - Multiple global declarations
    - Global after local declaration error detection
    - Nonlocal declarations in nested functions
    - Nonlocal at module level error detection
    - Nonlocal variable not found error detection
    - Nonlocal skips module scope (function scopes only)
    - Multiple nonlocal declarations
    - Nonlocal after local declaration error detection
    - Basic closures (inner function references outer variable)
    - Multi-level closures
    - Global and nonlocal for different variables
    - Nonlocal finds nearest enclosing scope
    - Nonlocal in class methods
    - Symbol tracking: is_captured, is_global, is_nonlocal flags
  - 15 additional closure/global/nonlocal tests (97 total)
- **Phase 1: Lexer & Tokenization (Complete!)**
  - Full tokenization of Python-compatible syntax
  - Support for all Python operators, keywords, and delimiters
  - Number literals: integers, floats, hex (0x), octal (0o), binary (0b)
  - String literals: single/double quotes, raw strings (r""), f-strings (f""), triple-quoted multiline strings
  - Comment handling (# comments)
  - Indentation tracking with INDENT/DEDENT tokens (Python's significant whitespace)
  - Unicode identifier support (Greek, Chinese, Japanese, Arabic, Cyrillic, etc.)
  - Comprehensive test suite:
    - 82 lexer unit tests (including 50+ edge case tests)
    - 23 indentation tests
    - 5 token tests
    - 17 Unicode tests
    - 15 property-based tests (fuzzing alternative)
    - **142 total tests, all passing**
  - Performance benchmarks with criterion:
    - Small files: ~527 ns (simple assignment)
    - Medium files: ~16.9 µs (50-line module)
    - Large files: ~288 µs (1000 assignments)
    - Throughput: ~3.5M lines/second
  - Fuzzing infrastructure (cargo-fuzz + proptest)
  - Error handling with position tracking
- **Phase 2: Parser & AST (In Progress)**
  - Bug fixes:
    - Fixed binary operator position tracking: operators now correctly report their own position instead of the right operand's position (added `previous_position` field to Parser)
  - Complete AST node definitions for expressions, statements, and literals
  - Recursive descent parser with operator precedence climbing
  - Expression parsing:
    - All literals (int, float, string, bool, None)
    - Identifiers and parenthesized expressions
    - Binary operators with correct precedence (or, and, not, comparison, bitwise, shift, arithmetic, power)
    - Unary operators (-, +, ~, not)
    - All comparison operators (==, !=, <, >, <=, >=)
    - Membership operators (in, not in)
    - Identity operators (is, is not)
    - Function calls with arbitrary arguments, including nested calls and trailing commas
    - Subscript operations with any expression as index, supporting chained subscripts and mixed with calls
    - Attribute access with chaining, enabling method calls and complex postfix expressions
    - List literals with support for empty lists, trailing commas, nested lists, and arbitrary expressions
    - Tuple literals with proper disambiguation from parenthesized expressions, supporting empty tuples and single-element tuples
    - Dict literals with key-value pairs, supporting empty dicts, trailing commas, nested dicts, expressions as keys/values
    - Set literals with proper disambiguation from dicts (empty braces = dict), supporting trailing commas, expressions as elements
    - Lambda expressions with parameter lists and expression bodies, enabling functional programming patterns
    - Conditional expressions (ternary operator) with proper precedence and chaining support
    - Walrus operator / assignment expressions (:=) for inline assignment within expressions
    - Ellipsis literal (...) for use in slicing, type hints, and as placeholder
    - List comprehensions ([expr for target in iter if cond]) with support for multiple generators and conditions
    - Dict comprehensions ({key: value for target in iter if cond}) with support for complex key-value expressions
    - Set comprehensions ({expr for target in iter if cond}) with support for nested loops and conditions
    - Generator expressions ((expr for target in iter if cond)) for lazy evaluation with full comprehension syntax
  - Statement parsing:
    - Assignment statements (x = 5)
    - Multiple/chained assignment (x = y = z = 5)
    - Tuple unpacking assignment (a, b = 1, 2 or x, y, z = tuple)
    - Starred unpacking assignment (a, *b, c = [1, 2, 3, 4])
    - Augmented assignment (+=, -=, *=, /=, //=, %=, **=, &=, |=, ^=, >>=, <<=)
    - Expression statements
    - Pass, break, continue, return statements
    - Assert statement with optional message (assert condition, "message")
    - Del statement for deleting variables, attributes, or subscripts (del x, del obj.attr, del list[0])
    - Global statement for declaring global variables (global x, y)
    - Nonlocal statement for declaring nonlocal variables (nonlocal x, y)
    - Raise statement for raising exceptions (raise, raise Exception, raise Exception("msg"))
    - Import statement with dotted module names and aliases (import os, import os.path, import numpy as np)
    - From...import statement with dotted module names, aliases, and wildcard (from os import path, from os.path import join as j, from os import *)
    - Control flow statements:
      * If/elif/else statements with proper indentation handling and chained elif blocks
      * While loops with optional else clause (executes if loop completes without break)
      * For loops with optional else clause, supporting tuple unpacking (for x, y in items)
    - Function definitions:
      * Basic function definitions with def keyword (def foo(): pass)
      * Async function definitions with async def keyword (async def foo(): pass)
      * Function parameters (simple, multiple, default values)
      * Variadic positional parameters (*args)
      * Variadic keyword parameters (**kwargs)
      * Keyword-only parameters (after * or *args)
      * Bare * separator for keyword-only parameters (def func(a, *, b): pass)
      * Keyword-only parameters with defaults (def func(*, a=1, b=2): pass)
      * Positional-only parameters (before /)
      * / separator for positional-only parameters (def func(a, /, b): pass)
      * Positional-only parameters with defaults (def func(a=1, /, b): pass)
      * Mixed positional-only, regular, and keyword-only parameters
      * Full parameter combination support (pos-only → regular → defaults → *args → keyword-only → **kwargs)
      * Async functions with all parameter types (async def func(a, /, b, *args, c, **kwargs): pass)
      * Parameter order validation with all parameter types
      * Nested function definitions
      * Async methods in classes
      * Complex default values (expressions, lists, dicts)
      * Parameter type annotations (x: int, y: str)
      * Type annotations for all parameter kinds (regular, positional-only, keyword-only, *args, **kwargs)
      * Generic type annotations (List[int], Optional[str], List[List[int]])
    - Return type annotations:
      * Function return types (def foo() -> int: pass)
      * Generic return types (def foo() -> list[int]: pass, def bar() -> dict[str, int]: pass)
      * Nested generic return types (def foo() -> list[dict[str, int]]: pass)
      * Union types (def foo() -> int | str: pass)
      * Optional types (def foo() -> Optional[int]: pass)
      * Async function return types (async def foo() -> int: pass)
      * Complex return types (Callable, etc.)
    - Class definitions:
      * Basic class definitions with class keyword (class Foo: pass)
      * Class body with statements and methods
      * Single inheritance (class Child(Parent): pass)
      * Multiple inheritance (class Child(Parent1, Parent2): pass)
      * Inheritance with dotted names (class Child(pkg.Module): pass)
      * Nested class definitions
      * Complex class bodies with attributes and methods
  - Subscript parsing enhancement: Subscripts now handle comma-separated indices for type annotations (e.g., dict[str, int] creates a tuple index)
  - Parser test suite: 341 tests (336 in parser_tests.rs + 5 in compound_operators_test.rs) covering operators, postfix operations, collection literals, lambda expressions, conditional expressions, walrus operator, ellipsis, comprehensions (list/dict/set), generator expressions, assignment statements, exception handling, import statements, from...import statements, control flow (if/elif/else, while, for), function definitions with all parameter types including positional-only and keyword-only parameters, async function definitions, parameter type annotations, return type annotations, class definitions with inheritance and async methods
    - Variable annotations:
      * Simple annotations without assignment (x: int, name: str, flag: bool)
      * Annotations with values (x: int = 5, name: str = "hello")
      * Generic type annotations (items: list[int], data: dict[str, int] = {})
      * Nested generic types (matrix: list[list[int]])
      * Union types (value: int | str)
      * Optional types (value: Optional[int] = None)
      * Callable types (func: Callable)
      * Complex values with annotations (result: dict[str, int] = {"a": 1, "b": 2})
    - Decorators (basic):
      * Simple decorators (@decorator)
      * Decorators with calls (@decorator())
      * Decorators with arguments (@decorator(arg1, arg2))
      * Multiple stacked decorators (@decorator1 @decorator2 @decorator3)
      * Attribute access decorators (@pkg.decorator, @pkg.module.decorator)
      * Decorators on async functions (@decorator async def foo(): pass)
      * Decorators on classes (@decorator class Foo: pass)
      * Class decorators with inheritance (@decorator class Foo(Base): pass)
      * Class decorators with all combinations (inheritance, methods, blank lines)
      * Decorators preserve all function and class details (parameters, return types, inheritance, body)
    - Metaclass specification:
      * Simple metaclass (class Foo(metaclass=Meta): pass)
      * Metaclass with inheritance (class Foo(Base, metaclass=Meta): pass)
      * Metaclass with multiple bases (class Foo(Base1, Base2, metaclass=Meta): pass)
      * Metaclass with attribute access (class Foo(metaclass=pkg.Meta): pass)
      * Metaclass with call expression (class Foo(metaclass=type()): pass)
      * Trailing comma support (class Foo(Base, metaclass=Meta,): pass)
      * Error handling for invalid keyword arguments, duplicate metaclass, bases after metaclass
  - Lexer enhancement: Added @ (At) token for decorator syntax
  - Parser enhancement: Proper handling of blank lines between statements, functions, and classes (PEP 8 compliant spacing)
  - Comprehensive negative tests for edge cases: empty statements, invalid syntax, malformed inputs with clear error messages, parameter order violations, invalid class names, duplicate * or *args or / parameters, / after * validation, async without def validation
  - Syntax validation: Multiple starred expressions in unpacking now properly rejected as syntax error; parameter order strictly enforced; class name validation; positional-only and keyword-only parameter validation; async keyword must be followed by def
  - Code quality: Refactored parse_global and parse_nonlocal to use shared parse_name_list helper function (DRY principle)
  - Improved error messages: More specific "Expected at least one identifier" message when no identifiers provided after global/nonlocal; clear parameter order error messages; clear class definition error messages; clear async syntax error messages; clear metaclass specification error messages
  - **577 total tests (142 lexer + 427 parser + 8 other); 572 passing, 5 ignored**
- **Phase 2.7: Parser Error Handling (In Progress)**
  - Error message helper functions:
    * `error()` - Create formatted error with current position
    * `expected()` - Generate "Expected X, found Y" messages
    * `expected_after()` - Generate "Expected X after Y, found Z" messages
    * `current_token_string()` - Human-readable token descriptions (all TokenKind variants)
  - Applied helpers throughout parser:
    * `parse_function_def()` - function header errors
    * `expect_token()` - general token expectation
    * `parse_if_statement()` - if/elif/else colon errors (3 locations)
    * `parse_while_loop()` - while/else colon errors (2 locations)
    * `parse_for_loop()` - for/else colon errors (2 locations)
    * `parse_class_def()` - class header colon error
    * `consume_newline_or_eof()` - improved to show what was found
    * `parse_primary()` - expression parsing errors
    * Lambda and dict colons (via expect_token)
  - Validation logic added:
    * Assignment target validation - rejects literals, operators, function calls, lambdas as assignment targets
    * Detects invalid assignment targets like `5 = x` with clear error messages
    * Empty expression detection - `if :` now produces "Expected expression, found ':'" error
    * Parameter order validation already working (non-default after default rejected)
  - Improved error messages:
    * Clear "Expected ':' after X" messages for all control structures
    * Better error messages for missing function names after 'def'
    * Consistent "Expected X, found Y" format with readable token descriptions
    * "Cannot assign to literal/operator/function call" messages
    * "Expected expression" when expression is missing
    * Position information in all error messages
  - Error message test suite:
    * 16 tests covering various error scenarios
    * All 16 tests passing (improved error messages and validation working)
    * Test helper enhanced to catch both lexer and parser errors
  - Error suggestions for common mistakes:
    * `error_with_suggestion()` - Append helpful hints to error messages
    * `suggest_keyword_fix()` - Detects 11 common keyword typos:
      - `elseif`/`elsif` → "Did you mean 'elif'?"
      - `define`/`function`/`func` → "Did you mean 'def'?"
      - `cls` → "Did you mean 'class'?"
      - `then` → "Remove 'then' (not needed in Mamba syntax)"
      - `switch` → "Did you mean 'match'?"
      - `foreach` → "Did you mean 'for'?"
      - `until` → "Mamba uses 'while not' instead of 'until'"
      - `unless` → "Mamba uses 'if not' instead of 'unless'"
    * Context-aware suggestion triggering:
      - Checks identifier context before suggesting (avoids false positives)
      - Detects typos in statement-like contexts (followed by colon/identifier)
      - Suggestions integrated into `expected()` helper for broader coverage
    * Special handling for 'then' in if statements
    * Error suggestion test suite: 11 tests covering all keyword typo suggestions
  - Error recovery and multiple error tracking:
    * Parser now collects multiple errors instead of stopping at first error
    * `synchronize()` method skips to safe recovery points:
      - Newline (statement boundary)
      - Dedent (block boundary)
      - Statement keywords (def, class, if, while, for, return, import, etc.)
    * Panic mode prevents cascading errors
    * Parser continues after errors, parsing as much valid code as possible
    * Returns `Result<Module, Vec<MambaError>>` with all collected errors
    * Separate errors reported when separated by successfully parsed code
    * Cascading errors suppressed until successful parse
    * Error recovery test suite: 21 tests covering:
      - Single and multiple error recovery
      - Recovery across statement boundaries
      - Recovery in function and class definitions
      - Recovery from incomplete syntax
      - Validation that good code is preserved around errors
      - Prevention of cascading errors
      - Multiple distinct errors with valid code between them
  - **614 total tests (142 lexer + 464 parser + 8 other); all passing ✅**
- **Parser Benchmarking**
  - Parser benchmarks:
    * Comprehensive benchmark suite with 40+ benchmarks across 7 categories
    * Expression parsing benchmarks (literals, operators, collections, comprehensions)
    * Statement parsing benchmarks (assignments, control flow, imports)
    * Function definition benchmarks (all parameter types, decorators, async)
    * Class definition benchmarks (methods, inheritance, decorators)
    * Medium file benchmarks (realistic 20-40 line modules)
    * Large file stress tests (100+ statements)
    * Edge case benchmarks (long parameter lists, deep nesting)
    * Performance characteristics documented in BENCHMARKS.md
  - Parser throughput: ~1.4M assignments/sec, ~543k functions/sec
  - Parser overhead: ~300-500ns on top of lexer for simple constructs
  - Full lexer+parser performance profiled and documented
- Documentation: BENCHMARKS.md, FUZZING.md
- Test organization: All tests moved to separate files in tests/ directory

### Fixed
- Invalid digit validation for octal (0-7 only) and binary (0-1 only) literals
- Raw string quote escaping (r"\"" now handled correctly)
- EOF dedent emission (balanced INDENT/DEDENT tokens)
- Unicode combining marks handling (simplified test inputs)
