# Mamba

**Mamba** is a programming language with **full Python syntax compatibility** that **transpiles to Rust** and then compiles to native binaries.

> **Goal:** Write Python-like code, get Rust-level performance, with a single self-contained toolchain.

---

## What Mamba Is

- A **Python syntax clone**
- A **source-to-source transpiler**: Mamba → Rust → native binary
- A **compiled language**, not an interpreter
- Distributed as a **single installer** per platform (Windows, macOS, Linux)

!CRITICAL:
- Everything that needs to be installed, should be installed by the single installer.
  Users should not have to install anything themselves.

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

## Developing rules

!CRITICAL:
- Small increments every step. Do not make large changes in one go. 
  If a change is too big, break it down into smaller parts that can be done in separate steps.
- THOROUGLY test EVERY aspect you implemented. DO NOT just test happy paths, but also edge cases and failure cases.
- Do NOT create new `.md` files to summarize what changes you made.
- Do NOT add mock data. Only placeholders marked with TODO comments are allowed.

!IMPORTANT:
- Use Rust as the programming language.
- Follow standard Rust coding conventions.
- Document the code thoroughly with comments.
- Keep the codebase clean and well organized.
- Sort the code into different files and packages based on functionality.
- Mark all changes in `docs/CHANGELOG.md`.