# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),  
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
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
      - `then` → "Remove 'then' (not needed in Python syntax)"
      - `switch` → "Did you mean 'match'? (Python 3.10+ pattern matching)"
      - `foreach` → "Did you mean 'for'?"
      - `until` → "Python uses 'while not' instead of 'until'"
      - `unless` → "Python uses 'if not' instead of 'unless'"
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
- Documentation: BENCHMARKS.md, FUZZING.md
- Test organization: All tests moved to separate files in tests/ directory

### Fixed
- Invalid digit validation for octal (0-7 only) and binary (0-1 only) literals
- Raw string quote escaping (r"\"" now handled correctly)
- EOF dedent emission (balanced INDENT/DEDENT tokens)
- Unicode combining marks handling (simplified test inputs)
