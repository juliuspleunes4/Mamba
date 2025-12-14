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
  - Statement parsing:
    - Assignment statements (x = 5)
    - Augmented assignment (+=, -=, *=, /=, //=, %=, **=, &=, |=, ^=, >>=, <<=)
    - Expression statements
    - Pass, break, continue, return statements
  - Parser test suite: 114 tests covering operators, postfix operations, collection literals, lambda expressions, conditional expressions, walrus operator, ellipsis, list comprehensions
  - **261 total tests, all passing (142 lexer + 114 parser + 5 compound operator tests)**
- Documentation: BENCHMARKS.md, FUZZING.md
- Test organization: All tests moved to separate files in tests/ directory

### Fixed
- Invalid digit validation for octal (0-7 only) and binary (0-1 only) literals
- Raw string quote escaping (r"\"" now handled correctly)
- EOF dedent emission (balanced INDENT/DEDENT tokens)
- Unicode combining marks handling (simplified test inputs)
