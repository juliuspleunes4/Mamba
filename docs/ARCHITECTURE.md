# Mamba Architecture

This document describes the high-level architecture of Mamba, a Python-syntax language that transpiles to Rust and compiles to native binaries.

---

## Overview

Mamba is a **source-to-source transpiler** with a simple, linear pipeline:

```
Mamba Source (.mmb) → Parser → Transpiler → Rust Source → Rust Compiler → Native Binary
```

The user interacts only with the Mamba CLI. Rust toolchain operations are completely abstracted away.

---

## Core Components

### 1. CLI (`mamba`)

The main entry point for all user interactions.

**Responsibilities:**
- Parse command-line arguments
- Orchestrate the compilation pipeline
- Wrap and format all output (errors, warnings, diagnostics)
- Manage bundled Rust toolchain invocation
- Handle file I/O and temporary build artifacts

**Key Commands:**
- `mamba <file>` - Compile a Mamba file
- `mamba br <file>` - Compile and run a Mamba file
- Future: package manager, formatter, linter

---

### 2. Parser

Parses Mamba source code (Python syntax) into an Abstract Syntax Tree (AST).

**Responsibilities:**
- Lexical analysis (tokenization)
- Syntax analysis (AST construction)
- Syntax error reporting with line/column information
- Full Python syntax support (variables, functions, control flow, data structures)

**Input:** `.mmb` source files  
**Output:** AST representation

**Implementation Notes:**
- Must support 100% Python syntax
- Error messages should be clear and actionable
- Line/column tracking for diagnostics

---

### 3. Transpiler

Converts the Mamba AST into equivalent Rust source code.

**Responsibilities:**
- AST traversal and transformation
- Map Python constructs to Rust equivalents
- Type inference and annotation generation
- Generate idiomatic Rust code
- Handle semantic differences between Python and Rust

**Input:** Mamba AST  
**Output:** Rust source code (`.rs`)

**Key Challenges:**
- Python's dynamic typing → Rust's static typing
- Python's memory model → Rust's ownership model
- Python data structures → Rust equivalents
- Error propagation patterns

**Translation Examples:**
```python
# Mamba
x = 5
print(x)
```
```rust
// Generated Rust
fn main() {
    let x: i32 = 5;
    println!("{}", x);
}
```

---

### 4. Rust Compiler Interface

Manages invocation of the bundled Rust toolchain.

**Responsibilities:**
- Invoke `rustc` with appropriate flags
- Capture compiler output (stdout/stderr)
- Parse Rust compiler diagnostics
- Reformat Rust errors as Mamba errors
- Manage temporary build directories

**Key Features:**
- Completely hidden from users
- All Rust errors are wrapped and re-rendered in Mamba context
- Uses embedded Rust compiler (no system dependency)

---

### 5. Standard Library

Mamba's built-in functionality.

**Responsibilities:**
- Core data types (int, float, string, list, dict, etc.)
- Built-in functions (print, len, range, etc.)
- Future: File I/O, networking, concurrency

**Implementation:**
- Written in Rust
- Exposed via transpiler-generated imports
- Designed for performance and safety

---

## Compilation Pipeline

### Step-by-Step Flow

1. **User Input**
   ```
   $ mamba program.mmb
   ```

2. **CLI Initialization**
   - Parse arguments
   - Locate source file
   - Initialize bundled Rust toolchain

3. **Parsing Phase**
   - Read `.mmb` file
   - Tokenize source
   - Build AST
   - Report syntax errors (if any)

4. **Transpilation Phase**
   - Walk AST
   - Generate Rust code
   - Write temporary `.rs` file

5. **Compilation Phase**
   - Invoke `rustc` on generated Rust code
   - Capture compiler output
   - Parse and reformat errors

6. **Output**
   - On success: native executable
   - On failure: clear error messages in Mamba context

---

## Error Handling Strategy

### Principle: All Errors Are Mamba Errors

Users should never see raw Rust compiler output or stack traces.

**Error Categories:**

1. **Syntax Errors** (from Parser)
   - Line/column information
   - Clear explanation
   - Example: `SyntaxError: unexpected token 'def' at line 5, column 10`

2. **Transpilation Errors** (from Transpiler)
   - Semantic issues during Rust generation
   - Example: `TypeError: unsupported operation for type 'str'`

3. **Compilation Errors** (from Rust Compiler)
   - Wrapped and translated to Mamba context
   - Reference original `.mmb` file, not generated `.rs`
   - Example: `CompileError: variable 'x' not found in scope (line 12)`

---

## Bundled Toolchain

Mamba ships with an embedded Rust compiler and standard library (sysroot).

**Components:**
- `rustc` - The Rust compiler
- Standard library (core, std, alloc)
- Platform-specific linker configuration

**Platform Support:**
- Windows: MSVC toolchain
- macOS: XCode command-line tools integration
- Linux: GCC/Clang integration

**Benefits:**
- Zero external dependencies
- Consistent behavior across systems
- No user-facing Rust installation required

---

## Package Management (Future)

Planned architecture for Mamba's package ecosystem.

**Components:**
- **Package Registry** - Central repository (similar to crates.io)
- **Package Manager** - CLI tool for installing/managing packages
- **Package Format** - Native, pre-compiled Rust-backed libraries

**Design Goals:**
- Fast, native performance
- Static linking (no runtime dependencies)
- Secure by default
- Versioned and reproducible builds

---

## Directory Structure

```
mamba/
├── src/
│   ├── cli/          # Command-line interface
│   ├── parser/       # Lexer, parser, AST
│   ├── transpiler/   # AST → Rust code generation
│   ├── compiler/     # Rust compiler interface
│   ├── stdlib/       # Standard library implementation
│   ├── error/        # Error handling and formatting
│   └── main.rs       # Entry point
├── docs/             # Documentation
├── tests/            # Test suite
└── bundled/          # Embedded Rust toolchain (per platform)
```

---

## Design Principles

1. **Simplicity** - One tool, one command, one workflow
2. **Transparency** - Clear errors, predictable behavior
3. **Performance** - Native binaries, zero overhead
4. **Safety** - Leverage Rust's safety guarantees
5. **Familiarity** - Python syntax, no learning curve for syntax

---

## Future Enhancements

- **Language Server Protocol (LSP)** - IDE integration (autocomplete, diagnostics)
- **Formatter** - Opinionated code formatting
- **Linter** - Static analysis and best practices
- **Debugger** - Native debugging support
- **REPL** - Interactive mode for testing
- **Cross-compilation** - Build for other platforms

---

## Summary

Mamba's architecture is designed around a single goal: **make it easy to write Python-style code and ship fast, native executables**.

The pipeline is simple, the tooling is unified, and the user experience is seamless. Rust is the engine, but the user never sees it.

**Parse → Transpile → Compile → Run.**
