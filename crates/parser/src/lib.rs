//! # Mamba Parser
//!
//! This crate provides lexing and parsing functionality for the Mamba programming language.
//! It converts Mamba source code (Python syntax) into an Abstract Syntax Tree (AST).

pub mod lexer;
pub mod token;
pub mod ast;
pub mod parser;
