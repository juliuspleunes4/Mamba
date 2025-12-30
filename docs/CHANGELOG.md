# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),  
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
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
