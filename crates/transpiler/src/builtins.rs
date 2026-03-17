/// Transpile selected Python built-in calls into Rust expressions/statements.
///
/// Returns `Some(rendered)` when a builtin is recognized and supported, otherwise `None`.
pub fn transpile_builtin_call(name: &str, args: &[String]) -> Option<String> {
    match name {
        "print" => Some(transpile_print(args)),
        "len" if args.len() == 1 => Some(format!("({}).len()", args[0])),
        "range" => transpile_range(args),
        "type" if args.len() == 1 => Some(format!("std::any::type_name_of_val(&{})", args[0])),
        "str" if args.len() == 1 => Some(format!("({}).to_string()", args[0])),
        "int" if args.len() == 1 => Some(format!("({} as i64)", args[0])),
        "float" if args.len() == 1 => Some(format!("({} as f64)", args[0])),
        "bool" if args.len() == 1 => Some(format!("({} != 0)", args[0])),
        "abs" if args.len() == 1 => Some(format!("({}).abs()", args[0])),
        "min" if args.len() == 1 => Some(format!(
            "({}).into_iter().min().expect(\"min() arg is an empty sequence\")",
            args[0]
        )),
        "max" if args.len() == 1 => Some(format!(
            "({}).into_iter().max().expect(\"max() arg is an empty sequence\")",
            args[0]
        )),
        "sum" if args.len() == 1 => Some(format!("({}).into_iter().sum()", args[0])),
        "reversed" if args.len() == 1 => {
            Some(format!("({}).into_iter().rev().collect::<Vec<_>>()", args[0]))
        }
        "enumerate" if args.len() == 1 => Some(format!("({}).into_iter().enumerate()", args[0])),
        "zip" if args.len() == 2 => Some(format!("({}).into_iter().zip({})", args[0], args[1])),
        "map" if args.len() == 2 => Some(format!("({}).into_iter().map({})", args[1], args[0])),
        "filter" if args.len() == 2 => Some(format!("({}).into_iter().filter({})", args[1], args[0])),
        "all" if args.len() == 1 => Some(format!("({}).into_iter().all(|x| x)", args[0])),
        "any" if args.len() == 1 => Some(format!("({}).into_iter().any(|x| x)", args[0])),
        _ => None,
    }
}

fn transpile_print(args: &[String]) -> String {
    if args.is_empty() {
        return "println!()".to_string();
    }

    let parts = args
        .iter()
        .map(|arg| format!("format!(\"{{:?}}\", {})", arg))
        .collect::<Vec<_>>()
        .join(", ");

    format!(r#"println!("{{}}", vec![{}].join(" "))"#, parts)
}

fn transpile_range(args: &[String]) -> Option<String> {
    match args.len() {
        1 => Some(format!("(0..{})", args[0])),
        2 => Some(format!("({}..{})", args[0], args[1])),
        3 => Some(format!("({}..{}).step_by(({}) as usize)", args[0], args[1], args[2])),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::transpile_builtin_call;

    #[test]
    fn transpiles_print_builtin() {
        let result = transpile_builtin_call("print", &["x".to_string(), "y".to_string()]);
        assert_eq!(
            result,
            Some(
                r#"println!("{}", vec![format!("{:?}", x), format!("{:?}", y)].join(" "))"#
                    .to_string()
            )
        );
    }

    #[test]
    fn transpiles_len_and_range_builtins() {
        assert_eq!(
            transpile_builtin_call("len", &["items".to_string()]),
            Some("(items).len()".to_string())
        );
        assert_eq!(
            transpile_builtin_call("range", &["10".to_string()]),
            Some("(0..10)".to_string())
        );
        assert_eq!(
            transpile_builtin_call(
                "range",
                &["1".to_string(), "10".to_string(), "2".to_string()]
            ),
            Some("(1..10).step_by((2) as usize)".to_string())
        );
    }

    #[test]
    fn transpiles_core_cast_and_aggregate_builtins() {
        assert_eq!(
            transpile_builtin_call("str", &["x".to_string()]),
            Some("(x).to_string()".to_string())
        );
        assert_eq!(
            transpile_builtin_call("int", &["x".to_string()]),
            Some("(x as i64)".to_string())
        );
        assert_eq!(
            transpile_builtin_call("sum", &["values".to_string()]),
            Some("(values).into_iter().sum()".to_string())
        );
        assert_eq!(
            transpile_builtin_call("any", &["values".to_string()]),
            Some("(values).into_iter().any(|x| x)".to_string())
        );
    }

    #[test]
    fn unknown_or_invalid_arity_falls_back() {
        assert_eq!(transpile_builtin_call("unknown", &[]), None);
        assert_eq!(
            transpile_builtin_call("len", &["a".to_string(), "b".to_string()]),
            None
        );
    }
}
