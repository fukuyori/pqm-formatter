//! Abstract Syntax Tree definitions for Power Query M language

use crate::token::Span;

/// Root document node
#[derive(Debug, Clone)]
pub struct Document {
    pub expression: Expr,
    pub span: Span,
}

/// Expression node
#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
    /// Leading trivia (comments, whitespace before this expression)
    pub leading_trivia: Vec<Trivia>,
    /// Trailing trivia (comments, whitespace after this expression)
    pub trailing_trivia: Vec<Trivia>,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self {
            kind,
            span,
            leading_trivia: Vec::new(),
            trailing_trivia: Vec::new(),
        }
    }
    
    pub fn with_leading_trivia(mut self, trivia: Vec<Trivia>) -> Self {
        self.leading_trivia = trivia;
        self
    }
    
    pub fn with_trailing_trivia(mut self, trivia: Vec<Trivia>) -> Self {
        self.trailing_trivia = trivia;
        self
    }
}

/// Expression kinds
#[derive(Debug, Clone)]
pub enum ExprKind {
    // Literals
    Null,
    Logical(bool),
    Number(f64),
    Text(String),
    
    // Identifiers
    Identifier(String),
    QuotedIdentifier(String),
    
    // Let expression
    Let(LetExpr),
    
    // If expression
    If(Box<IfExpr>),
    
    // Try expression
    Try(Box<TryExpr>),
    
    // Error expression
    Error(Box<Expr>),
    
    // Each expression (sugar for (x) => ...)
    Each(Box<Expr>),
    
    // Function definition
    Function(Box<FunctionExpr>),
    
    // Function call
    FunctionCall(Box<FunctionCallExpr>),
    
    // Record literal
    Record(RecordExpr),
    
    // List literal
    List(ListExpr),
    
    // Field access: record[field]
    FieldAccess(Box<FieldAccessExpr>),
    
    // Item access: list{index}
    ItemAccess(Box<ItemAccessExpr>),
    
    // Binary operation
    Binary(Box<BinaryExpr>),
    
    // Unary operation
    Unary(Box<UnaryExpr>),
    
    // Parenthesized expression
    Parenthesized(Box<Expr>),
    
    // Type expression
    Type(Box<TypeExpr>),
    
    // Metadata expression: expr meta record
    Metadata(Box<MetadataExpr>),
    
    // Not-standard-alone: underscore _ in each expressions
    Underscore,
    
    // Built-in constructors
    HashTable(Box<HashTableExpr>),
    HashDate(Box<HashDateExpr>),
    HashTime(Box<HashTimeExpr>),
    HashDatetime(Box<HashDatetimeExpr>),
    HashDatetimezone(Box<HashDatetimezoneExpr>),
    HashDuration(Box<HashDurationExpr>),
}

/// Let expression: let bindings in body
#[derive(Debug, Clone)]
pub struct LetExpr {
    pub bindings: Vec<Binding>,
    pub body: Box<Expr>,
}

/// Variable binding in let expression
#[derive(Debug, Clone)]
pub struct Binding {
    pub name: Identifier,
    pub value: Expr,
    pub span: Span,
    pub leading_trivia: Vec<Trivia>,
    pub trailing_trivia: Vec<Trivia>,
}

/// Identifier (normal or quoted)
#[derive(Debug, Clone)]
pub struct Identifier {
    pub name: String,
    pub quoted: bool,
    pub span: Span,
}

impl Identifier {
    pub fn new(name: String, quoted: bool, span: Span) -> Self {
        Self { name, quoted, span }
    }
}

/// If expression: if cond then true_expr else false_expr
#[derive(Debug, Clone)]
pub struct IfExpr {
    pub condition: Expr,
    pub then_branch: Expr,
    pub else_branch: Expr,
}

/// Try expression: try expr otherwise fallback
#[derive(Debug, Clone)]
pub struct TryExpr {
    pub expr: Expr,
    pub otherwise: Option<Expr>,
}

/// Function expression: (params) => body
#[derive(Debug, Clone)]
pub struct FunctionExpr {
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeAnnotation>,
    pub body: Expr,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: Identifier,
    pub type_annotation: Option<TypeAnnotation>,
    pub optional: bool,
    pub span: Span,
}

/// Type annotation
#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    pub kind: TypeKind,
    pub span: Span,
}

/// Type kinds
#[derive(Debug, Clone)]
pub enum TypeKind {
    Any,
    None,
    Null,
    Logical,
    Number,
    Time,
    Date,
    DateTime,
    DateTimeZone,
    Duration,
    Text,
    Binary,
    Type,
    List(Option<Box<TypeAnnotation>>),
    Record(Vec<FieldType>),
    Table(Vec<FieldType>),
    Function(Vec<TypeAnnotation>, Box<TypeAnnotation>),
    Custom(String),
    Nullable(Box<TypeAnnotation>),
}

/// Field type in record/table types
#[derive(Debug, Clone)]
pub struct FieldType {
    pub name: Identifier,
    pub type_annotation: TypeAnnotation,
    pub optional: bool,
    pub span: Span,
}

/// Function call expression
#[derive(Debug, Clone)]
pub struct FunctionCallExpr {
    pub function: Expr,
    pub arguments: Vec<Expr>,
}

/// Record expression: [field1 = value1, field2 = value2]
#[derive(Debug, Clone)]
pub struct RecordExpr {
    pub fields: Vec<RecordField>,
}

/// Record field
#[derive(Debug, Clone)]
pub struct RecordField {
    pub name: Identifier,
    pub value: Expr,
    pub span: Span,
    pub leading_trivia: Vec<Trivia>,
    pub trailing_trivia: Vec<Trivia>,
}

/// List expression: {item1, item2, item3}
#[derive(Debug, Clone)]
pub struct ListExpr {
    pub items: Vec<Expr>,
}

/// Field access expression: expr[field]
#[derive(Debug, Clone)]
pub struct FieldAccessExpr {
    pub expr: Expr,
    pub field: Identifier,
}

/// Item access expression: expr{index}
#[derive(Debug, Clone)]
pub struct ItemAccessExpr {
    pub expr: Expr,
    pub index: Expr,
}

/// Binary expression
#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Expr,
    pub operator: BinaryOp,
    pub right: Expr,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    
    // Comparison
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    
    // Logical
    And,
    Or,
    
    // Combination
    Concatenate,  // &
    
    // Null coalescing
    Coalesce,     // ??
    
    // Metadata
    Meta,
    
    // Type
    Is,
    As,
}

impl BinaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Equal => "=",
            BinaryOp::NotEqual => "<>",
            BinaryOp::LessThan => "<",
            BinaryOp::LessThanOrEqual => "<=",
            BinaryOp::GreaterThan => ">",
            BinaryOp::GreaterThanOrEqual => ">=",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
            BinaryOp::Concatenate => "&",
            BinaryOp::Coalesce => "??",
            BinaryOp::Meta => "meta",
            BinaryOp::Is => "is",
            BinaryOp::As => "as",
        }
    }
    
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Meta => 1,
            BinaryOp::Coalesce => 2,
            BinaryOp::Or => 3,
            BinaryOp::And => 4,
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::LessThan
            | BinaryOp::LessThanOrEqual
            | BinaryOp::GreaterThan
            | BinaryOp::GreaterThanOrEqual => 5,
            BinaryOp::Is | BinaryOp::As => 5,
            BinaryOp::Concatenate => 6,
            BinaryOp::Add | BinaryOp::Subtract => 7,
            BinaryOp::Multiply | BinaryOp::Divide => 8,
        }
    }
}

/// Unary expression
#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
    pub operand: Expr,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,     // -
    Positive,   // +
    Not,        // not
}

impl UnaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            UnaryOp::Negate => "-",
            UnaryOp::Positive => "+",
            UnaryOp::Not => "not",
        }
    }
}

/// Type expression
#[derive(Debug, Clone)]
pub struct TypeExpr {
    pub type_annotation: TypeAnnotation,
}

/// Metadata expression: expr meta record
#[derive(Debug, Clone)]
pub struct MetadataExpr {
    pub expr: Expr,
    pub metadata: Expr,
}

/// #table constructor
#[derive(Debug, Clone)]
pub struct HashTableExpr {
    pub columns: Expr,
    pub rows: Expr,
}

/// #date constructor
#[derive(Debug, Clone)]
pub struct HashDateExpr {
    pub year: Expr,
    pub month: Expr,
    pub day: Expr,
}

/// #time constructor
#[derive(Debug, Clone)]
pub struct HashTimeExpr {
    pub hour: Expr,
    pub minute: Expr,
    pub second: Expr,
}

/// #datetime constructor
#[derive(Debug, Clone)]
pub struct HashDatetimeExpr {
    pub year: Expr,
    pub month: Expr,
    pub day: Expr,
    pub hour: Expr,
    pub minute: Expr,
    pub second: Expr,
}

/// #datetimezone constructor
#[derive(Debug, Clone)]
pub struct HashDatetimezoneExpr {
    pub year: Expr,
    pub month: Expr,
    pub day: Expr,
    pub hour: Expr,
    pub minute: Expr,
    pub second: Expr,
    pub offset_hours: Expr,
    pub offset_minutes: Expr,
}

/// #duration constructor
#[derive(Debug, Clone)]
pub struct HashDurationExpr {
    pub days: Expr,
    pub hours: Expr,
    pub minutes: Expr,
    pub seconds: Expr,
}

/// Trivia (preserved comments and whitespace)
#[derive(Debug, Clone)]
pub enum Trivia {
    Whitespace(String),
    Newline,
    LineComment(String),
    BlockComment(String),
}

impl Trivia {
    pub fn is_comment(&self) -> bool {
        matches!(self, Trivia::LineComment(_) | Trivia::BlockComment(_))
    }
    
    pub fn is_newline(&self) -> bool {
        matches!(self, Trivia::Newline)
    }
}
