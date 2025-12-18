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
- **Simple Literal**: Basic literal parsing
- **Binary Expression**: Arithmetic with operator precedence
- **Nested Expression**: Deeply parenthesized expressions
- **Function Call**: Function calls with arguments
- **Chained Access**: Attribute/subscript chains
- **List Literal**: List with 10 elements
- **Dict Literal**: Dict with 5 key-value pairs
- **List Comprehension**: Comprehension with filter
- **Lambda**: Lambda expression parsing

### Statement Parsing
Performance for statement types:
- **Assignment**: Simple variable assignment
- **Multiple Assignment**: Chained assignments (x = y = z)
- **Tuple Unpacking**: Multiple target unpacking
- **Augmented Assignment**: += style operators
- **If Statement**: If/elif/else chain
- **While Loop**: While with body
- **For Loop**: For loop with body
- **Import Statement**: Module imports
- **From Import**: Selective imports

### Function Definitions
Performance for function parsing:
- **Simple Function**: Basic function with no parameters
- **Function with Params**: Parameters with defaults
- **All Parameter Types**: Positional-only, keyword-only, *args, **kwargs
- **With Annotations**: Type-annotated parameters and return
- **Async Function**: Async function definition
- **Decorated Function**: Multiple decorators

### Class Definitions
Performance for class parsing:
- **Simple Class**: Empty class
- **Class with Methods**: Class with __init__ and methods
- **With Inheritance**: Class inheriting from parent
- **Decorated Class**: Class with decorators

### Medium Files (Realistic Modules)
Real-world code patterns:
- **Calculator Module**: ~30 line class with methods
- **Data Processor**: Functions with type hints and comprehensions

### Large Files (Stress Tests)
Performance under load:
- **100 Assignments**: Sequential assignments
- **50 Functions**: Multiple function definitions
- **Deep Nesting**: 10 levels of nested if statements
- **Complex Expressions**: 50 complex arithmetic expressions

### Edge Cases
Challenging scenarios:
- **Long Parameter List**: Function with 50 parameters
- **Long Argument List**: Call with 50 arguments
- **Deeply Nested Collections**: 10-level nested lists
- **Complex Comprehension**: Nested comprehension with filter

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
