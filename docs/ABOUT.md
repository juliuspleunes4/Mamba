
# Mamba

**Mamba** is a programming language with **full Python syntax compatibility** that **transpiles to Rust** and then compiles to native binaries.

> **Goal:** Write Python-like code, get Rust-level performance, with a single self-contained toolchain.

---

## What Mamba Is

- A **Python syntax clone**
- A **source-to-source transpiler**: Mamba → Rust → native binary
- A **compiled language**, not an interpreter
- Distributed as a **single installer** per platform (Windows, macOS, Linux)

---

## Quick Start

Users install Mamba once and can run:

```bash
mamba program.mmb
```

No Python. No Rust. No extra setup.

---

## Design Goals

- **100% Python syntax**
	- for / while
	- variables
	- functions
	- conditionals
	- basic data structures
	- familiar Python control flow
- **Native performance**
	- Rust backend
	- Optimized binaries
	- Zero external dependencies for users
	- Bundled Rust toolchain
	- No system Python or Rust required
- **Consistent developer experience**
	- All output, errors, and diagnostics come from Mamba
	- Rust compiler output is fully wrapped and re-rendered

---

## How It Works

1. Parse Mamba (Python-syntax) source code
2. Transpile to equivalent Rust source code
3. Compile using a bundled Rust toolchain
4. Output a native executable

> Rust is never exposed to the user.

---

## Packages & Ecosystem

- **Python packages are not supported**
	- `pip`, `numpy`, etc. will **not** work
	- This is intentional

**Planned:**

- A custom Mamba package manager
- Native, Rust-backed packages designed for:
	- performance
	- safety
	- static compilation
- A growing standard library beyond basic functionality

---

## Platform Support

- Windows
- macOS
- Linux

Each platform ships with:

- mamba CLI
- Embedded Rust compiler + sysroot
- Automatic PATH integration via installer

---

## Non-Goals

- CPython compatibility
- Dynamic runtime execution
- Supporting the entire Python ecosystem
- Being a drop-in replacement for Python

> Mamba is Python syntax, not Python semantics or runtime.

---

## Why “Mamba”

- A fast snake
- Familiar to Python developers
- Symbolizes speed + precision
- Matches the language philosophy: Python feel, Rust power

---

## Status

Mamba is under active development.

**Early focus:**

- Syntax coverage
- Correct transpilation
- Clean diagnostics
- Stable compilation pipeline

**Long-term:**

- Package manager
- Extended standard library
- Tooling (formatter, linter, language server, VS Code extension)

---

## Summary

Mamba lets you write Python-style code and ship fast, native executables — without Python, without Rust, and without setup.

**Python syntax. Rust speed. One tool.**