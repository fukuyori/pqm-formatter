//! Token definitions for Power Query M language

use std::fmt;

/// Source location information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { start, end, line, column }
    }
    
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line,
            column: self.column,
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self { start: 0, end: 0, line: 1, column: 1 }
    }
}

/// Token with span information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Token kinds for Power Query M
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Null,
    True,
    False,
    Number(f64),
    Text(String),
    
    // Identifiers
    Identifier(String),
    QuotedIdentifier(String),  // #"identifier"
    
    // Keywords
    And,
    As,
    Each,
    Else,
    Error,
    If,
    In,
    Is,
    Let,
    Meta,
    Not,
    Or,
    Otherwise,
    Section,
    Shared,
    Then,
    Try,
    Type,
    
    // Built-in type keywords
    HashBinary,      // #binary
    HashDate,        // #date
    HashDatetime,    // #datetime
    HashDatetimezone,// #datetimezone
    HashDuration,    // #duration
    HashInfinity,    // #infinity
    HashNan,         // #nan
    HashSections,    // #sections
    HashShared,      // #shared
    HashTable,       // #table
    HashTime,        // #time
    
    // Operators
    Plus,            // +
    Minus,           // -
    Star,            // *
    Slash,           // /
    Ampersand,       // &
    Equal,           // =
    NotEqual,        // <>
    LessThan,        // <
    LessThanEqual,   // <=
    GreaterThan,     // >
    GreaterThanEqual,// >=
    FatArrow,        // =>
    QuestionQuestion,// ??
    Dot,             // .
    DotDot,          // ..
    DotDotDot,       // ...
    
    // Punctuation
    Comma,           // ,
    Semicolon,       // ;
    LeftParen,       // (
    RightParen,      // )
    LeftBracket,     // [
    RightBracket,    // ]
    LeftBrace,       // {
    RightBrace,      // }
    At,              // @
    Bang,            // !
    Question,        // ?
    
    // Comments (preserved for formatting)
    LineComment(String),      // // comment
    BlockComment(String),     // /* comment */
    
    // Whitespace (preserved for formatting decisions)
    Whitespace(String),
    Newline,
    
    // End of file
    Eof,
    
    // Error token
    Invalid(String),
}

impl TokenKind {
    /// Check if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::And
                | TokenKind::As
                | TokenKind::Each
                | TokenKind::Else
                | TokenKind::Error
                | TokenKind::False
                | TokenKind::If
                | TokenKind::In
                | TokenKind::Is
                | TokenKind::Let
                | TokenKind::Meta
                | TokenKind::Not
                | TokenKind::Null
                | TokenKind::Or
                | TokenKind::Otherwise
                | TokenKind::Section
                | TokenKind::Shared
                | TokenKind::Then
                | TokenKind::True
                | TokenKind::Try
                | TokenKind::Type
        )
    }
    
    /// Check if this token is a binary operator
    pub fn is_binary_operator(&self) -> bool {
        matches!(
            self,
            TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Ampersand
                | TokenKind::Equal
                | TokenKind::NotEqual
                | TokenKind::LessThan
                | TokenKind::LessThanEqual
                | TokenKind::GreaterThan
                | TokenKind::GreaterThanEqual
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::QuestionQuestion
                | TokenKind::Meta
        )
    }
    
    /// Check if this token is trivia (whitespace or comment)
    pub fn is_trivia(&self) -> bool {
        matches!(
            self,
            TokenKind::Whitespace(_)
                | TokenKind::Newline
                | TokenKind::LineComment(_)
                | TokenKind::BlockComment(_)
        )
    }
    
    /// Get operator precedence (higher = binds tighter)
    pub fn precedence(&self) -> Option<u8> {
        match self {
            TokenKind::Meta => Some(1),
            TokenKind::QuestionQuestion => Some(2),
            TokenKind::Or => Some(3),
            TokenKind::And => Some(4),
            TokenKind::Equal
            | TokenKind::NotEqual
            | TokenKind::LessThan
            | TokenKind::LessThanEqual
            | TokenKind::GreaterThan
            | TokenKind::GreaterThanEqual => Some(5),
            TokenKind::Ampersand => Some(6),
            TokenKind::Plus | TokenKind::Minus => Some(7),
            TokenKind::Star | TokenKind::Slash => Some(8),
            _ => None,
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Null => write!(f, "null"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::Text(s) => write!(f, "\"{}\"", s),
            TokenKind::Identifier(s) => write!(f, "{}", s),
            TokenKind::QuotedIdentifier(s) => write!(f, "#\"{}\"", s),
            TokenKind::And => write!(f, "and"),
            TokenKind::As => write!(f, "as"),
            TokenKind::Each => write!(f, "each"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Error => write!(f, "error"),
            TokenKind::If => write!(f, "if"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Is => write!(f, "is"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Meta => write!(f, "meta"),
            TokenKind::Not => write!(f, "not"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Otherwise => write!(f, "otherwise"),
            TokenKind::Section => write!(f, "section"),
            TokenKind::Shared => write!(f, "shared"),
            TokenKind::Then => write!(f, "then"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::HashBinary => write!(f, "#binary"),
            TokenKind::HashDate => write!(f, "#date"),
            TokenKind::HashDatetime => write!(f, "#datetime"),
            TokenKind::HashDatetimezone => write!(f, "#datetimezone"),
            TokenKind::HashDuration => write!(f, "#duration"),
            TokenKind::HashInfinity => write!(f, "#infinity"),
            TokenKind::HashNan => write!(f, "#nan"),
            TokenKind::HashSections => write!(f, "#sections"),
            TokenKind::HashShared => write!(f, "#shared"),
            TokenKind::HashTable => write!(f, "#table"),
            TokenKind::HashTime => write!(f, "#time"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::Equal => write!(f, "="),
            TokenKind::NotEqual => write!(f, "<>"),
            TokenKind::LessThan => write!(f, "<"),
            TokenKind::LessThanEqual => write!(f, "<="),
            TokenKind::GreaterThan => write!(f, ">"),
            TokenKind::GreaterThanEqual => write!(f, ">="),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::QuestionQuestion => write!(f, "??"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::DotDotDot => write!(f, "..."),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::At => write!(f, "@"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::Question => write!(f, "?"),
            TokenKind::LineComment(s) => write!(f, "//{}", s),
            TokenKind::BlockComment(s) => write!(f, "/*{}*/", s),
            TokenKind::Whitespace(s) => write!(f, "{}", s),
            TokenKind::Newline => writeln!(f),
            TokenKind::Eof => write!(f, ""),
            TokenKind::Invalid(s) => write!(f, "{}", s),
        }
    }
}
