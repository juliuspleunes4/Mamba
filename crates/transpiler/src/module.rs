use std::collections::HashSet;

use mamba_parser::ast::{Expression, Module, Statement};
use thiserror::Error;

use crate::codegen::{CodeGenerator, CodegenError};
use crate::statement::{StatementTranspileError, StatementTranspiler};

#[derive(Debug, Error)]
pub enum ModuleTranspileError {
    #[error("statement transpilation failed")]
    Statement(#[from] StatementTranspileError),
    #[error("code generation failed")]
    Codegen(#[from] CodegenError),
}

/// Transpiles a full Mamba module into Rust source with a concrete entry point.
#[derive(Debug, Clone, Default)]
pub struct ModuleTranspiler {
    statements: StatementTranspiler,
}

impl ModuleTranspiler {
    pub fn new() -> Self {
        Self {
            statements: StatementTranspiler::new(),
        }
    }

    pub fn transpile_module(&self, module: &Module) -> Result<String, ModuleTranspileError> {
        let mut generator = CodeGenerator::new();

        let mut main_body = Vec::new();
        for statement in &module.statements {
            if matches!(statement, Statement::FunctionDef { .. }) {
                self.statements.transpile_into(&mut generator, statement)?;
                generator.emit_empty_line();
            } else {
                main_body.push(statement);
            }
        }

        generator.open_block("fn main()");
        self.emit_main_body(&mut generator, &main_body)?;
        generator.close_block()?;

        Ok(generator.into_string())
    }

    fn emit_main_body(
        &self,
        generator: &mut CodeGenerator,
        statements: &[&Statement],
    ) -> Result<(), ModuleTranspileError> {
        let mut declared_variables = HashSet::new();

        for statement in statements {
            match statement {
                Statement::Assignment { targets, value, .. } => {
                    if let Some(name) = extract_single_identifier_target(targets) {
                        if declared_variables.insert(name.clone()) {
                            let decl = self
                                .statements
                                .transpile_variable_declaration(&name, None, Some(value))?;
                            generator.emit(&decl);
                            continue;
                        }
                    }
                    self.statements.transpile_into(generator, statement)?;
                }
                Statement::AnnAssignment { target, .. } => {
                    declared_variables.insert(target.clone());
                    self.statements.transpile_into(generator, statement)?;
                }
                _ => {
                    self.statements.transpile_into(generator, statement)?;
                }
            }
        }

        Ok(())
    }
}

fn extract_single_identifier_target(targets: &[Expression]) -> Option<String> {
    if targets.len() != 1 {
        return None;
    }

    match &targets[0] {
        Expression::Identifier { name, .. } => Some(name.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::ModuleTranspiler;
    use mamba_parser::ast::{Expression, Literal, Module, Statement};
    use mamba_parser::token::SourcePosition;

    fn pos() -> SourcePosition {
        SourcePosition::new(1, 1, 0)
    }

    fn ident(name: &str) -> Expression {
        Expression::Identifier {
            name: name.to_string(),
            position: pos(),
        }
    }

    fn int_lit(value: i64) -> Expression {
        Expression::Literal(Literal::Integer {
            value,
            position: pos(),
        })
    }

    #[test]
    fn wraps_top_level_code_in_main() {
        let transpiler = ModuleTranspiler::new();
        let module = Module {
            statements: vec![Statement::Expression(Expression::Call {
                function: Box::new(ident("print")),
                arguments: vec![int_lit(1)],
                position: pos(),
            })],
            position: pos(),
        };

        assert_eq!(
            transpiler.transpile_module(&module).unwrap(),
            r#"fn main() {
    println!("{}", vec![format!("{:?}", 1)].join(" "));
}
"#
        );
    }

    #[test]
    fn emits_function_definitions_outside_main() {
        let transpiler = ModuleTranspiler::new();
        let function = Statement::FunctionDef {
            name: "add1".to_string(),
            parameters: vec![mamba_parser::ast::Parameter {
                name: "x".to_string(),
                kind: mamba_parser::ast::ParameterKind::Regular,
                default: None,
                type_annotation: Some(ident("int")),
                position: pos(),
            }],
            body: vec![Statement::Return {
                value: Some(Expression::BinaryOp {
                    left: Box::new(ident("x")),
                    op: mamba_parser::ast::BinaryOperator::Add,
                    right: Box::new(int_lit(1)),
                    position: pos(),
                }),
                position: pos(),
            }],
            is_async: false,
            return_type: Some(ident("int")),
            decorators: vec![],
            position: pos(),
        };

        let module = Module {
            statements: vec![
                function,
                Statement::Expression(Expression::Call {
                    function: Box::new(ident("add1")),
                    arguments: vec![int_lit(5)],
                    position: pos(),
                }),
            ],
            position: pos(),
        };

        assert_eq!(
            transpiler.transpile_module(&module).unwrap(),
            "fn add1(x: i64) -> i64 {\n    return (x + 1);\n}\n\nfn main() {\n    add1(5);\n}\n"
        );
    }

    #[test]
    fn declares_top_level_identifier_assignments_once() {
        let transpiler = ModuleTranspiler::new();
        let module = Module {
            statements: vec![
                Statement::Assignment {
                    targets: vec![ident("x")],
                    value: int_lit(1),
                    position: pos(),
                },
                Statement::Assignment {
                    targets: vec![ident("x")],
                    value: int_lit(2),
                    position: pos(),
                },
            ],
            position: pos(),
        };

        assert_eq!(
            transpiler.transpile_module(&module).unwrap(),
            "fn main() {\n    let mut x = 1;\n    x = 2;\n}\n"
        );
    }

    #[test]
    fn always_generates_main_entrypoint() {
        let transpiler = ModuleTranspiler::new();
        let module = Module {
            statements: vec![],
            position: pos(),
        };

        assert_eq!(transpiler.transpile_module(&module).unwrap(), "fn main() {\n}\n");
    }
}
