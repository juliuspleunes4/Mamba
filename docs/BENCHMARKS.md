# Mamba Performance Benchmarks

Comprehensive benchmark suite for the Mamba parser measuring both lexer (tokenization) and parser (AST construction) performance across different scenarios.

## Lexer Benchmarks

### Benchmark Categories

### Small Files
Realistic small Python-like code snippets:
- **Simple Assignment**: ~527 ns
- **Function Definition**: ~1.29 µs
- **Class Definition**: ~2.36 µs

### Token Types
Performance per token category:
- **Numbers** (various formats): ~738 ns
- **Strings** (raw, f-strings, multiline): ~1.09 µs
- **Keywords**: ~900 ns
- **Operators**: ~1.23 µs
- **Identifiers** (ASCII): ~1.07 µs
- **Unicode Identifiers**: ~1.01 µs

### Indentation Handling
Python's significant whitespace:
- **Simple Indent**: ~807 ns
- **Nested Indent** (3 levels): ~1.76 µs
- **Deep Nesting** (10 levels): ~3.09 µs

### Medium Files
Realistic module (~50 lines):
- **Calculator Module**: ~16.9 µs

### Large Files
Stress testing:
- **1000 Assignments**: ~288 µs
- **500 Functions**: ~333 µs

### Edge Cases
Performance under stress:
- **Long Identifier** (1000 chars): ~2.44 µs
- **Long String** (1500 chars): ~4.22 µs
- **Operators Without Spaces**: ~1.70 µs
- **Mixed Content**: ~3.03 µs

## Running Benchmarks

```bash
# Run all benchmarks (lexer + parser)
cargo bench --package mamba-parser

# Run only lexer benchmarks
cargo bench --package mamba-parser --bench lexer_benchmarks

# Run only parser benchmarks
cargo bench --package mamba-parser --bench parser_benchmarks

# Run specific benchmark group
cargo bench --package mamba-parser -- small_files

# Save baseline for comparison
cargo bench --package mamba-parser -- --save-baseline main

# Compare against baseline
cargo bench --package mamba-parser -- --baseline main
```

## Parser Benchmarks

### Expression Parsing
Performance for various expression types:
- **Simple Literal**: ~368 ns
- **Binary Expression**: ~1.23 µs (arithmetic with operator precedence)
- **Nested Expression**: ~2.71 µs (deeply parenthesized)
- **Function Call**: ~1.90 µs (with arguments)
- **Chained Access**: ~1.52 µs (attribute/subscript chains)
- **List Literal**: ~3.40 µs (10 elements)
- **Dict Literal**: ~3.78 µs (5 key-value pairs)
- **List Comprehension**: ~2.76 µs (with filter)
- **Lambda**: ~1.20 µs (lambda expression)

### Statement Parsing
Performance for statement types:
- **Assignment**: ~823 ns (simple variable assignment)
- **Multiple Assignment**: ~1.49 µs (chained x = y = z)
- **Tuple Unpacking**: ~2.19 µs (a, b, c = 1, 2, 3)
- **Augmented Assignment**: ~750 ns (+= style operators)
- **If Statement**: ~4.77 µs (if/elif/else chain)
- **While Loop**: ~2.40 µs (while with body)
- **For Loop**: ~2.11 µs (for loop with body)
- **Import Statement**: ~749 ns (module imports)
- **From Import**: ~930 ns (selective imports)

### Function Definitions
Performance for function parsing:
- **Simple Function**: ~1.58 µs (basic function, no parameters)
- **Function with Params**: ~2.45 µs (parameters with defaults)
- **All Parameter Types**: ~4.29 µs (positional-only, keyword-only, *args, **kwargs)
- **With Annotations**: ~2.57 µs (type-annotated parameters and return)
- **Async Function**: ~2.67 µs (async function definition)
- **Decorated Function**: ~2.25 µs (multiple decorators)

### Class Definitions
Performance for class parsing:
- **Simple Class**: ~672 ns (empty class)
- **Class with Methods**: ~6.44 µs (~30 lines with __init__ and methods)
- **With Inheritance**: ~4.81 µs (class inheriting from parent)
- **Decorated Class**: ~2.36 µs (class with decorators)

### Medium Files (Realistic Modules)
Real-world code patterns:
- **Calculator Module**: ~21.5 µs (~30 line class with methods)
- **Data Processor**: ~29.5 µs (~40 lines with type hints and comprehensions)

### Large Files (Stress Tests)
Performance under load:
- **100 Assignments**: ~72.2 µs (~722 ns per assignment)
- **50 Functions**: ~92.0 µs (~1.84 µs per function)
- **Deep Nesting**: ~9.03 µs (10 levels of nested if statements)
- **Complex Expressions**: ~236 µs (50 complex arithmetic expressions)

### Edge Cases
Challenging scenarios:
- **Long Parameter List**: ~10.3 µs (function with 50 parameters)
- **Long Argument List**: ~15.1 µs (call with 50 arguments)
- **Deeply Nested Collections**: ~3.46 µs (10-level nested lists)
- **Complex Comprehension**: ~4.19 µs (nested comprehension with filter)

### Parser Performance Characteristics

1. **Linear Scaling**: Parser performance scales linearly with code complexity
   - 100 assignments: ~72 µs (~722 ns each)
   - 50 functions: ~92 µs (~1.84 µs each)
   - Consistent with individual benchmark results

2. **Expression vs Statement Cost**:
   - Simple expression (literal): ~368 ns
   - Simple statement (assignment): ~823 ns (2.2x)
   - Statement overhead includes expression parsing + AST node creation

3. **Collection Costs** (ranked):
   - List literal (10 items): ~3.40 µs
   - Dict literal (5 pairs): ~3.78 µs
   - Deeply nested collections: ~3.46 µs
   - Collection initialization overhead is consistent

4. **Control Flow Overhead**:
   - If statement (with elif/else): ~4.77 µs
   - While loop: ~2.40 µs
   - For loop: ~2.11 µs
   - For loops are most efficient for iteration

5. **Function Definition Costs**:
   - Simple (no params): ~1.58 µs
   - With parameters + defaults: ~2.45 µs
   - All parameter types: ~4.29 µs
   - Parameter complexity adds ~2.7 µs overhead

6. **Parser Throughput** (large files):
   - 50 functions: ~92 µs → ~543k functions/second
   - 100 assignments: ~72 µs → ~1.4M assignments/second
   - Complex expressions (50): ~236 µs → ~212k expressions/second

### Lexer + Parser Combined Cost

Comparing lexer-only vs full parsing (lex + parse):
- **Simple assignment**:
  * Lexer: ~527 ns
  * Parser: ~823 ns
  * Parser overhead: ~296 ns (56% additional)
  
- **Function definition**:
  * Lexer: ~1.29 µs
  * Simple parser: ~1.58 µs
  * Parser overhead: ~290 ns (22% additional)

**Key insight**: Parser adds relatively small overhead (~300-500ns) for simple constructs, showing efficient AST construction.

## Performance Characteristics (Lexer)

### Key Observations

1. **Linear Scaling**: Performance scales linearly with input size
   - 1000 lines: ~288 µs (~288 ns per line)
   - Consistent with small file benchmarks

2. **Unicode Performance**: Unicode identifiers perform **slightly better** than ASCII
   - Unicode: ~1.01 µs
   - ASCII: ~1.07 µs
   - Rust's `is_alphabetic()` is highly optimized

3. **Indentation Overhead**: Minimal cost for indentation tracking
   - Simple code: ~527 ns
   - Simple code with indentation: ~807 ns
   - Overhead: ~280 ns (~53%)

4. **Token Type Costs** (ranked):
   - Numbers: ~738 ns (cheapest)
   - Keywords: ~900 ns
   - Unicode identifiers: ~1.01 µs
   - Identifiers: ~1.07 µs
   - Strings: ~1.09 µs
   - Operators: ~1.23 µs (most expensive)

5. **Deep Nesting**: Efficient handling of nested structures
   - 10-level nesting: ~3.09 µs
   - ~309 ns per level
   - No exponential growth

### Throughput Estimates

Based on large file benchmarks:
- **~3,500 lines/ms** (1000 lines in 288 µs)
- **~3.5M lines/second**
- **~280 MB/second** (assuming 80 chars/line)

## Optimization Opportunities

1. **String Tokenization**: Most expensive token type
   - Consider lazy string parsing
   - Optimize escape sequence handling

2. **Operator Parsing**: Second most expensive
   - Lookahead optimization
   - Trie-based operator matching

3. **Memory Allocation**: Benchmark doesn't measure allocations
   - Profile with `cargo bench -- --profile-time=5`
   - Consider arena allocators for tokens

## Comparison with Other Lexers

*(To be added: Compare with Python's tokenizer, other language lexers)*

## Regression Testing

Run benchmarks before each release:
```bash
# Save baseline
cargo bench --package mamba-parser -- --save-baseline v0.1.0

# After changes, compare
cargo bench --package mamba-parser -- --baseline v0.1.0
```

Criterion will show performance changes as percentages.

## Notes

- All benchmarks use `black_box()` to prevent compiler optimizations
- Results from release builds (`--release`)
- Platform: Windows (results may vary on other platforms)
- Criterion automatically detects outliers and statistical noise
- Benchmarks include error handling overhead (realistic usage)
