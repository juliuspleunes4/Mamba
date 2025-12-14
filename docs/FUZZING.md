# Fuzzing Tests

Comprehensive fuzz testing suite for the Mamba lexer using cargo-fuzz (libFuzzer).

## Platform Support

⚠️ **Note**: Fuzzing with sanitizers works best on **Linux and macOS**. On Windows, use WSL (Windows Subsystem for Linux) for full fuzzing support.

### Windows Users
```bash
# Option 1: Use WSL (recommended)
wsl
cd /mnt/c/Users/Gebruiker/Desktop/projects/Mamba/crates/parser
rustup default nightly
cargo fuzz run fuzz_lexer

# Option 2: Build without sanitizers (limited testing)
cargo build --manifest-path=fuzz/Cargo.toml --bin fuzz_lexer
# Run manually with test inputs
```

### Linux/macOS Users
```bash
rustup default nightly
cd crates/parser
cargo fuzz run fuzz_lexer
```

## Setup

Fuzzing requires nightly Rust:

```bash
# Install cargo-fuzz (already done)
cargo install cargo-fuzz

# Switch to nightly (required for fuzzing)
rustup default nightly
```

## Fuzz Targets

### 1. `fuzz_lexer` - General Lexer Fuzzing
Tests the lexer with completely random inputs.

```bash
cd crates/parser
cargo fuzz run fuzz_lexer
```

**What it tests:**
- Random byte sequences
- Invalid UTF-8
- Malformed syntax
- Unexpected character combinations
- Buffer boundaries

### 2. `fuzz_numbers` - Number Literal Fuzzing
Focuses on number-like inputs to stress test numeric parsing.

```bash
cargo fuzz run fuzz_numbers
```

**What it tests:**
- Invalid number formats
- Extreme exponents
- Mixed bases (hex, octal, binary)
- Multiple decimal points
- Invalid digit characters
- Edge cases like `0x`, `0b`, `0o` without digits

### 3. `fuzz_strings` - String Literal Fuzzing
Focuses on string-like inputs to test string tokenization.

```bash
cargo fuzz run fuzz_strings
```

**What it tests:**
- Unclosed strings
- Invalid escape sequences
- Mixed quote types
- Raw and f-string edge cases
- Unicode in strings
- Very long strings

### 4. `fuzz_identifiers` - Identifier Fuzzing
Focuses on identifier-like inputs including Unicode.

```bash
cargo fuzz run fuzz_identifiers
```

**What it tests:**
- Unicode identifiers
- Invalid identifier starts (digits)
- Very long identifiers
- Mixed scripts
- Special Unicode characters
- Emoji and other non-letter characters

## Running Fuzzing

### Quick Test (10 seconds)
```bash
cd crates/parser
cargo fuzz run fuzz_lexer -- -max_total_time=10
```

### Run All Targets
```bash
# Run each target for 1 minute
cargo fuzz run fuzz_lexer -- -max_total_time=60
cargo fuzz run fuzz_numbers -- -max_total_time=60
cargo fuzz run fuzz_strings -- -max_total_time=60
cargo fuzz run fuzz_identifiers -- -max_total_time=60
```

### Continuous Fuzzing
```bash
# Run indefinitely (Ctrl+C to stop)
cargo fuzz run fuzz_lexer
```

### With More Workers
```bash
# Use all CPU cores
cargo fuzz run fuzz_lexer -- -workers=$(nproc)
```

## Corpus Management

### Seed Corpus
Initial test cases located in `corpus/<target>/`:
- Helps fuzzer learn patterns faster
- Provides baseline coverage
- **Committed to git** (`.txt` files only)
- Fuzzer-generated files are gitignored

Our seed inputs:
- `corpus/fuzz_lexer/simple.txt` - Basic assignment
- `corpus/fuzz_lexer/function.txt` - Function definition
- `corpus/fuzz_numbers/numbers.txt` - Various number formats
- `corpus/fuzz_strings/strings.txt` - String literals
- `corpus/fuzz_identifiers/identifiers.txt` - Unicode identifiers

### Add to Corpus
```bash
# Add a specific test case (will be committed to git)
echo "your_test_case" > corpus/fuzz_lexer/my_seed.txt

# Fuzzer will generate additional corpus files (gitignored)
```

### Minimize Corpus
```bash
# Remove redundant test cases
cargo fuzz cmin fuzz_lexer
```

## Crash Analysis

When fuzzer finds a crash:

```bash
# Crashes saved to: fuzz/artifacts/fuzz_<target>/
# Each crash includes the exact input that caused it

# Reproduce a crash
cargo fuzz run fuzz_lexer fuzz/artifacts/fuzz_lexer/crash-<hash>

# Get crash details
cat fuzz/artifacts/fuzz_lexer/crash-<hash> | xxd
```

## Coverage

Check code coverage from fuzzing:

```bash
# Generate coverage report
cargo fuzz coverage fuzz_lexer

# View coverage
cargo cov -- show target/coverage/<binary> \
    --format=html > coverage.html
```

## Expected Behavior

The lexer should **NEVER**:
- ❌ Panic
- ❌ Crash
- ❌ Infinite loop
- ❌ Allocate unbounded memory
- ❌ Access out of bounds

The lexer **MAY**:
- ✅ Return errors for invalid input
- ✅ Report syntax errors
- ✅ Reject malformed literals
- ✅ Handle any UTF-8 input gracefully

## Continuous Integration

### GitHub Actions Example
```yaml
- name: Run Fuzzing Tests
  run: |
    rustup default nightly
    cd crates/parser
    cargo fuzz run fuzz_lexer -- -max_total_time=300
    cargo fuzz run fuzz_numbers -- -max_total_time=300
    cargo fuzz run fuzz_strings -- -max_total_time=300
    cargo fuzz run fuzz_identifiers -- -max_total_time=300
```

## Troubleshooting

### "cargo fuzz command not found"
```bash
cargo install cargo-fuzz
```

### "requires nightly"
```bash
rustup default nightly
```

### Very slow on Windows
Fuzzing is slower on Windows. Consider:
- Running on Linux/WSL for better performance
- Using fewer workers
- Running for shorter durations

### Out of memory
```bash
# Limit memory usage (in MB)
cargo fuzz run fuzz_lexer -- -rss_limit_mb=2048
```

## Results & Statistics

After fuzzing, check statistics:

```bash
# View fuzzer stats
cat fuzz/stats.txt

# Key metrics:
# - exec/s: Executions per second (throughput)
# - cov: Coverage (unique code paths)
# - corpus: Number of interesting test cases
```

## Best Practices

1. **Run regularly**: Fuzz before each release
2. **Long runs**: Run overnight for deep testing
3. **Review crashes**: Every crash is a bug
4. **Expand corpus**: Add interesting edge cases
5. **Track coverage**: Aim for high code coverage

## Integration with Testing

Fuzzing complements other testing:
- **Unit tests**: Known edge cases
- **Property tests**: Logical invariants
- **Fuzzing**: Unknown edge cases and crashes

## Notes

- Fuzzing requires nightly Rust toolchain
- Uses libFuzzer under the hood
- Deterministic: Same seed = same sequence
- Coverage-guided: Explores new code paths
- Can run indefinitely (continuous testing)

## Further Reading

- [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Fuzzing best practices](https://rust-fuzz.github.io/book/introduction.html)
