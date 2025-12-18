use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use mamba_parser::lexer::Lexer;

/// Benchmark tokenizing small Python-like code snippets
fn bench_small_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_files");
    
    // Simple assignment
    let simple_code = "x = 42\ny = 3.14\n";
    group.bench_function("simple_assignment", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(simple_code));
            lexer.tokenize()
        })
    });
    
    // Function definition
    let func_code = r#"
def greet(name):
    print("Hello, " + name)
    return True
"#;
    group.bench_function("function_definition", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(func_code));
            lexer.tokenize()
        })
    });
    
    // Class definition
    let class_code = r#"
class Person:
    def __init__(self, name):
        self.name = name
    
    def greet(self):
        return "Hi, " + self.name
"#;
    group.bench_function("class_definition", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(class_code));
            lexer.tokenize()
        })
    });
    
    group.finish();
}

/// Benchmark tokenizing different token types
fn bench_token_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_types");
    
    // Numbers
    let numbers = "42 3.14 0x1A 0o17 0b1010 1e10 2.5e-3";
    group.bench_function("numbers", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(numbers));
            lexer.tokenize()
        })
    });
    
    // Strings
    let strings = r#""hello" 'world' """multiline
string""" r"raw\nstring" f"formatted {x}""#;
    group.bench_function("strings", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(strings));
            lexer.tokenize()
        })
    });
    
    // Keywords
    let keywords = "if else elif for while def class return break continue pass import from as";
    group.bench_function("keywords", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(keywords));
            lexer.tokenize()
        })
    });
    
    // Operators
    let operators = "+ - * / // ** % == != < > <= >= and or not in is";
    group.bench_function("operators", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(operators));
            lexer.tokenize()
        })
    });
    
    // Identifiers
    let identifiers = "variable_name another_var x y z camelCase snake_case PascalCase";
    group.bench_function("identifiers", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(identifiers));
            lexer.tokenize()
        })
    });
    
    // Unicode identifiers
    let unicode = "café π 数据 переменная変数 한국어";
    group.bench_function("unicode_identifiers", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(unicode));
            lexer.tokenize()
        })
    });
    
    group.finish();
}

/// Benchmark indentation handling
fn bench_indentation(c: &mut Criterion) {
    let mut group = c.benchmark_group("indentation");
    
    // Simple indentation
    let simple_indent = r#"
if True:
    x = 1
    y = 2
"#;
    group.bench_function("simple_indent", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(simple_indent));
            lexer.tokenize()
        })
    });
    
    // Nested indentation
    let nested_indent = r#"
def outer():
    def inner():
        if True:
            x = 1
        return x
    return inner
"#;
    group.bench_function("nested_indent", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(nested_indent));
            lexer.tokenize()
        })
    });
    
    // Deep nesting (10 levels)
    let deep_indent = r#"
if True:
    if True:
        if True:
            if True:
                if True:
                    if True:
                        if True:
                            if True:
                                if True:
                                    if True:
                                        x = 1
"#;
    group.bench_function("deep_nesting", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(deep_indent));
            lexer.tokenize()
        })
    });
    
    group.finish();
}

/// Benchmark medium-sized files (realistic code)
fn bench_medium_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("medium_files");
    
    // Realistic Mamba module (~50 lines)
    let medium_code = r#"
# Calculator module
class Calculator:
    """A simple calculator class."""
    
    def __init__(self):
        self.memory = 0
    
    def add(self, a, b):
        """Add two numbers."""
        return a + b
    
    def subtract(self, a, b):
        """Subtract b from a."""
        return a - b
    
    def multiply(self, a, b):
        """Multiply two numbers."""
        return a * b
    
    def divide(self, a, b):
        """Divide a by b."""
        if b == 0:
            raise ValueError("Cannot divide by zero")
        return a / b
    
    def store(self, value):
        """Store a value in memory."""
        self.memory = value
    
    def recall(self):
        """Recall the stored value."""
        return self.memory
    
    def clear(self):
        """Clear the memory."""
        self.memory = 0

def main():
    calc = Calculator()
    result = calc.add(5, 3)
    print(f"Result: {result}")
    
    calc.store(result)
    memory = calc.recall()
    print(f"Memory: {memory}")

if __name__ == "__main__":
    main()
"#;
    
    group.bench_function("calculator_module", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(medium_code));
            lexer.tokenize()
        })
    });
    
    group.finish();
}

/// Benchmark large files (stress test)
fn bench_large_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_files");
    
    // Generate a large file with 1000 assignments
    let mut large_code = String::new();
    for i in 0..1000 {
        large_code.push_str(&format!("variable_{} = {}\n", i, i));
    }
    
    group.bench_with_input(
        BenchmarkId::new("assignments", "1000_lines"),
        &large_code,
        |b, code| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(code.as_str()));
                lexer.tokenize()
            })
        }
    );
    
    // Generate a large file with 500 function definitions
    let mut func_code = String::new();
    for i in 0..500 {
        func_code.push_str(&format!(
            "def function_{}():\n    return {}\n\n",
            i, i
        ));
    }
    
    group.bench_with_input(
        BenchmarkId::new("functions", "500_functions"),
        &func_code,
        |b, code| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(code.as_str()));
                lexer.tokenize()
            })
        }
    );
    
    group.finish();
}

/// Benchmark edge cases and stress tests
fn bench_edge_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_cases");
    
    // Very long identifier
    let long_id = format!("variable_{}", "x".repeat(1000));
    group.bench_function("long_identifier", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(long_id.as_str()));
            lexer.tokenize()
        })
    });
    
    // Very long string
    let long_string = format!("\"{}\"", "Hello, World! ".repeat(100));
    group.bench_function("long_string", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(long_string.as_str()));
            lexer.tokenize()
        })
    });
    
    // Many operators without spaces
    let operators = "1+2*3/4-5%6**7//8==9!=10<11>12<=13>=14";
    group.bench_function("operators_no_spaces", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(operators));
            lexer.tokenize()
        })
    });
    
    // Mixed content
    let mixed = r#"
# Complex mixed content
x = 42; y = 3.14; z = "hello"
if x > 0 and y < 10:
    result = x + y * 2
    print(f"Result: {result}")
"#;
    group.bench_function("mixed_content", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(mixed));
            lexer.tokenize()
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_small_files,
    bench_token_types,
    bench_indentation,
    bench_medium_files,
    bench_large_files,
    bench_edge_cases
);
criterion_main!(benches);
