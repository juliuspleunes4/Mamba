use mamba_parser::ast::{BinaryOperator, Comprehension, Expression, Literal, UnaryOperator};
use thiserror::Error;

use crate::builtins::transpile_builtin_call;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExpressionTranspileError {
    #[error("expression kind is not supported yet")]
    UnsupportedExpression,
    #[error("only single-generator comprehensions are supported in this phase")]
    UnsupportedComprehension,
}

/// Transpiles Mamba expression AST nodes into Rust expression source.
#[derive(Debug, Clone, Default)]
pub struct ExpressionTranspiler;

impl ExpressionTranspiler {
    pub fn new() -> Self {
        Self
    }

    pub fn transpile(&self, expr: &Expression) -> Result<String, ExpressionTranspileError> {
        match expr {
            Expression::Literal(literal) => self.transpile_literal(literal),
            Expression::Identifier { name, .. } => Ok(name.clone()),
            Expression::BinaryOp {
                left, op, right, ..
            } => {
                let left_expr = self.transpile(left)?;
                let right_expr = self.transpile(right)?;
                self.transpile_binary(op, &left_expr, &right_expr)
            }
            Expression::UnaryOp { op, operand, .. } => {
                let value = self.transpile(operand)?;
                self.transpile_unary(op, &value)
            }
            Expression::Parenthesized { expr, .. } => Ok(format!("({})", self.transpile(expr)?)),
            Expression::Call {
                function,
                arguments,
                ..
            } => {
                let function_name = self.transpile(function)?;
                let args = arguments
                    .iter()
                    .map(|arg| self.transpile(arg))
                    .collect::<Result<Vec<_>, _>>()?;

                if let Some(builtin) = transpile_builtin_call(&function_name, &args) {
                    return Ok(builtin);
                }

                Ok(format!("{}({})", function_name, args.join(", ")))
            }
            Expression::Attribute {
                object,
                attribute,
                ..
            } => Ok(format!("{}.{}", self.transpile(object)?, attribute)),
            Expression::Subscript { object, index, .. } => {
                Ok(format!("{}[{}]", self.transpile(object)?, self.transpile(index)?))
            }
            Expression::Tuple { elements, .. } => {
                let items = elements
                    .iter()
                    .map(|element| self.transpile(element))
                    .collect::<Result<Vec<_>, _>>()?;

                if items.len() == 1 {
                    Ok(format!("({},)", items[0]))
                } else {
                    Ok(format!("({})", items.join(", ")))
                }
            }
            Expression::List { elements, .. } => {
                let items = elements
                    .iter()
                    .map(|element| self.transpile(element))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                Ok(format!("vec![{}]", items))
            }
            Expression::Dict { pairs, .. } => {
                let entries = pairs
                    .iter()
                    .map(|(key, value)| {
                        let key_expr = self.transpile(key)?;
                        let value_expr = self.transpile(value)?;
                        Ok(format!("({}, {})", key_expr, value_expr))
                    })
                    .collect::<Result<Vec<String>, ExpressionTranspileError>>()?
                    .join(", ");

                Ok(format!("std::collections::HashMap::from([{}])", entries))
            }
            Expression::Lambda {
                parameters, body, ..
            } => {
                let params = parameters.join(", ");
                let rendered_body = self.transpile(body)?;
                Ok(format!("|{}| {}", params, rendered_body))
            }
            Expression::ListComp {
                element,
                generators,
                ..
            } => self.transpile_list_comprehension(element, generators),
            _ => Err(ExpressionTranspileError::UnsupportedExpression),
        }
    }

    fn transpile_list_comprehension(
        &self,
        element: &Expression,
        generators: &[Comprehension],
    ) -> Result<String, ExpressionTranspileError> {
        if generators.len() != 1 {
            return Err(ExpressionTranspileError::UnsupportedComprehension);
        }

        let generator = &generators[0];
        let iter_expr = self.transpile(&generator.iter)?;
        let element_expr = self.transpile(element)?;

        if generator.conditions.is_empty() {
            return Ok(format!(
                "({}).into_iter().map(|{}| {}).collect::<Vec<_>>()",
                iter_expr, generator.target, element_expr
            ));
        }

        let rendered_conditions = generator
            .conditions
            .iter()
            .map(|condition| self.transpile(condition))
            .collect::<Result<Vec<_>, _>>()?
            .join(" && ");

        Ok(format!(
            "({}).into_iter().filter(|{}| {}).map(|{}| {}).collect::<Vec<_>>()",
            iter_expr, generator.target, rendered_conditions, generator.target, element_expr
        ))
    }

    fn transpile_literal(&self, literal: &Literal) -> Result<String, ExpressionTranspileError> {
        match literal {
            Literal::Integer { value, .. } => Ok(value.to_string()),
            Literal::Float { value, .. } => Ok(value.to_string()),
            Literal::String { value, .. } => Ok(format!("\"{}\"", escape_rust_string(value))),
            Literal::Boolean { value, .. } => Ok(value.to_string()),
            Literal::None { .. } => Ok("None".to_string()),
            Literal::Ellipsis { .. } => Ok("..".to_string()),
        }
    }

    fn transpile_unary(
        &self,
        op: &UnaryOperator,
        operand: &str,
    ) -> Result<String, ExpressionTranspileError> {
        let rust = match op {
            UnaryOperator::Minus => format!("(-{})", operand),
            UnaryOperator::Plus => format!("(+{})", operand),
            UnaryOperator::Not => format!("(!{})", operand),
            UnaryOperator::BitwiseNot => format!("(!{})", operand),
        };
        Ok(rust)
    }

    fn transpile_binary(
        &self,
        op: &BinaryOperator,
        left: &str,
        right: &str,
    ) -> Result<String, ExpressionTranspileError> {
        let rust = match op {
            BinaryOperator::Add => format!("({} + {})", left, right),
            BinaryOperator::Subtract => format!("({} - {})", left, right),
            BinaryOperator::Multiply => format!("({} * {})", left, right),
            BinaryOperator::Divide => format!("({} / {})", left, right),
            BinaryOperator::FloorDivide => format!("({} / {})", left, right),
            BinaryOperator::Modulo => format!("({} % {})", left, right),
            BinaryOperator::Power => format!("({}.pow({}))", left, right),
            BinaryOperator::Equal => format!("({} == {})", left, right),
            BinaryOperator::NotEqual => format!("({} != {})", left, right),
            BinaryOperator::LessThan => format!("({} < {})", left, right),
            BinaryOperator::LessThanEq => format!("({} <= {})", left, right),
            BinaryOperator::GreaterThan => format!("({} > {})", left, right),
            BinaryOperator::GreaterThanEq => format!("({} >= {})", left, right),
            BinaryOperator::And => format!("({} && {})", left, right),
            BinaryOperator::Or => format!("({} || {})", left, right),
            BinaryOperator::BitwiseAnd => format!("({} & {})", left, right),
            BinaryOperator::BitwiseOr => format!("({} | {})", left, right),
            BinaryOperator::BitwiseXor => format!("({} ^ {})", left, right),
            BinaryOperator::LeftShift => format!("({} << {})", left, right),
            BinaryOperator::RightShift => format!("({} >> {})", left, right),
            BinaryOperator::In => format!("({}.contains(&{}))", right, left),
            BinaryOperator::NotIn => format!("(!{}.contains(&{}))", right, left),
            BinaryOperator::Is => format!("({} == {})", left, right),
            BinaryOperator::IsNot => format!("({} != {})", left, right),
        };
        Ok(rust)
    }
}

fn escape_rust_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::{ExpressionTranspileError, ExpressionTranspiler};
    use mamba_parser::ast::{
        BinaryOperator, Comprehension, Expression, Literal, UnaryOperator,
    };
    use mamba_parser::token::SourcePosition;

    fn pos() -> SourcePosition {
        SourcePosition::new(1, 1, 0)
    }

    fn int_lit(value: i64) -> Expression {
        Expression::Literal(Literal::Integer {
            value,
            position: pos(),
        })
    }

    fn str_lit(value: &str) -> Expression {
        Expression::Literal(Literal::String {
            value: value.to_string(),
            position: pos(),
        })
    }

    fn ident(name: &str) -> Expression {
        Expression::Identifier {
            name: name.to_string(),
            position: pos(),
        }
    }

    #[test]
    fn transpiles_literals() {
        let tr = ExpressionTranspiler::new();
        assert_eq!(tr.transpile(&int_lit(42)).unwrap(), "42");
        assert_eq!(tr.transpile(&str_lit("hi\n\"x\"")).unwrap(), "\"hi\\n\\\"x\\\"\"");
    }

    #[test]
    fn transpiles_identifier() {
        let tr = ExpressionTranspiler::new();
        assert_eq!(tr.transpile(&ident("value")).unwrap(), "value");
    }

    #[test]
    fn transpiles_binary_arithmetic() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::BinaryOp {
            left: Box::new(int_lit(2)),
            op: BinaryOperator::Add,
            right: Box::new(int_lit(3)),
            position: pos(),
        };
        assert_eq!(tr.transpile(&expr).unwrap(), "(2 + 3)");
    }

    #[test]
    fn transpiles_comparison() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::BinaryOp {
            left: Box::new(ident("x")),
            op: BinaryOperator::GreaterThanEq,
            right: Box::new(int_lit(10)),
            position: pos(),
        };
        assert_eq!(tr.transpile(&expr).unwrap(), "(x >= 10)");
    }

    #[test]
    fn transpiles_logical() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::BinaryOp {
            left: Box::new(ident("ok")),
            op: BinaryOperator::And,
            right: Box::new(ident("ready")),
            position: pos(),
        };
        assert_eq!(tr.transpile(&expr).unwrap(), "(ok && ready)");
    }

    #[test]
    fn transpiles_unary() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::UnaryOp {
            op: UnaryOperator::Not,
            operand: Box::new(ident("flag")),
            position: pos(),
        };
        assert_eq!(tr.transpile(&expr).unwrap(), "(!flag)");
    }

    #[test]
    fn transpiles_function_call() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::Call {
            function: Box::new(ident("add")),
            arguments: vec![int_lit(1), int_lit(2)],
            position: pos(),
        };
        assert_eq!(tr.transpile(&expr).unwrap(), "add(1, 2)");
    }

    #[test]
    fn transpiles_builtin_len_call() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::Call {
            function: Box::new(ident("len")),
            arguments: vec![ident("items")],
            position: pos(),
        };

        assert_eq!(tr.transpile(&expr).unwrap(), "(items).len()");
    }

    #[test]
    fn transpiles_builtin_range_call() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::Call {
            function: Box::new(ident("range")),
            arguments: vec![int_lit(1), int_lit(10), int_lit(2)],
            position: pos(),
        };

        assert_eq!(tr.transpile(&expr).unwrap(), "(1..10).step_by((2) as usize)");
    }

    #[test]
    fn transpiles_builtin_print_call() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::Call {
            function: Box::new(ident("print")),
            arguments: vec![ident("x")],
            position: pos(),
        };

        assert_eq!(
            tr.transpile(&expr).unwrap(),
            r#"println!("{}", vec![format!("{:?}", x)].join(" "))"#
        );
    }

    #[test]
    fn falls_back_for_non_builtin_calls() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::Call {
            function: Box::new(ident("custom_fn")),
            arguments: vec![int_lit(7)],
            position: pos(),
        };

        assert_eq!(tr.transpile(&expr).unwrap(), "custom_fn(7)");
    }

    #[test]
    fn transpiles_parenthesized() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::Parenthesized {
            expr: Box::new(ident("x")),
            position: pos(),
        };
        assert_eq!(tr.transpile(&expr).unwrap(), "(x)");
    }

    #[test]
    fn transpiles_tuple() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::Tuple {
            elements: vec![int_lit(1), int_lit(2)],
            position: pos(),
        };
        assert_eq!(tr.transpile(&expr).unwrap(), "(1, 2)");
    }

    #[test]
    fn transpiles_list_as_vec() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::List {
            elements: vec![int_lit(1), int_lit(2), int_lit(3)],
            position: pos(),
        };
        assert_eq!(tr.transpile(&expr).unwrap(), "vec![1, 2, 3]");
    }

    #[test]
    fn transpiles_dict_as_hashmap_from() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::Dict {
            pairs: vec![(str_lit("a"), int_lit(1)), (str_lit("b"), int_lit(2))],
            position: pos(),
        };
        assert_eq!(
            tr.transpile(&expr).unwrap(),
            "std::collections::HashMap::from([((\"a\"), 1), ((\"b\"), 2)])"
                .replace("((\"a\"), 1)", "(\"a\", 1)")
                .replace("((\"b\"), 2)", "(\"b\", 2)")
        );
    }

    #[test]
    fn transpiles_subscript_and_attribute() {
        let tr = ExpressionTranspiler::new();
        let attr = Expression::Attribute {
            object: Box::new(ident("obj")),
            attribute: "field".to_string(),
            position: pos(),
        };
        let sub = Expression::Subscript {
            object: Box::new(ident("items")),
            index: Box::new(int_lit(0)),
            position: pos(),
        };

        assert_eq!(tr.transpile(&attr).unwrap(), "obj.field");
        assert_eq!(tr.transpile(&sub).unwrap(), "items[0]");
    }

    #[test]
    fn transpiles_membership() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::BinaryOp {
            left: Box::new(int_lit(1)),
            op: BinaryOperator::In,
            right: Box::new(ident("items")),
            position: pos(),
        };

        assert_eq!(tr.transpile(&expr).unwrap(), "(items.contains(&1))");
    }

    #[test]
    fn unsupported_expression_errors() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::Set {
            elements: vec![int_lit(1)],
            position: pos(),
        };

        assert_eq!(
            tr.transpile(&expr),
            Err(ExpressionTranspileError::UnsupportedExpression)
        );
    }

    #[test]
    fn transpiles_lambda_expression() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::Lambda {
            parameters: vec!["x".to_string(), "y".to_string()],
            body: Box::new(Expression::BinaryOp {
                left: Box::new(ident("x")),
                op: BinaryOperator::Add,
                right: Box::new(ident("y")),
                position: pos(),
            }),
            position: pos(),
        };

        assert_eq!(tr.transpile(&expr).unwrap(), "|x, y| (x + y)");
    }

    #[test]
    fn transpiles_list_comprehension_with_map() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::ListComp {
            element: Box::new(Expression::BinaryOp {
                left: Box::new(ident("x")),
                op: BinaryOperator::Add,
                right: Box::new(int_lit(1)),
                position: pos(),
            }),
            generators: vec![Comprehension {
                target: "x".to_string(),
                iter: ident("items"),
                conditions: vec![],
                position: pos(),
            }],
            position: pos(),
        };

        assert_eq!(
            tr.transpile(&expr).unwrap(),
            "(items).into_iter().map(|x| (x + 1)).collect::<Vec<_>>()"
        );
    }

    #[test]
    fn transpiles_list_comprehension_with_filter_and_map() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::ListComp {
            element: Box::new(ident("x")),
            generators: vec![Comprehension {
                target: "x".to_string(),
                iter: ident("items"),
                conditions: vec![Expression::BinaryOp {
                    left: Box::new(ident("x")),
                    op: BinaryOperator::GreaterThan,
                    right: Box::new(int_lit(0)),
                    position: pos(),
                }],
                position: pos(),
            }],
            position: pos(),
        };

        assert_eq!(
            tr.transpile(&expr).unwrap(),
            "(items).into_iter().filter(|x| (x > 0)).map(|x| x).collect::<Vec<_>>()"
        );
    }

    #[test]
    fn rejects_multi_generator_list_comprehension_for_now() {
        let tr = ExpressionTranspiler::new();
        let expr = Expression::ListComp {
            element: Box::new(ident("x")),
            generators: vec![
                Comprehension {
                    target: "x".to_string(),
                    iter: ident("xs"),
                    conditions: vec![],
                    position: pos(),
                },
                Comprehension {
                    target: "y".to_string(),
                    iter: ident("ys"),
                    conditions: vec![],
                    position: pos(),
                },
            ],
            position: pos(),
        };

        assert_eq!(
            tr.transpile(&expr),
            Err(ExpressionTranspileError::UnsupportedComprehension)
        );
    }
}
