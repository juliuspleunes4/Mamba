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
        "sorted" if args.len() == 1 => Some(transpile_sorted(&args[0])),
        "enumerate" if args.len() == 1 => Some(format!("({}).into_iter().enumerate()", args[0])),
        "zip" if args.len() == 2 => Some(format!("({}).into_iter().zip({})", args[0], args[1])),
        "map" if args.len() == 2 => Some(format!("({}).into_iter().map({})", args[1], args[0])),
        "filter" if args.len() == 2 => Some(format!("({}).into_iter().filter({})", args[1], args[0])),
        "all" if args.len() == 1 => Some(format!("({}).into_iter().all(|x| x)", args[0])),
        "any" if args.len() == 1 => Some(format!("({}).into_iter().any(|x| x)", args[0])),
        "input" => transpile_input(args),
        "chr" if args.len() == 1 => Some(format!(
            "char::from_u32(({}) as u32).expect(\"chr() arg not in range(0x110000)\")",
            args[0]
        )),
        "ord" if args.len() == 1 => Some(format!(
            "({}).chars().next().expect(\"ord() expected a character\") as u32",
            args[0]
        )),
        "hex" if args.len() == 1 => Some(format!("format!(\"0x{{:x}}\", ({}))", args[0])),
        "oct" if args.len() == 1 => Some(format!("format!(\"0o{{:o}}\", ({}))", args[0])),
        "bin" if args.len() == 1 => Some(format!("format!(\"0b{{:b}}\", ({}))", args[0])),
        "round" if args.len() == 1 => Some(format!("({}).round()", args[0])),
        "round" if args.len() == 2 => Some(format!(
            "(({} * 10f64.powi(({}) as i32)).round() / 10f64.powi(({}) as i32))",
            args[0], args[1], args[1]
        )),
        "pow" if args.len() == 2 => Some(format!("(({} as f64).powf({} as f64))", args[0], args[1])),
        "divmod" if args.len() == 2 => Some(format!("(({} / {}), ({} % {}))", args[0], args[1], args[0], args[1])),
        "hash" if args.len() == 1 => Some(transpile_hash(&args[0])),
        "id" if args.len() == 1 => Some(format!("(&{} as *const _ as usize)", args[0])),
        "isinstance" if args.len() == 2 => Some(format!(
            "(std::any::type_name_of_val(&{}) == {})",
            args[0], args[1]
        )),
        "issubclass" if args.len() == 2 => Some(format!("({} == {})", args[0], args[1])),
        "callable" if args.len() == 1 => Some(transpile_callable(&args[0])),
        "getattr" => transpile_getattr(args),
        "setattr" if args.len() == 3 => Some(format!(
            "{{ let _ = (&{}, &{}, &{}); () }}",
            args[0], args[1], args[2]
        )),
        "hasattr" if args.len() == 2 => Some(format!("{{ let _ = (&{}, &{}); false }}", args[0], args[1])),
        "delattr" if args.len() == 2 => Some(format!("{{ let _ = (&{}, &{}); () }}", args[0], args[1])),
        "dir" => transpile_dir(args),
        "vars" => transpile_vars(args),
        "globals" if args.is_empty() => Some("std::collections::HashMap::<String, String>::new()".to_string()),
        "locals" if args.is_empty() => Some("std::collections::HashMap::<String, String>::new()".to_string()),
        "iter" if args.len() == 1 => Some(format!("({}).into_iter()", args[0])),
        "next" if args.len() == 1 => Some(format!(
            "({}).next().expect(\"next() called on exhausted iterator\")",
            args[0]
        )),
        "next" if args.len() == 2 => Some(format!("({}).next().unwrap_or({})", args[0], args[1])),
        "slice" => transpile_slice(args),
        "format" if args.len() == 1 => Some(format!("({}).to_string()", args[0])),
        "repr" if args.len() == 1 => Some(format!("format!(\"{{:?}}\", ({}))", args[0])),
        "ascii" if args.len() == 1 => Some(format!(
            "format!(\"{{:?}}\", ({})).chars().flat_map(|c| c.escape_default()).collect::<String>()",
            args[0]
        )),
        "bytes" if args.len() == 1 => Some(format!("({}).as_bytes().to_vec()", args[0])),
        "bytearray" if args.len() == 1 => Some(format!("({}).as_bytes().to_vec()", args[0])),
        "memoryview" if args.len() == 1 => Some(format!("({}).as_bytes()", args[0])),
        "frozenset" if args.len() == 1 => Some(format!(
            "({}).into_iter().collect::<std::collections::BTreeSet<_>>()",
            args[0]
        )),
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

fn transpile_sorted(arg: &str) -> String {
    format!(
        "{{ let mut __mamba_sorted = ({}).into_iter().collect::<Vec<_>>(); __mamba_sorted.sort(); __mamba_sorted }}",
        arg
    )
}

fn transpile_input(args: &[String]) -> Option<String> {
    match args.len() {
        0 => Some(
            "{ let mut __mamba_input = String::new(); std::io::stdin().read_line(&mut __mamba_input).expect(\"input() failed\"); __mamba_input.trim_end_matches(['\\n', '\\r']).to_string() }".to_string(),
        ),
        1 => Some(format!(
            "{{ use std::io::Write; print!(\"{{}}\", {}); std::io::stdout().flush().expect(\"flush failed\"); let mut __mamba_input = String::new(); std::io::stdin().read_line(&mut __mamba_input).expect(\"input() failed\"); __mamba_input.trim_end_matches(['\\n', '\\r']).to_string() }}",
            args[0]
        )),
        _ => None,
    }
}

fn transpile_hash(arg: &str) -> String {
    format!(
        "{{ use std::hash::{{Hash, Hasher}}; let mut __mamba_hasher = std::collections::hash_map::DefaultHasher::new(); ({}).hash(&mut __mamba_hasher); __mamba_hasher.finish() }}",
        arg
    )
}

fn transpile_callable(arg: &str) -> String {
    format!("{{ let _ = &{}; true }}", arg)
}

fn transpile_getattr(args: &[String]) -> Option<String> {
    match args.len() {
        2 => Some(format!(
            "{{ let _ = (&{}, &{}); panic!(\"getattr() baseline lowering requires runtime reflection support\") }}",
            args[0], args[1]
        )),
        3 => Some(format!("{{ let _ = (&{}, &{}); {} }}", args[0], args[1], args[2])),
        _ => None,
    }
}

fn transpile_dir(args: &[String]) -> Option<String> {
    match args.len() {
        0 => Some("Vec::<String>::new()".to_string()),
        1 => Some(format!("{{ let _ = &{}; Vec::<String>::new() }}", args[0])),
        _ => None,
    }
}

fn transpile_vars(args: &[String]) -> Option<String> {
    match args.len() {
        0 => Some("std::collections::HashMap::<String, String>::new()".to_string()),
        1 => Some(format!(
            "{{ let _ = &{}; std::collections::HashMap::<String, String>::new() }}",
            args[0]
        )),
        _ => None,
    }
}

fn transpile_slice(args: &[String]) -> Option<String> {
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
        assert_eq!(
            transpile_builtin_call("sorted", &["values".to_string()]),
            Some(
                "{ let mut __mamba_sorted = (values).into_iter().collect::<Vec<_>>(); __mamba_sorted.sort(); __mamba_sorted }"
                    .to_string()
            )
        );
    }

    #[test]
    fn transpiles_math_and_conversion_helpers() {
        assert_eq!(
            transpile_builtin_call("round", &["x".to_string()]),
            Some("(x).round()".to_string())
        );
        assert_eq!(
            transpile_builtin_call("pow", &["x".to_string(), "y".to_string()]),
            Some("((x as f64).powf(y as f64))".to_string())
        );
        assert_eq!(
            transpile_builtin_call("divmod", &["a".to_string(), "b".to_string()]),
            Some("((a / b), (a % b))".to_string())
        );
        assert_eq!(
            transpile_builtin_call("hex", &["255".to_string()]),
            Some("format!(\"0x{:x}\", (255))".to_string())
        );
        assert_eq!(
            transpile_builtin_call("ord", &["s".to_string()]),
            Some("(s).chars().next().expect(\"ord() expected a character\") as u32".to_string())
        );
    }

    #[test]
    fn transpiles_iter_like_and_repr_helpers() {
        assert_eq!(
            transpile_builtin_call("iter", &["xs".to_string()]),
            Some("(xs).into_iter()".to_string())
        );
        assert_eq!(
            transpile_builtin_call("next", &["it".to_string(), "0".to_string()]),
            Some("(it).next().unwrap_or(0)".to_string())
        );
        assert_eq!(
            transpile_builtin_call("repr", &["x".to_string()]),
            Some("format!(\"{:?}\", (x))".to_string())
        );
        assert_eq!(
            transpile_builtin_call("bytes", &["s".to_string()]),
            Some("(s).as_bytes().to_vec()".to_string())
        );
        assert_eq!(
            transpile_builtin_call("frozenset", &["xs".to_string()]),
            Some("(xs).into_iter().collect::<std::collections::BTreeSet<_>>()".to_string())
        );
    }

    #[test]
    fn transpiles_input_builtin() {
        assert!(transpile_builtin_call("input", &[])
            .expect("input() should transpile")
            .contains("read_line"));

        assert!(transpile_builtin_call("input", &["prompt".to_string()])
            .expect("input(prompt) should transpile")
            .contains("print!(\"{}\", prompt)"));
    }

    #[test]
    fn transpiles_introspection_builtins_baseline() {
        assert_eq!(
            transpile_builtin_call("isinstance", &["obj".to_string(), "ty".to_string()]),
            Some("(std::any::type_name_of_val(&obj) == ty)".to_string())
        );
        assert_eq!(
            transpile_builtin_call("issubclass", &["a".to_string(), "b".to_string()]),
            Some("(a == b)".to_string())
        );
        assert_eq!(
            transpile_builtin_call("callable", &["f".to_string()]),
            Some("{ let _ = &f; true }".to_string())
        );
        assert_eq!(
            transpile_builtin_call("setattr", &["o".to_string(), "n".to_string(), "v".to_string()]),
            Some("{ let _ = (&o, &n, &v); () }".to_string())
        );
        assert_eq!(
            transpile_builtin_call("hasattr", &["o".to_string(), "n".to_string()]),
            Some("{ let _ = (&o, &n); false }".to_string())
        );
    }

    #[test]
    fn transpiles_attribute_and_scope_helpers_baseline() {
        assert_eq!(
            transpile_builtin_call("getattr", &["o".to_string(), "n".to_string(), "d".to_string()]),
            Some("{ let _ = (&o, &n); d }".to_string())
        );
        assert_eq!(
            transpile_builtin_call("dir", &[]),
            Some("Vec::<String>::new()".to_string())
        );
        assert_eq!(
            transpile_builtin_call("vars", &["obj".to_string()]),
            Some("{ let _ = &obj; std::collections::HashMap::<String, String>::new() }".to_string())
        );
        assert_eq!(
            transpile_builtin_call("globals", &[]),
            Some("std::collections::HashMap::<String, String>::new()".to_string())
        );
        assert_eq!(
            transpile_builtin_call("locals", &[]),
            Some("std::collections::HashMap::<String, String>::new()".to_string())
        );
    }

    #[test]
    fn unknown_or_invalid_arity_falls_back() {
        assert_eq!(transpile_builtin_call("unknown", &[]), None);
        assert_eq!(
            transpile_builtin_call("len", &["a".to_string(), "b".to_string()]),
            None
        );
        assert_eq!(transpile_builtin_call("input", &["a".to_string(), "b".to_string()]), None);
        assert_eq!(transpile_builtin_call("globals", &["x".to_string()]), None);
        assert_eq!(transpile_builtin_call("setattr", &["a".to_string(), "b".to_string()]), None);
    }
}
