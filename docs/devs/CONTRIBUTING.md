# Contributing to Mamba

Thank you for your interest in contributing to Mamba! This document provides guidelines and instructions for contributing to the project.

---

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates.

**When submitting a bug report, include:**

- A clear and descriptive title
- Steps to reproduce the issue
- Expected behavior vs. actual behavior
- Mamba version and platform (Windows, macOS, Linux)
- Sample code that demonstrates the issue
- Any error messages or output

### Suggesting Features

Feature suggestions are welcome! Please:

- Check if the feature has already been suggested
- Provide a clear description of the feature
- Explain why it would be useful to the Mamba community
- Consider how it aligns with Mamba's design goals

### Contributing Code

1. **Fork the repository** and create a branch from `main`
2. **Make your changes** following the coding standards below
3. **Test your changes** thoroughly
4. **Commit your changes** with clear, descriptive messages
5. **Submit a pull request** with a detailed description

---

## Development Setup

### Prerequisites

- Rust toolchain (latest stable)
- Git
- A code editor (VS Code recommended)

### Getting Started

```bash
# Clone your fork
git clone https://github.com/juliuspleunes4/mamba.git
cd mamba

# Build the project
cargo build

# Run tests
cargo test

# Run Mamba
cargo run -- program.mmb
```

---

## Coding Standards

### Rust Code Style

- Follow standard Rust conventions
- Use `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Write meaningful comments for complex logic
- Document public APIs with doc comments

### Code Organization

- Keep files focused and modular
- Group related functionality into separate modules
- Use descriptive names for functions and variables
- Avoid deep nesting where possible

### Example

```rust
// Good: Clear, documented, idiomatic
/// Parses a Mamba source file into an AST.
pub fn parse_file(source: &str) -> Result<Ast, ParseError> {
    let tokens = tokenize(source)?;
    build_ast(tokens)
}

// Avoid: Unclear, no documentation
pub fn pf(s: &str) -> Result<Ast, ParseError> { /* ... */ }
```

---

## Commit Messages

Follow these conventions:

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Prefix with type: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`
- Keep the first line under 72 characters
- Reference issues when applicable

**Examples:**

```
feat: add support for list comprehensions
fix: resolve parser error with nested functions
docs: update ARCHITECTURE.md with error handling details
refactor: simplify transpiler type inference logic
test: add integration tests for control flow
```

---

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Writing Tests

- Add tests for all new functionality
- Include edge cases and error conditions
- Use descriptive test names
- Keep tests focused and independent

**Example:**

```rust
#[test]
fn test_parse_variable_declaration() {
    let source = "x = 5";
    let ast = parse(source).unwrap();
    assert_eq!(ast.statements.len(), 1);
}
```

---

## Pull Request Process

1. **Ensure all tests pass** before submitting
2. **Update documentation** if you change APIs or behavior
3. **Add an entry to `CHANGELOG.md`** under "Unreleased"
4. **Keep PRs focused** - one feature or fix per PR
5. **Respond to feedback** from maintainers promptly
6. **Be patient** - reviews may take time

### PR Description Template

```
## Description
Brief description of what this PR does

## Changes
- List of specific changes made
- Another change

## Testing
How the changes were tested

## Related Issues
Fixes #123
```

---

## Areas for Contribution

### High Priority

- **Parser improvements** - Expand Python syntax coverage
- **Transpiler features** - Better Rust code generation
- **Error messages** - Clearer, more helpful diagnostics
- **Standard library** - Built-in functions and data structures
- **Tests** - Increase coverage

### Future Work

- Language Server Protocol (LSP) implementation
- Code formatter
- Linter
- Package manager
- Cross-platform installer

---

## Getting Help

- Check existing [documentation](../../README.md)
- Browse [open issues](https://github.com/juliuspleunes4/mamba/issues)
- Ask questions in GitHub Discussions
- Review [ARCHITECTURE.md](../ARCHITECTURE.md) for technical details

---

## Code of Conduct

This project adheres to a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

---

## License

By contributing to Mamba, you agree that your contributions will be licensed under the Apache 2.0 License.

---

Thank you for contributing to Mamba! 🐍⚡
