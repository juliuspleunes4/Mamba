//! # Mamba Transpiler
//!
//! This crate transpiles Mamba AST into equivalent Rust source code.

// TODO: Implement code generator
// TODO: Implement type mappings
// TODO: Implement transpilation logic

pub mod codegen;
pub mod type_mapping;

pub use codegen::{CodeGenerator, CodegenError};
pub use type_mapping::{IntWidth, RustType, TypeMapper, TypeMappingError};
