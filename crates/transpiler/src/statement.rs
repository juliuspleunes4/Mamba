use mamba_parser::ast::{AugmentedOperator, Expression, Literal, Parameter, ParameterKind, Statement};
use thiserror::Error;

use crate::codegen::{CodeGenerator, CodegenError};
use crate::expression::{ExpressionTranspileError, ExpressionTranspiler};
use crate::type_mapping::{TypeMapper, TypeMappingError};

#[derive(Debug, Clone)]
struct LoopContext {
    break_flag: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct TranspileContext {
    next_temp_id: usize,
    loop_stack: Vec<LoopContext>,
}

impl TranspileContext {
    fn allocate_break_flag(&mut self) -> String {
        let name = format!("__mamba_loop_broke_{}", self.next_temp_id);
        self.next_temp_id += 1;
        name
    }

    fn push_loop(&mut self, break_flag: Option<String>) {
        self.loop_stack.push(LoopContext { break_flag });
    }

    fn pop_loop(&mut self) {
        let _ = self.loop_stack.pop();
    }

    fn current_break_flag(&self) -> Option<&str> {
        self.loop_stack.last()?.break_flag.as_deref()
    }
}

#[derive(Debug, Error)]
pub enum StatementTranspileError {
    #[error("unsupported statement kind for current transpilation phase")]
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

/// Transpiles core statement forms for Phase 4.4 and control flow for Phase 4.5.
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
        let mut context = TranspileContext::default();
        self.transpile_into_with_context(&mut generator, statement, &mut context)?;
        Ok(generator.into_string())
    }

    pub fn transpile_statements(
        &self,
        statements: &[Statement],
    ) -> Result<String, StatementTranspileError> {
        let mut generator = CodeGenerator::new();
        let mut context = TranspileContext::default();
        self.transpile_block(&mut generator, statements, &mut context)?;
        Ok(generator.into_string())
    }

    pub fn transpile_into(
        &self,
        generator: &mut CodeGenerator,
        statement: &Statement,
    ) -> Result<(), StatementTranspileError> {
        let mut context = TranspileContext::default();
        self.transpile_into_with_context(generator, statement, &mut context)
    }

    fn transpile_into_with_context(
        &self,
        generator: &mut CodeGenerator,
        statement: &Statement,
        context: &mut TranspileContext,
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
                if let Some(flag) = context.current_break_flag() {
                    generator.emit_line(&format!("{} = true;", flag));
                }
                generator.emit_line("break;");
                Ok(())
            }
            Statement::Continue(_) => {
                generator.emit_line("continue;");
                Ok(())
            }
            Statement::If {
                condition,
                then_block,
                elif_blocks,
                else_block,
                ..
            } => {
                let condition_expr = self.expr.transpile(condition)?;
                generator.open_block(&format!("if {}", condition_expr));
                self.transpile_block(generator, then_block, context)?;
                generator.close_block()?;

                for (elif_condition, elif_body) in elif_blocks {
                    let elif_expr = self.expr.transpile(elif_condition)?;
                    generator.open_block(&format!("else if {}", elif_expr));
                    self.transpile_block(generator, elif_body, context)?;
                    generator.close_block()?;
                }

                if let Some(else_body) = else_block {
                    generator.open_block("else");
                    self.transpile_block(generator, else_body, context)?;
                    generator.close_block()?;
                }

                Ok(())
            }
            Statement::While {
                condition,
                body,
                else_block,
                ..
            } => {
                let break_flag = else_block.as_ref().map(|_| context.allocate_break_flag());
                if let Some(flag) = &break_flag {
                    generator.emit_line(&format!("let mut {} = false;", flag));
                }

                let condition_expr = self.expr.transpile(condition)?;
                context.push_loop(break_flag.clone());
                generator.open_block(&format!("while {}", condition_expr));
                self.transpile_block(generator, body, context)?;
                generator.close_block()?;
                context.pop_loop();

                if let (Some(else_body), Some(flag)) = (else_block, break_flag.as_deref()) {
                    generator.open_block(&format!("if !{}", flag));
                    self.transpile_block(generator, else_body, context)?;
                    generator.close_block()?;
                }

                Ok(())
            }
            Statement::For {
                target,
                iter,
                body,
                else_block,
                ..
            } => {
                let break_flag = else_block.as_ref().map(|_| context.allocate_break_flag());
                if let Some(flag) = &break_flag {
                    generator.emit_line(&format!("let mut {} = false;", flag));
                }

                let rendered_target = self.expr.transpile(target)?;
                let rendered_iter = self.expr.transpile(iter)?;

                context.push_loop(break_flag.clone());
                generator.open_block(&format!("for {} in {}", rendered_target, rendered_iter));
                self.transpile_block(generator, body, context)?;
                generator.close_block()?;
                context.pop_loop();

                if let (Some(else_body), Some(flag)) = (else_block, break_flag.as_deref()) {
                    generator.open_block(&format!("if !{}", flag));
                    self.transpile_block(generator, else_body, context)?;
                    generator.close_block()?;
                }

                Ok(())
            }
            Statement::FunctionDef {
                name,
                parameters,
                body,
                is_async,
                return_type,
                decorators,
                ..
            } => self.transpile_function_definition(
                generator,
                name,
                parameters,
                body,
                *is_async,
                return_type.as_ref(),
                decorators,
                context,
            ),
            _ => Err(StatementTranspileError::UnsupportedStatement),
        }
    }

    fn transpile_block(
        &self,
        generator: &mut CodeGenerator,
        statements: &[Statement],
        context: &mut TranspileContext,
    ) -> Result<(), StatementTranspileError> {
        for statement in statements {
            self.transpile_into_with_context(generator, statement, context)?;
        }
        Ok(())
    }

    fn transpile_function_definition(
        &self,
        generator: &mut CodeGenerator,
        name: &str,
        parameters: &[Parameter],
        body: &[Statement],
        is_async: bool,
        return_type: Option<&Expression>,
        decorators: &[Expression],
        context: &mut TranspileContext,
    ) -> Result<(), StatementTranspileError> {
        for decorator in decorators {
            let rendered_decorator = self.expr.transpile(decorator)?;
            generator.emit_line(&format!("// @{}", rendered_decorator));
        }

        let signature = self.render_function_signature(name, parameters, is_async, return_type)?;
        generator.open_block(&signature);

        for parameter in parameters {
            if let Some(default_expr) = &parameter.default {
                if matches!(parameter.kind, ParameterKind::Regular | ParameterKind::PositionalOnly | ParameterKind::KwOnly) {
                    let rendered_default = self.expr.transpile(default_expr)?;
                    generator.emit_line(&format!(
                        "let mut {} = {}.unwrap_or({});",
                        parameter.name, parameter.name, rendered_default
                    ));
                }
            }
        }

        if self.try_emit_direct_tail_recursion_optimization(generator, name, parameters, body)? {
            generator.close_block()?;
            return Ok(());
        }

        self.transpile_block(generator, body, context)?;
        generator.close_block()?;
        Ok(())
    }

    fn try_emit_direct_tail_recursion_optimization(
        &self,
        generator: &mut CodeGenerator,
        function_name: &str,
        parameters: &[Parameter],
        body: &[Statement],
    ) -> Result<bool, StatementTranspileError> {
        if body.len() != 1 {
            return Ok(false);
        }

        let tail_call_args = match &body[0] {
            Statement::Return {
                value:
                    Some(Expression::Call {
                        function,
                        arguments,
                        ..
                    }),
                ..
            } => match function.as_ref() {
                Expression::Identifier { name, .. } if name == function_name => arguments,
                _ => return Ok(false),
            },
            _ => return Ok(false),
        };

        let updatable_parameters = parameters
            .iter()
            .filter(|param| {
                matches!(
                    param.kind,
                    ParameterKind::Regular | ParameterKind::PositionalOnly | ParameterKind::KwOnly
                )
            })
            .collect::<Vec<_>>();

        if updatable_parameters.len() != tail_call_args.len() {
            return Ok(false);
        }

        generator.emit_line("// Tail recursion optimized to an explicit loop");
        for parameter in &updatable_parameters {
            generator.emit_line(&format!("let mut {} = {};", parameter.name, parameter.name));
        }

        generator.open_block("loop");
        for (index, arg_expr) in tail_call_args.iter().enumerate() {
            let rendered_arg = self.expr.transpile(arg_expr)?;
            generator.emit_line(&format!("let __mamba_next_{} = {};", index, rendered_arg));
        }

        for (index, parameter) in updatable_parameters.iter().enumerate() {
            generator.emit_line(&format!("{} = __mamba_next_{};", parameter.name, index));
        }
        generator.emit_line("continue;");
        generator.close_block()?;

        Ok(true)
    }

    fn render_function_signature(
        &self,
        name: &str,
        parameters: &[Parameter],
        is_async: bool,
        return_type: Option<&Expression>,
    ) -> Result<String, StatementTranspileError> {
        let rendered_params = parameters
            .iter()
            .map(|param| self.render_parameter(param))
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");

        let return_rust_type = if let Some(ann) = return_type {
            self.types.render_annotation(ann)?
        } else {
            "()".to_string()
        };

        let fn_prefix = if is_async { "async fn" } else { "fn" };
        Ok(format!(
            "{} {}({}) -> {}",
            fn_prefix, name, rendered_params, return_rust_type
        ))
    }

    fn render_parameter(&self, parameter: &Parameter) -> Result<String, StatementTranspileError> {
        let base_type = self.parameter_base_type(parameter)?;

        let rendered = match parameter.kind {
            ParameterKind::Regular | ParameterKind::PositionalOnly | ParameterKind::KwOnly => {
                if parameter.default.is_some() {
                    format!("{}: Option<{}>", parameter.name, base_type)
                } else {
                    format!("{}: {}", parameter.name, base_type)
                }
            }
            ParameterKind::VarArgs => format!("{}: Vec<{}>", parameter.name, base_type),
            ParameterKind::VarKwargs => format!(
                "{}: std::collections::HashMap<String, {}>",
                parameter.name, base_type
            ),
        };

        Ok(rendered)
    }

    fn parameter_base_type(&self, parameter: &Parameter) -> Result<String, StatementTranspileError> {
        if let Some(annotation) = &parameter.type_annotation {
            return Ok(self.types.render_annotation(annotation)?);
        }

        if let Some(default_expr) = &parameter.default {
            return Ok(infer_expression_type(default_expr));
        }

        Ok("()".to_string())
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

fn infer_expression_type(expr: &Expression) -> String {
    match expr {
        Expression::Literal(Literal::Integer { .. }) => "i64".to_string(),
        Expression::Literal(Literal::Float { .. }) => "f64".to_string(),
        Expression::Literal(Literal::String { .. }) => "String".to_string(),
        Expression::Literal(Literal::Boolean { .. }) => "bool".to_string(),
        Expression::Literal(Literal::None { .. }) => "Option<()>".to_string(),
        _ => "()".to_string(),
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
        AugmentedOperator, Expression, Literal, Parameter, ParameterKind, Statement,
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

    fn bool_lit(value: bool) -> Expression {
        Expression::Literal(Literal::Boolean {
            value,
            position: pos(),
        })
    }

    fn param(name: &str) -> Parameter {
        Parameter {
            name: name.to_string(),
            kind: ParameterKind::Regular,
            default: None,
            type_annotation: None,
            position: pos(),
        }
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

    #[test]
    fn transpiles_if_statement() {
        let st = StatementTranspiler::new();
        let stmt = Statement::If {
            condition: ident("ready"),
            then_block: vec![Statement::Expression(ident("run"))],
            elif_blocks: vec![],
            else_block: None,
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "if ready {\n    run;\n}\n"
        );
    }

    #[test]
    fn transpiles_if_else_statement() {
        let st = StatementTranspiler::new();
        let stmt = Statement::If {
            condition: ident("ok"),
            then_block: vec![Statement::Expression(ident("left"))],
            elif_blocks: vec![],
            else_block: Some(vec![Statement::Expression(ident("right"))]),
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "if ok {\n    left;\n}\nelse {\n    right;\n}\n"
        );
    }

    #[test]
    fn transpiles_if_elif_else_statement() {
        let st = StatementTranspiler::new();
        let stmt = Statement::If {
            condition: ident("a"),
            then_block: vec![Statement::Expression(ident("first"))],
            elif_blocks: vec![(ident("b"), vec![Statement::Expression(ident("second"))])],
            else_block: Some(vec![Statement::Expression(ident("third"))]),
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "if a {\n    first;\n}\nelse if b {\n    second;\n}\nelse {\n    third;\n}\n"
        );
    }

    #[test]
    fn transpiles_while_loop() {
        let st = StatementTranspiler::new();
        let stmt = Statement::While {
            condition: ident("running"),
            body: vec![Statement::Expression(ident("tick"))],
            else_block: None,
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "while running {\n    tick;\n}\n"
        );
    }

    #[test]
    fn transpiles_for_loop_iterator_pattern() {
        let st = StatementTranspiler::new();
        let stmt = Statement::For {
            target: ident("item"),
            iter: ident("items"),
            body: vec![Statement::Expression(ident("work"))],
            else_block: None,
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "for item in items {\n    work;\n}\n"
        );
    }

    #[test]
    fn transpiles_nested_control_flow() {
        let st = StatementTranspiler::new();
        let nested = Statement::While {
            condition: bool_lit(true),
            body: vec![Statement::If {
                condition: ident("flag"),
                then_block: vec![Statement::Continue(pos())],
                elif_blocks: vec![],
                else_block: Some(vec![Statement::Break(pos())]),
                position: pos(),
            }],
            else_block: None,
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&nested).unwrap(),
            "while true {\n    if flag {\n        continue;\n    }\n    else {\n        break;\n    }\n}\n"
        );
    }

    #[test]
    fn transpiles_while_else_with_break_tracking() {
        let st = StatementTranspiler::new();
        let stmt = Statement::While {
            condition: ident("cond"),
            body: vec![Statement::Break(pos())],
            else_block: Some(vec![Statement::Expression(ident("after"))]),
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "let mut __mamba_loop_broke_0 = false;\nwhile cond {\n    __mamba_loop_broke_0 = true;\n    break;\n}\nif !__mamba_loop_broke_0 {\n    after;\n}\n"
        );
    }

    #[test]
    fn transpiles_for_else_with_break_tracking() {
        let st = StatementTranspiler::new();
        let stmt = Statement::For {
            target: ident("x"),
            iter: ident("xs"),
            body: vec![Statement::Break(pos())],
            else_block: Some(vec![Statement::Expression(ident("done"))]),
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "let mut __mamba_loop_broke_0 = false;\nfor x in xs {\n    __mamba_loop_broke_0 = true;\n    break;\n}\nif !__mamba_loop_broke_0 {\n    done;\n}\n"
        );
    }

    #[test]
    fn transpiles_function_definition_and_signature() {
        let st = StatementTranspiler::new();
        let mut a = param("a");
        a.type_annotation = Some(ident("int"));
        let mut b = param("b");
        b.type_annotation = Some(ident("int"));

        let stmt = Statement::FunctionDef {
            name: "add".to_string(),
            parameters: vec![a, b],
            body: vec![Statement::Return {
                value: Some(Expression::BinaryOp {
                    left: Box::new(ident("a")),
                    op: mamba_parser::ast::BinaryOperator::Add,
                    right: Box::new(ident("b")),
                    position: pos(),
                }),
                position: pos(),
            }],
            is_async: false,
            return_type: Some(ident("int")),
            decorators: vec![],
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "fn add(a: i64, b: i64) -> i64 {\n    return (a + b);\n}\n"
        );
    }

    #[test]
    fn transpiles_default_parameters() {
        let st = StatementTranspiler::new();
        let mut p = param("count");
        p.type_annotation = Some(ident("int"));
        p.default = Some(int_lit(5));

        let stmt = Statement::FunctionDef {
            name: "take".to_string(),
            parameters: vec![p],
            body: vec![Statement::Return {
                value: Some(ident("count")),
                position: pos(),
            }],
            is_async: false,
            return_type: Some(ident("int")),
            decorators: vec![],
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "fn take(count: Option<i64>) -> i64 {\n    let mut count = count.unwrap_or(5);\n    return count;\n}\n"
        );
    }

    #[test]
    fn transpiles_async_function() {
        let st = StatementTranspiler::new();
        let stmt = Statement::FunctionDef {
            name: "run".to_string(),
            parameters: vec![],
            body: vec![Statement::Return {
                value: None,
                position: pos(),
            }],
            is_async: true,
            return_type: None,
            decorators: vec![],
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "async fn run() -> () {\n    return;\n}\n"
        );
    }

    #[test]
    fn transpiles_varargs_and_kwargs_parameters() {
        let st = StatementTranspiler::new();
        let mut args = param("args");
        args.kind = ParameterKind::VarArgs;
        args.type_annotation = Some(ident("str"));

        let mut kwargs = param("kwargs");
        kwargs.kind = ParameterKind::VarKwargs;
        kwargs.type_annotation = Some(ident("bool"));

        let stmt = Statement::FunctionDef {
            name: "collect".to_string(),
            parameters: vec![args, kwargs],
            body: vec![Statement::Pass(pos())],
            is_async: false,
            return_type: None,
            decorators: vec![],
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "fn collect(args: Vec<String>, kwargs: std::collections::HashMap<String, bool>) -> () {\n}\n"
        );
    }

    #[test]
    fn handles_multiple_return_statements() {
        let st = StatementTranspiler::new();
        let stmt = Statement::FunctionDef {
            name: "sign".to_string(),
            parameters: vec![param("x")],
            body: vec![Statement::If {
                condition: ident("x"),
                then_block: vec![Statement::Return {
                    value: Some(int_lit(1)),
                    position: pos(),
                }],
                elif_blocks: vec![],
                else_block: Some(vec![Statement::Return {
                    value: Some(int_lit(-1)),
                    position: pos(),
                }]),
                position: pos(),
            }],
            is_async: false,
            return_type: Some(ident("int")),
            decorators: vec![],
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "fn sign(x: ()) -> i64 {\n    if x {\n        return 1;\n    }\n    else {\n        return -1;\n    }\n}\n"
        );
    }

    #[test]
    fn transpiles_basic_decorators_as_metadata_comments() {
        let st = StatementTranspiler::new();
        let stmt = Statement::FunctionDef {
            name: "work".to_string(),
            parameters: vec![],
            body: vec![Statement::Return {
                value: None,
                position: pos(),
            }],
            is_async: false,
            return_type: None,
            decorators: vec![ident("trace"), ident("timed")],
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "// @trace\n// @timed\nfn work() -> () {\n    return;\n}\n"
        );
    }

    #[test]
    fn keeps_recursive_call_for_non_tail_pattern() {
        let st = StatementTranspiler::new();
        let mut n_param = param("n");
        n_param.type_annotation = Some(ident("int"));

        let stmt = Statement::FunctionDef {
            name: "fact".to_string(),
            parameters: vec![n_param],
            body: vec![Statement::Return {
                value: Some(Expression::BinaryOp {
                    left: Box::new(ident("n")),
                    op: mamba_parser::ast::BinaryOperator::Multiply,
                    right: Box::new(Expression::Call {
                        function: Box::new(ident("fact")),
                        arguments: vec![Expression::BinaryOp {
                            left: Box::new(ident("n")),
                            op: mamba_parser::ast::BinaryOperator::Subtract,
                            right: Box::new(int_lit(1)),
                            position: pos(),
                        }],
                        position: pos(),
                    }),
                    position: pos(),
                }),
                position: pos(),
            }],
            is_async: false,
            return_type: Some(ident("int")),
            decorators: vec![],
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "fn fact(n: i64) -> i64 {\n    return (n * fact((n - 1)));\n}\n"
        );
    }

    #[test]
    fn optimizes_direct_tail_recursion_to_loop() {
        let st = StatementTranspiler::new();
        let mut n_param = param("n");
        n_param.type_annotation = Some(ident("int"));

        let stmt = Statement::FunctionDef {
            name: "countdown".to_string(),
            parameters: vec![n_param],
            body: vec![Statement::Return {
                value: Some(Expression::Call {
                    function: Box::new(ident("countdown")),
                    arguments: vec![Expression::BinaryOp {
                        left: Box::new(ident("n")),
                        op: mamba_parser::ast::BinaryOperator::Subtract,
                        right: Box::new(int_lit(1)),
                        position: pos(),
                    }],
                    position: pos(),
                }),
                position: pos(),
            }],
            is_async: false,
            return_type: Some(ident("int")),
            decorators: vec![],
            position: pos(),
        };

        assert_eq!(
            st.transpile_statement(&stmt).unwrap(),
            "fn countdown(n: i64) -> i64 {\n    // Tail recursion optimized to an explicit loop\n    let mut n = n;\n    loop {\n        let __mamba_next_0 = (n - 1);\n        n = __mamba_next_0;\n        continue;\n    }\n}\n"
        );
    }
}
