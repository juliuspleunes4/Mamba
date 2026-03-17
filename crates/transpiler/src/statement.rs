use mamba_parser::ast::{AugmentedOperator, Expression, Statement};
use thiserror::Error;

use crate::codegen::{CodeGenerator, CodegenError};
use crate::expression::{ExpressionTranspileError, ExpressionTranspiler};
use crate::type_mapping::{TypeMapper, TypeMappingError};

#[derive(Debug, Error)]
pub enum StatementTranspileError {
    #[error("unsupported statement kind for phase 4.4")]
    UnsupportedStatement,
    #[error("assignment must include at least one target")]
    EmptyAssignmentTargets,
    #[error("multiple assignment targets are not supported yet")]
    MultipleAssignmentTargets,
    #[error("cannot transpile expression")]
    Expression(#[from] ExpressionTranspileError),
    #[error("cannot map type annotation")]
    TypeMapping(#[from] TypeMappingError),
    #[error("code generation failed")]
    Codegen(#[from] CodegenError),
}

/// Transpiles core statement forms for Phase 4.4.
#[derive(Debug, Clone, Default)]
pub struct StatementTranspiler {
    expr: ExpressionTranspiler,
    types: TypeMapper,
}

impl StatementTranspiler {
    pub fn new() -> Self {
        Self {
            expr: ExpressionTranspiler::new(),
            types: TypeMapper::new(),
        }
    }

    pub fn transpile_statement(
        &self,
        statement: &Statement,
    ) -> Result<String, StatementTranspileError> {
        let mut generator = CodeGenerator::new();
        self.transpile_into(&mut generator, statement)?;
        Ok(generator.into_string())
    }

    pub fn transpile_into(
        &self,
        generator: &mut CodeGenerator,
        statement: &Statement,
    ) -> Result<(), StatementTranspileError> {
        match statement {
            Statement::Expression(expr) => {
                let rendered = self.expr.transpile(expr)?;
                generator.emit_line(&format!("{};", rendered));
                Ok(())
            }
            Statement::Assignment { targets, value, .. } => {
                if targets.is_empty() {
                    return Err(StatementTranspileError::EmptyAssignmentTargets);
                }
                if targets.len() > 1 {
                    return Err(StatementTranspileError::MultipleAssignmentTargets);
                }

                let target = self.expr.transpile(&targets[0])?;
                let rendered_value = self.expr.transpile(value)?;
                generator.emit_line(&format!("{} = {};", target, rendered_value));
                Ok(())
            }
            Statement::AnnAssignment {
                target,
                annotation,
                value,
                ..
            } => {
                let rust_type = self.types.render_annotation(annotation)?;
                if let Some(init) = value {
                    let rendered_value = self.expr.transpile(init)?;
                    generator.emit_line(&format!("let mut {}: {} = {};", target, rust_type, rendered_value));
                } else {
                    generator.emit_line(&format!("let mut {}: {};", target, rust_type));
                }
                Ok(())
            }
            Statement::AugmentedAssignment {
                target, op, value, ..
            } => {
                let rendered_target = self.expr.transpile(target)?;
                let rendered_value = self.expr.transpile(value)?;
                match op {
                    AugmentedOperator::Power => {
                        generator.emit_line(&format!(
                            "{} = {}.pow({});",
                            rendered_target, rendered_target, rendered_value
                        ));
                    }
                    _ => {
                        let rendered_op = augmented_operator_to_rust(op);
                        generator.emit_line(&format!(
                            "{} {}= {};",
                            rendered_target, rendered_op, rendered_value
                        ));
                    }
                }
                Ok(())
            }
            Statement::Return { value, .. } => {
                if let Some(return_value) = value {
                    let rendered = self.expr.transpile(return_value)?;
                    generator.emit_line(&format!("return {};", rendered));
                } else {
                    generator.emit_line("return;");
                }
                Ok(())
            }
            Statement::Pass(_) => Ok(()),
            Statement::Break(_) => {
                generator.emit_line("break;");
                Ok(())
            }
            Statement::Continue(_) => {
                generator.emit_line("continue;");
                Ok(())
            }
            _ => Err(StatementTranspileError::UnsupportedStatement),
        }
    }

    pub fn transpile_variable_declaration(
        &self,
        name: &str,
        annotation: Option<&Expression>,
        initializer: Option<&Expression>,
    ) -> Result<String, StatementTranspileError> {
        let mut generator = CodeGenerator::new();
        match (annotation, initializer) {
            (Some(ann), Some(init)) => {
                generator.emit_line(&format!(
                    "let mut {}: {} = {};",
                    name,
                    self.types.render_annotation(ann)?,
                    self.expr.transpile(init)?
                ));
            }
            (Some(ann), None) => {
                generator.emit_line(&format!("let mut {}: {};", name, self.types.render_annotation(ann)?));
            }
            (None, Some(init)) => {
                generator.emit_line(&format!("let mut {} = {};", name, self.expr.transpile(init)?));
            }
            (None, None) => {
                generator.emit_line(&format!("let mut {} = ();", name));
            }
        }

        Ok(generator.into_string())
    }
}

fn augmented_operator_to_rust(op: &AugmentedOperator) -> &'static str {
    match op {
        AugmentedOperator::Add => "+",
        AugmentedOperator::Subtract => "-",
        AugmentedOperator::Multiply => "*",
        AugmentedOperator::Divide => "/",
        AugmentedOperator::FloorDivide => "/",
        AugmentedOperator::Modulo => "%",
        AugmentedOperator::Power => panic!("power is handled separately"),
        AugmentedOperator::BitwiseAnd => "&",
        AugmentedOperator::BitwiseOr => "|",
        AugmentedOperator::BitwiseXor => "^",
        AugmentedOperator::LeftShift => "<<",
        AugmentedOperator::RightShift => ">>",
    }
}

#[cfg(test)]
mod tests {
    use super::{StatementTranspileError, StatementTranspiler};
    use mamba_parser::ast::{
        AugmentedOperator, Expression, Literal, Statement,
    };
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
    fn transpiles_variable_declaration_helper() {
        let st = StatementTranspiler::new();
        let text = st
            .transpile_variable_declaration("count", Some(&ident("int")), Some(&int_lit(10)))
            .expect("declaration should transpile");

        assert_eq!(text, "let mut count: i64 = 10;\n");
    }

    #[test]
    fn transpiles_annotated_assignment() {
        let st = StatementTranspiler::new();
        let stmt = Statement::AnnAssignment {
            target: "name".to_string(),
            annotation: ident("str"),
            value: Some(Expression::Literal(Literal::String {
                value: "mamba".to_string(),
                position: pos(),
            })),
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "let mut name: String = \"mamba\";\n"
        );
    }

    #[test]
    fn transpiles_assignment() {
        let st = StatementTranspiler::new();
        let stmt = Statement::Assignment {
            targets: vec![ident("x")],
            value: int_lit(42),
            position: pos(),
        };

        assert_eq!(st.transpile_statement(&stmt).unwrap(), "x = 42;\n");
    }

    #[test]
    fn transpiles_augmented_assignment() {
        let st = StatementTranspiler::new();
        let stmt = Statement::AugmentedAssignment {
            target: ident("x"),
            op: AugmentedOperator::Add,
            value: int_lit(1),
            position: pos(),
        };

        assert_eq!(st.transpile_statement(&stmt).unwrap(), "x += 1;\n");
    }

    #[test]
    fn transpiles_expression_statement() {
        let st = StatementTranspiler::new();
        let stmt = Statement::Expression(Expression::Call {
            function: Box::new(ident("print")),
            arguments: vec![int_lit(5)],
            position: pos(),
        });

        assert_eq!(st.transpile_statement(&stmt).unwrap(), "print(5);\n");
    }

    #[test]
    fn transpiles_return_statement() {
        let st = StatementTranspiler::new();
        let stmt = Statement::Return {
            value: Some(int_lit(9)),
            position: pos(),
        };

        assert_eq!(st.transpile_statement(&stmt).unwrap(), "return 9;\n");
    }

    #[test]
    fn transpiles_pass_to_empty_output() {
        let st = StatementTranspiler::new();
        let stmt = Statement::Pass(pos());

        assert_eq!(st.transpile_statement(&stmt).unwrap(), "");
    }

    #[test]
    fn transpiles_break_and_continue() {
        let st = StatementTranspiler::new();

        assert_eq!(st.transpile_statement(&Statement::Break(pos())).unwrap(), "break;\n");
        assert_eq!(
            st.transpile_statement(&Statement::Continue(pos())).unwrap(),
            "continue;\n"
        );
    }

    #[test]
    fn multiple_assignment_targets_not_supported_yet() {
        let st = StatementTranspiler::new();
        let stmt = Statement::Assignment {
            targets: vec![ident("a"), ident("b")],
            value: int_lit(1),
            position: pos(),
        };

        assert!(matches!(
            st.transpile_statement(&stmt),
            Err(StatementTranspileError::MultipleAssignmentTargets)
        ));
    }
}
