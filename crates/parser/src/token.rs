//! Token definitions for the Mamba lexer

use std::fmt;

/// Represents a position in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl SourcePosition {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    pub fn start() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }
}

impl fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A token with its position in the source code
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub position: SourcePosition,
    pub lexeme: String,
}

impl Token {
    pub fn new(kind: TokenKind, position: SourcePosition, lexeme: String) -> Self {
        Self {
            kind,
            position,
            lexeme,
        }
    }
}

/// All token types in the Mamba language
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    True,
    False,
    None,

    // Identifiers and keywords
    Identifier(String),
    
    // Keywords
    And,
    As,
    Assert,
    Async,
    Await,
    Break,
    Class,
    Continue,
    Def,
    Del,
    Elif,
    Else,
    Except,
    Finally,
    For,
    From,
    Global,
    If,
    Import,
    In,
    Is,
    Lambda,
    Nonlocal,
    Not,
    Or,
    Pass,
    Raise,
    Return,
    Try,
    While,
    With,
    Yield,
    Match,
    Case,

    // Operators
    Plus,              // +
    Minus,             // -
    Star,              // *
    Slash,             // /
    DoubleSlash,       // //
    Percent,           // %
    DoubleStar,        // **
    
    // Comparison
    Equal,             // ==
    NotEqual,          // !=
    Less,              // <
    Greater,           // >
    LessEqual,         // <=
    GreaterEqual,      // >=
    
    // Bitwise
    Ampersand,         // &
    Pipe,              // |
    Caret,             // ^
    Tilde,             // ~
    LeftShift,         // <<
    RightShift,        // >>
    
    // Assignment
    Assign,            // =
    PlusAssign,        // +=
    MinusAssign,       // -=
    StarAssign,        // *=
    SlashAssign,       // /=
    DoubleSlashAssign, // //=
    PercentAssign,     // %=
    DoubleStarAssign,  // **=
    AmpersandAssign,   // &=
    PipeAssign,        // |=
    CaretAssign,       // ^=
    LeftShiftAssign,   // <<=
    RightShiftAssign,  // >>=
    
    // Walrus operator
    Walrus,            // :=
    
    // Delimiters
    LeftParen,         // (
    RightParen,        // )
    LeftBracket,       // [
    RightBracket,      // ]
    LeftBrace,         // {
    RightBrace,        // }
    Comma,             // ,
    Colon,             // :
    Semicolon,         // ;
    Dot,               // .
    Arrow,             // ->
    Ellipsis,          // ...
    At,                // @
    
    // Special
    Newline,
    Indent,
    Dedent,
    Eof,
    
    // Comment (not typically emitted, but useful for preprocessing)
    Comment(String),
}

impl TokenKind {
    /// Check if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::And
                | TokenKind::As
                | TokenKind::Assert
                | TokenKind::Async
                | TokenKind::Await
                | TokenKind::Break
                | TokenKind::Class
                | TokenKind::Continue
                | TokenKind::Def
                | TokenKind::Del
                | TokenKind::Elif
                | TokenKind::Else
                | TokenKind::Except
                | TokenKind::Finally
                | TokenKind::For
                | TokenKind::From
                | TokenKind::Global
                | TokenKind::If
                | TokenKind::Import
                | TokenKind::In
                | TokenKind::Is
                | TokenKind::Lambda
                | TokenKind::Nonlocal
                | TokenKind::Not
                | TokenKind::Or
                | TokenKind::Pass
                | TokenKind::Raise
                | TokenKind::Return
                | TokenKind::Try
                | TokenKind::While
                | TokenKind::With
                | TokenKind::Yield
                | TokenKind::Match
                | TokenKind::Case
                | TokenKind::True
                | TokenKind::False
                | TokenKind::None
        )
    }

    /// Get the keyword token from a string, if it exists
    pub fn keyword_from_str(s: &str) -> Option<Self> {
        match s {
            "and" => Some(TokenKind::And),
            "as" => Some(TokenKind::As),
            "assert" => Some(TokenKind::Assert),
            "async" => Some(TokenKind::Async),
            "await" => Some(TokenKind::Await),
            "break" => Some(TokenKind::Break),
            "class" => Some(TokenKind::Class),
            "continue" => Some(TokenKind::Continue),
            "def" => Some(TokenKind::Def),
            "del" => Some(TokenKind::Del),
            "elif" => Some(TokenKind::Elif),
            "else" => Some(TokenKind::Else),
            "except" => Some(TokenKind::Except),
            "finally" => Some(TokenKind::Finally),
            "for" => Some(TokenKind::For),
            "from" => Some(TokenKind::From),
            "global" => Some(TokenKind::Global),
            "if" => Some(TokenKind::If),
            "import" => Some(TokenKind::Import),
            "in" => Some(TokenKind::In),
            "is" => Some(TokenKind::Is),
            "lambda" => Some(TokenKind::Lambda),
            "nonlocal" => Some(TokenKind::Nonlocal),
            "not" => Some(TokenKind::Not),
            "or" => Some(TokenKind::Or),
            "pass" => Some(TokenKind::Pass),
            "raise" => Some(TokenKind::Raise),
            "return" => Some(TokenKind::Return),
            "try" => Some(TokenKind::Try),
            "while" => Some(TokenKind::While),
            "with" => Some(TokenKind::With),
            "yield" => Some(TokenKind::Yield),
            "match" => Some(TokenKind::Match),
            "case" => Some(TokenKind::Case),
            "True" => Some(TokenKind::True),
            "False" => Some(TokenKind::False),
            "None" => Some(TokenKind::None),
            _ => None,
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Integer(n) => write!(f, "Integer({})", n),
            TokenKind::Float(n) => write!(f, "Float({})", n),
            TokenKind::String(s) => write!(f, "String(\"{}\")", s),
            TokenKind::True => write!(f, "True"),
            TokenKind::False => write!(f, "False"),
            TokenKind::None => write!(f, "None"),
            TokenKind::Identifier(name) => write!(f, "Identifier({})", name),
            TokenKind::Comment(c) => write!(f, "Comment({})", c),
            _ => write!(f, "{:?}", self),
        }
    }
}
