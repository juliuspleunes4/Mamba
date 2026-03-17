//! # Mamba Transpiler
//!
//! This crate transpiles Mamba AST into equivalent Rust source code.

// TODO: Implement code generator
// TODO: Implement type mappings
// TODO: Implement transpilation logic

pub mod codegen;

pub use codegen::{CodeGenerator, CodegenError};
