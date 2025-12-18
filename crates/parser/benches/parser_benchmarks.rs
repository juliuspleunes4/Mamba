use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mamba_parser::lexer::Lexer;
use mamba_parser::parser::Parser;

/// Helper to parse source code
fn parse(source: &str) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let _ = parser.parse();
}

// ============================================================================
// Small Expressions
// ============================================================================

fn bench_simple_literal(c: &mut Criterion) {
    c.bench_function("parse_simple_literal", |b| {
        b.iter(|| parse(black_box("42\n")))
    });
}

fn bench_binary_expression(c: &mut Criterion) {
    c.bench_function("parse_binary_expression", |b| {
        b.iter(|| parse(black_box("1 + 2 * 3 - 4 / 5\n")))
    });
}

fn bench_nested_expression(c: &mut Criterion) {
    c.bench_function("parse_nested_expression", |b| {
        b.iter(|| parse(black_box("((1 + 2) * (3 - 4)) / (5 ** 6)\n")))
    });
}

fn bench_function_call(c: &mut Criterion) {
    c.bench_function("parse_function_call", |b| {
        b.iter(|| parse(black_box("func(a, b, c=1, d=2)\n")))
    });
}

fn bench_chained_access(c: &mut Criterion) {
    c.bench_function("parse_chained_access", |b| {
        b.iter(|| parse(black_box("obj.attr.method().result[0]\n")))
    });
}

fn bench_list_literal(c: &mut Criterion) {
    c.bench_function("parse_list_literal", |b| {
        b.iter(|| parse(black_box("[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]\n")))
    });
}

fn bench_dict_literal(c: &mut Criterion) {
    c.bench_function("parse_dict_literal", |b| {
        b.iter(|| parse(black_box("{'a': 1, 'b': 2, 'c': 3, 'd': 4, 'e': 5}\n")))
    });
}

fn bench_list_comprehension(c: &mut Criterion) {
    c.bench_function("parse_list_comprehension", |b| {
        b.iter(|| parse(black_box("[x * 2 for x in range(10) if x % 2 == 0]\n")))
    });
}

fn bench_lambda(c: &mut Criterion) {
    c.bench_function("parse_lambda", |b| {
        b.iter(|| parse(black_box("lambda x, y: x + y\n")))
    });
}

// ============================================================================
// Statements
// ============================================================================

fn bench_assignment(c: &mut Criterion) {
    c.bench_function("parse_assignment", |b| {
        b.iter(|| parse(black_box("x = 42\n")))
    });
}

fn bench_multiple_assignment(c: &mut Criterion) {
    c.bench_function("parse_multiple_assignment", |b| {
        b.iter(|| parse(black_box("x = y = z = 42\n")))
    });
}

fn bench_tuple_unpacking(c: &mut Criterion) {
    c.bench_function("parse_tuple_unpacking", |b| {
        b.iter(|| parse(black_box("a, b, c = 1, 2, 3\n")))
    });
}

fn bench_augmented_assignment(c: &mut Criterion) {
    c.bench_function("parse_augmented_assignment", |b| {
        b.iter(|| parse(black_box("x += 42\n")))
    });
}

fn bench_if_statement(c: &mut Criterion) {
    let code = "\
if x > 0:
    print('positive')
elif x < 0:
    print('negative')
else:
    print('zero')
";
    c.bench_function("parse_if_statement", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

fn bench_while_loop(c: &mut Criterion) {
    let code = "\
while x < 10:
    x += 1
    print(x)
";
    c.bench_function("parse_while_loop", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

fn bench_for_loop(c: &mut Criterion) {
    let code = "\
for i in range(10):
    print(i)
";
    c.bench_function("parse_for_loop", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

fn bench_import_statement(c: &mut Criterion) {
    c.bench_function("parse_import", |b| {
        b.iter(|| parse(black_box("import os.path as path\n")))
    });
}

fn bench_from_import(c: &mut Criterion) {
    c.bench_function("parse_from_import", |b| {
        b.iter(|| parse(black_box("from os.path import join, exists\n")))
    });
}

// ============================================================================
// Function Definitions
// ============================================================================

fn bench_simple_function(c: &mut Criterion) {
    let code = "\
def hello():
    print('Hello, World!')
";
    c.bench_function("parse_simple_function", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

fn bench_function_with_params(c: &mut Criterion) {
    let code = "\
def greet(name, greeting='Hello'):
    print(f'{greeting}, {name}!')
";
    c.bench_function("parse_function_with_params", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

fn bench_function_with_all_params(c: &mut Criterion) {
    let code = "\
def complex(a, b=1, /, c=2, *args, d, e=3, **kwargs):
    return a + b + c + d + e
";
    c.bench_function("parse_function_all_params", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

fn bench_function_with_annotations(c: &mut Criterion) {
    let code = "\
def add(x: int, y: int) -> int:
    return x + y
";
    c.bench_function("parse_function_with_annotations", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

fn bench_async_function(c: &mut Criterion) {
    let code = "\
async def fetch(url: str) -> dict:
    return {'status': 'ok'}
";
    c.bench_function("parse_async_function", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

fn bench_decorated_function(c: &mut Criterion) {
    let code = "\
@decorator
@another(arg=1)
def decorated():
    pass
";
    c.bench_function("parse_decorated_function", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

// ============================================================================
// Class Definitions
// ============================================================================

fn bench_simple_class(c: &mut Criterion) {
    let code = "\
class Point:
    pass
";
    c.bench_function("parse_simple_class", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

fn bench_class_with_methods(c: &mut Criterion) {
    let code = "\
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    
    def distance(self):
        return (self.x ** 2 + self.y ** 2) ** 0.5
";
    c.bench_function("parse_class_with_methods", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

fn bench_class_with_inheritance(c: &mut Criterion) {
    let code = "\
class Point3D(Point):
    def __init__(self, x, y, z):
        super().__init__(x, y)
        self.z = z
";
    c.bench_function("parse_class_with_inheritance", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

fn bench_decorated_class(c: &mut Criterion) {
    let code = "\
@dataclass
class Point:
    x: int
    y: int
";
    c.bench_function("parse_decorated_class", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

// ============================================================================
// Medium Files (Realistic Modules)
// ============================================================================

fn bench_calculator_module(c: &mut Criterion) {
    let code = "\
class Calculator:
    def __init__(self):
        self.history = []
    
    def add(self, a, b):
        result = a + b
        self.history.append(f'{a} + {b} = {result}')
        return result
    
    def subtract(self, a, b):
        result = a - b
        self.history.append(f'{a} - {b} = {result}')
        return result
    
    def multiply(self, a, b):
        result = a * b
        self.history.append(f'{a} * {b} = {result}')
        return result
    
    def divide(self, a, b):
        if b == 0:
            raise ValueError('Division by zero')
        result = a / b
        self.history.append(f'{a} / {b} = {result}')
        return result
    
    def get_history(self):
        return self.history
";
    c.bench_function("parse_calculator_module", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

fn bench_data_processor(c: &mut Criterion) {
    let code = "\
from typing import List, Dict, Optional

def process_data(items: List[Dict[str, int]]) -> List[int]:
    results = []
    for item in items:
        if 'value' in item:
            value = item['value']
            if value > 0:
                results.append(value * 2)
            else:
                results.append(0)
    return results

def filter_data(items: List[int], threshold: int = 10) -> List[int]:
    return [x for x in items if x >= threshold]

def aggregate(items: List[int]) -> Dict[str, float]:
    if not items:
        return {'sum': 0, 'mean': 0, 'max': 0, 'min': 0}
    
    total = sum(items)
    return {
        'sum': total,
        'mean': total / len(items),
        'max': max(items),
        'min': min(items),
    }
";
    c.bench_function("parse_data_processor", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

// ============================================================================
// Large Files (Stress Tests)
// ============================================================================

fn bench_100_assignments(c: &mut Criterion) {
    let mut code = String::new();
    for i in 0..100 {
        code.push_str(&format!("var{} = {}\n", i, i));
    }
    
    c.bench_function("parse_100_assignments", |b| {
        b.iter(|| parse(black_box(&code)))
    });
}

fn bench_50_functions(c: &mut Criterion) {
    let mut code = String::new();
    for i in 0..50 {
        code.push_str(&format!("\
def func{}(x, y):
    return x + y + {}

", i, i));
    }
    
    c.bench_function("parse_50_functions", |b| {
        b.iter(|| parse(black_box(&code)))
    });
}

fn bench_deep_nesting(c: &mut Criterion) {
    let mut code = String::new();
    for i in 0..10 {
        code.push_str(&format!("{}if x > {}:\n", "    ".repeat(i), i));
    }
    code.push_str(&format!("{}pass\n", "    ".repeat(10)));
    
    c.bench_function("parse_deep_nesting", |b| {
        b.iter(|| parse(black_box(&code)))
    });
}

fn bench_complex_expressions(c: &mut Criterion) {
    let mut code = String::new();
    for i in 0..50 {
        code.push_str(&format!("result{} = ((a + b) * c - d / e) ** f + g % h | i & j ^ k << l >> m\n", i));
    }
    
    c.bench_function("parse_complex_expressions", |b| {
        b.iter(|| parse(black_box(&code)))
    });
}

// ============================================================================
// Edge Cases
// ============================================================================

fn bench_long_parameter_list(c: &mut Criterion) {
    let params = (0..50).map(|i| format!("p{}", i)).collect::<Vec<_>>().join(", ");
    let code = format!("def func({}):\n    pass\n", params);
    
    c.bench_function("parse_long_parameter_list", |b| {
        b.iter(|| parse(black_box(&code)))
    });
}

fn bench_long_argument_list(c: &mut Criterion) {
    let args = (0..50).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
    let code = format!("func({})\n", args);
    
    c.bench_function("parse_long_argument_list", |b| {
        b.iter(|| parse(black_box(&code)))
    });
}

fn bench_deeply_nested_collections(c: &mut Criterion) {
    c.bench_function("parse_deeply_nested_collections", |b| {
        b.iter(|| parse(black_box("[[[[[[[[[[1]]]]]]]]]]")))
    });
}

fn bench_complex_comprehension(c: &mut Criterion) {
    let code = "[[x * y for y in range(5)] for x in range(10) if x % 2 == 0]\n";
    c.bench_function("parse_complex_comprehension", |b| {
        b.iter(|| parse(black_box(code)))
    });
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    expressions,
    bench_simple_literal,
    bench_binary_expression,
    bench_nested_expression,
    bench_function_call,
    bench_chained_access,
    bench_list_literal,
    bench_dict_literal,
    bench_list_comprehension,
    bench_lambda,
);

criterion_group!(
    statements,
    bench_assignment,
    bench_multiple_assignment,
    bench_tuple_unpacking,
    bench_augmented_assignment,
    bench_if_statement,
    bench_while_loop,
    bench_for_loop,
    bench_import_statement,
    bench_from_import,
);

criterion_group!(
    functions,
    bench_simple_function,
    bench_function_with_params,
    bench_function_with_all_params,
    bench_function_with_annotations,
    bench_async_function,
    bench_decorated_function,
);

criterion_group!(
    classes,
    bench_simple_class,
    bench_class_with_methods,
    bench_class_with_inheritance,
    bench_decorated_class,
);

criterion_group!(
    medium_files,
    bench_calculator_module,
    bench_data_processor,
);

criterion_group!(
    large_files,
    bench_100_assignments,
    bench_50_functions,
    bench_deep_nesting,
    bench_complex_expressions,
);

criterion_group!(
    edge_cases,
    bench_long_parameter_list,
    bench_long_argument_list,
    bench_deeply_nested_collections,
    bench_complex_comprehension,
);

criterion_main!(
    expressions,
    statements,
    functions,
    classes,
    medium_files,
    large_files,
    edge_cases,
);
