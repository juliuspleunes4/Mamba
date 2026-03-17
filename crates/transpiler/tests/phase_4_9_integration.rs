use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use mamba_parser::ast::{
    BinaryOperator, Expression, Literal, Module, Parameter, ParameterKind, Statement,
};
use mamba_parser::token::SourcePosition;
use mamba_transpiler::{CodeGenerator, ExpressionTranspiler, ModuleTranspiler, StatementTranspiler};

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

fn temp_workspace(test_name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "mamba_transpiler_{}_{}_{}",
        test_name,
        std::process::id(),
        stamp
    ));
    fs::create_dir_all(&dir).expect("temp workspace should be created");
    dir
}

fn rustc_cmd() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

fn compile_rust_binary(source_path: &Path, output_path: &Path) {
    let output = Command::new(rustc_cmd())
        .arg(source_path)
        .arg("-o")
        .arg(output_path)
        .output()
        .expect("rustc should execute");

    assert!(
        output.status.success(),
        "rustc failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn compile_rust_test_binary(source_path: &Path, output_path: &Path) {
    let output = Command::new(rustc_cmd())
        .arg("--test")
        .arg(source_path)
        .arg("-o")
        .arg(output_path)
        .output()
        .expect("rustc --test should execute");

    assert!(
        output.status.success(),
        "rustc --test failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_binary(binary_path: &Path) {
    let output = Command::new(binary_path)
        .output()
        .expect("compiled binary should run");

    assert!(
        output.status.success(),
        "binary failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn transpilation_units_smoke_test() {
    let mut generator = CodeGenerator::new();
    generator.open_block("fn smoke()");
    generator.emit_line("let x = 1;");
    generator.close_block().expect("block should close");
    assert_eq!(generator.as_str(), "fn smoke() {\n    let x = 1;\n}\n");

    let expr_transpiler = ExpressionTranspiler::new();
    let expr = Expression::BinaryOp {
        left: Box::new(int_lit(2)),
        op: BinaryOperator::Add,
        right: Box::new(int_lit(3)),
        position: pos(),
    };
    assert_eq!(expr_transpiler.transpile(&expr).unwrap(), "(2 + 3)");

    let statement_transpiler = StatementTranspiler::new();
    let statement = Statement::Assignment {
        targets: vec![ident("x")],
        value: int_lit(10),
        position: pos(),
    };
    assert_eq!(statement_transpiler.transpile_statement(&statement).unwrap(), "x = 10;\n");

    let module_transpiler = ModuleTranspiler::new();
    let module = Module {
        statements: vec![Statement::Expression(Expression::Call {
            function: Box::new(ident("noop")),
            arguments: vec![],
            position: pos(),
        })],
        position: pos(),
    };
    assert_eq!(
        module_transpiler.transpile_module(&module).unwrap(),
        "fn main() {\n    noop();\n}\n"
    );
}

#[test]
fn complex_nested_module_transpiles_and_compiles() {
    let transpiler = ModuleTranspiler::new();

    let compute_function = Statement::FunctionDef {
        name: "compute".to_string(),
        parameters: vec![Parameter {
            name: "n".to_string(),
            kind: ParameterKind::Regular,
            default: None,
            type_annotation: Some(ident("int")),
            position: pos(),
        }],
        body: vec![
            Statement::AnnAssignment {
                target: "x".to_string(),
                annotation: ident("int"),
                value: Some(ident("n")),
                position: pos(),
            },
            Statement::AnnAssignment {
                target: "acc".to_string(),
                annotation: ident("int"),
                value: Some(int_lit(0)),
                position: pos(),
            },
            Statement::While {
                condition: Expression::BinaryOp {
                    left: Box::new(ident("x")),
                    op: BinaryOperator::GreaterThan,
                    right: Box::new(int_lit(0)),
                    position: pos(),
                },
                body: vec![
                    Statement::Assignment {
                        targets: vec![ident("acc")],
                        value: Expression::BinaryOp {
                            left: Box::new(ident("acc")),
                            op: BinaryOperator::Add,
                            right: Box::new(ident("x")),
                            position: pos(),
                        },
                        position: pos(),
                    },
                    Statement::Assignment {
                        targets: vec![ident("x")],
                        value: Expression::BinaryOp {
                            left: Box::new(ident("x")),
                            op: BinaryOperator::Subtract,
                            right: Box::new(int_lit(1)),
                            position: pos(),
                        },
                        position: pos(),
                    },
                ],
                else_block: None,
                position: pos(),
            },
            Statement::Return {
                value: Some(ident("acc")),
                position: pos(),
            },
        ],
        is_async: false,
        return_type: Some(ident("int")),
        decorators: vec![],
        position: pos(),
    };

    let module = Module {
        statements: vec![
            compute_function,
            Statement::Assignment {
                targets: vec![ident("result")],
                value: Expression::Call {
                    function: Box::new(ident("compute")),
                    arguments: vec![int_lit(5)],
                    position: pos(),
                },
                position: pos(),
            },
        ],
        position: pos(),
    };

    let source = transpiler
        .transpile_module(&module)
        .expect("module should transpile");

    let dir = temp_workspace("complex_nested_compile");
    let source_path = dir.join("generated.rs");
    let binary_path = dir.join("generated_bin");
    fs::write(&source_path, source).expect("generated source should be written");

    compile_rust_binary(&source_path, &binary_path);
    run_binary(&binary_path);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn generated_runtime_behavior_matches_expectations() {
    let transpiler = ModuleTranspiler::new();

    let add1_fn = Statement::FunctionDef {
        name: "add1".to_string(),
        parameters: vec![Parameter {
            name: "x".to_string(),
            kind: ParameterKind::Regular,
            default: None,
            type_annotation: Some(ident("int")),
            position: pos(),
        }],
        body: vec![Statement::Return {
            value: Some(Expression::BinaryOp {
                left: Box::new(ident("x")),
                op: BinaryOperator::Add,
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
        statements: vec![add1_fn],
        position: pos(),
    };

    let mut source = transpiler
        .transpile_module(&module)
        .expect("module should transpile");

    source.push_str(
        "\n#[test]\nfn generated_add1_runtime_behavior() {\n    assert_eq!(add1(41), 42);\n}\n",
    );

    let dir = temp_workspace("runtime_behavior");
    let source_path = dir.join("generated_test.rs");
    let test_bin = dir.join("generated_test_bin");
    fs::write(&source_path, source).expect("generated test source should be written");

    compile_rust_test_binary(&source_path, &test_bin);
    run_binary(&test_bin);

    let _ = fs::remove_dir_all(dir);
}
