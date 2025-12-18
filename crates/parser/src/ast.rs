//! Abstract Syntax Tree (AST) node definitions for Mamba
//!
//! This module defines the structure of the AST that represents parsed Mamba code.
//! Each node includes position information for error reporting.

use crate::token::SourcePosition;

/// A single import item in an import statement (module name + optional alias)
#[derive(Debug, Clone, PartialEq)]
pub struct ImportItem {
    /// Dotted module name (e.g., "os", "os.path")
    pub module: String,
    /// Optional alias (the name after 'as')
    pub alias: Option<String>,
    pub position: SourcePosition,
}

/// A single name imported in a from...import statement (name + optional alias)
#[derive(Debug, Clone, PartialEq)]
pub struct FromImportItem {
    /// Name being imported (e.g., "path", "*")
    pub name: String,
    /// Optional alias (the name after 'as')
    pub alias: Option<String>,
    pub position: SourcePosition,
}

/// A complete Mamba program (module)
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub statements: Vec<Statement>,
    pub position: SourcePosition,
}

/// Represents any statement in Mamba
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Expression statement (e.g., function call)
    Expression(Expression),
    /// Assignment statement (x = 5 or x = y = 5)
    Assignment {
        targets: Vec<Expression>,
        value: Expression,
        position: SourcePosition,
    },
    /// Annotated assignment (x: int or x: int = 5)
    AnnAssignment {
        target: String,
        annotation: Expression,
        value: Option<Expression>,
        position: SourcePosition,
    },
    /// Augmented assignment (x += 5)
    AugmentedAssignment {
        target: Expression,
        op: AugmentedOperator,
        value: Expression,
        position: SourcePosition,
    },
    /// Pass statement
    Pass(SourcePosition),
    /// Break statement
    Break(SourcePosition),
    /// Continue statement
    Continue(SourcePosition),
    /// Return statement
    Return {
        value: Option<Expression>,
        position: SourcePosition,
    },
    /// Assert statement (assert condition, optional_message)
    Assert {
        condition: Expression,
        message: Option<Expression>,
        position: SourcePosition,
    },
    /// Del statement (del x, del obj.attr, del list[0])
    Del {
        targets: Vec<Expression>,
        position: SourcePosition,
    },
    /// Global statement (global x, y)
    Global {
        names: Vec<String>,
        position: SourcePosition,
    },
    /// Nonlocal statement (nonlocal x, y)
    Nonlocal {
        names: Vec<String>,
        position: SourcePosition,
    },
    /// Raise statement (raise, raise Exception, raise Exception("msg"))
    Raise {
        exception: Option<Expression>,
        position: SourcePosition,
    },
    /// Import statement (import module, import module as alias)
    Import {
        items: Vec<ImportItem>,
        position: SourcePosition,
    },
    /// From...import statement (from module import name, from module import *)
    FromImport {
        module: String,
        items: Vec<FromImportItem>,
        position: SourcePosition,
    },
    /// If statement
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        elif_blocks: Vec<(Expression, Vec<Statement>)>,
        else_block: Option<Vec<Statement>>,
        position: SourcePosition,
    },
    /// While loop
    While {
        condition: Expression,
        body: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
        position: SourcePosition,
    },
    /// For loop
    For {
        target: Expression,
        iter: Expression,
        body: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
        position: SourcePosition,
    },
    /// Function definition
    FunctionDef {
        name: String,
        parameters: Vec<Parameter>,
        body: Vec<Statement>,
        is_async: bool,
        return_type: Option<Expression>,
        position: SourcePosition,
    },
    /// Class definition
    ClassDef {
        name: String,
        bases: Vec<Expression>,
        body: Vec<Statement>,
        position: SourcePosition,
    },
}

/// Represents any expression in Mamba
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Literal value (42, 3.14, "hello", True, False, None)
    Literal(Literal),
    /// Identifier (variable reference)
    Identifier {
        name: String,
        position: SourcePosition,
    },
    /// Binary operation (x + y, a == b)
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
        position: SourcePosition,
    },
    /// Unary operation (-x, not y)
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
        position: SourcePosition,
    },
    /// Parenthesized expression
    Parenthesized {
        expr: Box<Expression>,
        position: SourcePosition,
    },
    /// Function call (func(arg1, arg2))
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
        position: SourcePosition,
    },
    /// Attribute access (obj.attr)
    Attribute {
        object: Box<Expression>,
        attribute: String,
        position: SourcePosition,
    },
    /// Subscript (list[index])
    Subscript {
        object: Box<Expression>,
        index: Box<Expression>,
        position: SourcePosition,
    },
    /// List literal ([1, 2, 3])
    List {
        elements: Vec<Expression>,
        position: SourcePosition,
    },
    /// Tuple literal ((1, 2, 3))
    Tuple {
        elements: Vec<Expression>,
        position: SourcePosition,
    },
    /// Dict literal ({key: value})
    Dict {
        pairs: Vec<(Expression, Expression)>,
        position: SourcePosition,
    },
    /// Set literal ({1, 2, 3})
    Set {
        elements: Vec<Expression>,
        position: SourcePosition,
    },
    /// Lambda expression (lambda x, y: x + y)
    Lambda {
        parameters: Vec<String>,
        body: Box<Expression>,
        position: SourcePosition,
    },
    /// Conditional expression (x if condition else y)
    Conditional {
        condition: Box<Expression>,
        true_expr: Box<Expression>,
        false_expr: Box<Expression>,
        position: SourcePosition,
    },
    /// Walrus operator / Assignment expression (name := value)
    AssignmentExpr {
        target: String,
        value: Box<Expression>,
        position: SourcePosition,
    },
    /// List comprehension ([expr for target in iter])
    ListComp {
        element: Box<Expression>,
        generators: Vec<Comprehension>,
        position: SourcePosition,
    },
    /// Dict comprehension ({key: value for target in iter})
    DictComp {
        key: Box<Expression>,
        value: Box<Expression>,
        generators: Vec<Comprehension>,
        position: SourcePosition,
    },
    /// Set comprehension ({expr for target in iter})
    SetComp {
        element: Box<Expression>,
        generators: Vec<Comprehension>,
        position: SourcePosition,
    },
    /// Generator expression ((expr for target in iter))
    GeneratorExpr {
        element: Box<Expression>,
        generators: Vec<Comprehension>,
        position: SourcePosition,
    },
    /// Starred expression (*expr) - used in unpacking
    Starred {
        value: Box<Expression>,
        position: SourcePosition,
    },
}

/// Comprehension clause (for target in iter [if condition])
#[derive(Debug, Clone, PartialEq)]
pub struct Comprehension {
    pub target: String,
    pub iter: Expression,
    pub conditions: Vec<Expression>,
    pub position: SourcePosition,
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer {
        value: i64,
        position: SourcePosition,
    },
    Float {
        value: f64,
        position: SourcePosition,
    },
    String {
        value: String,
        position: SourcePosition,
    },
    Boolean {
        value: bool,
        position: SourcePosition,
    },
    None {
        position: SourcePosition,
    },
    Ellipsis {
        position: SourcePosition,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,         // +
    Subtract,    // -
    Multiply,    // *
    Divide,      // /
    FloorDivide, // //
    Modulo,      // %
    Power,       // **
    
    // Comparison
    Equal,        // ==
    NotEqual,     // !=
    LessThan,     // <
    LessThanEq,   // <=
    GreaterThan,  // >
    GreaterThanEq, // >=
    
    // Logical
    And, // and
    Or,  // or
    
    // Bitwise
    BitwiseAnd,      // &
    BitwiseOr,       // |
    BitwiseXor,      // ^
    LeftShift,       // <<
    RightShift,      // >>
    
    // Membership
    In,    // in
    NotIn, // not in
    
    // Identity
    Is,    // is
    IsNot, // is not
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Minus,       // -
    Plus,        // +
    Not,         // not
    BitwiseNot,  // ~
}

/// Augmented assignment operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AugmentedOperator {
    Add,         // +=
    Subtract,    // -=
    Multiply,    // *=
    Divide,      // /=
    FloorDivide, // //=
    Modulo,      // %=
    Power,       // **=
    BitwiseAnd,  // &=
    BitwiseOr,   // |=
    BitwiseXor,  // ^=
    LeftShift,   // <<=
    RightShift,  // >>=
}

/// Parameter kind (regular, *args, **kwargs, keyword-only)
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterKind {
    PositionalOnly, // before / marker
    Regular,   // x or x=default
    VarArgs,   // *args
    VarKwargs, // **kwargs
    KwOnly,    // keyword-only (after * or *args)
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub kind: ParameterKind,
    pub default: Option<Expression>,
    pub type_annotation: Option<Expression>,
    pub position: SourcePosition,
}

impl Literal {
    /// Get the position of this literal
    pub fn position(&self) -> &SourcePosition {
        match self {
            Literal::Integer { position, .. } => position,
            Literal::Float { position, .. } => position,
            Literal::String { position, .. } => position,
            Literal::Boolean { position, .. } => position,
            Literal::None { position } => position,
            Literal::Ellipsis { position } => position,
        }
    }
}

impl Expression {
    /// Get the position of this expression
    pub fn position(&self) -> &SourcePosition {
        match self {
            Expression::Literal(lit) => lit.position(),
            Expression::Identifier { position, .. } => position,
            Expression::BinaryOp { position, .. } => position,
            Expression::UnaryOp { position, .. } => position,
            Expression::Parenthesized { position, .. } => position,
            Expression::Call { position, .. } => position,
            Expression::Attribute { position, .. } => position,
            Expression::Subscript { position, .. } => position,
            Expression::List { position, .. } => position,
            Expression::Tuple { position, .. } => position,
            Expression::Dict { position, .. } => position,
            Expression::Set { position, .. } => position,
            Expression::Lambda { position, .. } => position,
            Expression::Conditional { position, .. } => position,
            Expression::AssignmentExpr { position, .. } => position,
            Expression::ListComp { position, .. } => position,
            Expression::DictComp { position, .. } => position,
            Expression::SetComp { position, .. } => position,
            Expression::GeneratorExpr { position, .. } => position,
            Expression::Starred { position, .. } => position,
        }
    }
}

impl Statement {
    /// Get the position of this statement
    pub fn position(&self) -> &SourcePosition {
        match self {
            Statement::Expression(expr) => expr.position(),
            Statement::Assignment { position, .. } => position,
            Statement::AnnAssignment { position, .. } => position,
            Statement::AugmentedAssignment { position, .. } => position,
            Statement::Pass(position) => position,
            Statement::Break(position) => position,
            Statement::Continue(position) => position,
            Statement::Return { position, .. } => position,
            Statement::Assert { position, .. } => position,
            Statement::Del { position, .. } => position,
            Statement::Global { position, .. } => position,
            Statement::Nonlocal { position, .. } => position,
            Statement::Raise { position, .. } => position,
            Statement::Import { position, .. } => position,
            Statement::FromImport { position, .. } => position,
            Statement::If { position, .. } => position,
            Statement::While { position, .. } => position,
            Statement::For { position, .. } => position,
            Statement::FunctionDef { position, .. } => position,
            Statement::ClassDef { position, .. } => position,
        }
    }
}
