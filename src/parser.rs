//! Parser for Power Query M language

use crate::ast::*;
use crate::token::{Span, Token, TokenKind};

/// Parser errors
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

/// Parser for Power Query M
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
}

impl Parser {
    /// Create a new parser from tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }
    
    /// Parse the document
    pub fn parse(&mut self) -> Result<Document, Vec<ParseError>> {
        self.skip_trivia();
        let start_span = self.current_span();
        
        let expression = self.parse_expression()?;
        
        self.skip_trivia();
        if !self.is_at_end() {
            self.errors.push(ParseError::new(
                "Unexpected token after expression",
                self.current_span(),
            ));
        }
        
        if self.errors.is_empty() {
            Ok(Document {
                expression,
                span: start_span.merge(self.current_span()),
            })
        } else {
            Err(self.errors.clone())
        }
    }
    
    /// Parse an expression
    fn parse_expression(&mut self) -> Result<Expr, Vec<ParseError>> {
        self.parse_or_expression()
    }
    
    /// Parse or expression (lowest precedence binary)
    fn parse_or_expression(&mut self) -> Result<Expr, Vec<ParseError>> {
        self.parse_binary_expression(0)
    }
    
    /// Parse binary expression with precedence climbing
    fn parse_binary_expression(&mut self, min_prec: u8) -> Result<Expr, Vec<ParseError>> {
        let mut left = self.parse_unary_expression()?;
        
        loop {
            self.skip_whitespace_only();  // Don't consume comments here
            
            let op = match self.current_kind() {
                TokenKind::Or => BinaryOp::Or,
                TokenKind::And => BinaryOp::And,
                TokenKind::Equal => BinaryOp::Equal,
                TokenKind::NotEqual => BinaryOp::NotEqual,
                TokenKind::LessThan => BinaryOp::LessThan,
                TokenKind::LessThanEqual => BinaryOp::LessThanOrEqual,
                TokenKind::GreaterThan => BinaryOp::GreaterThan,
                TokenKind::GreaterThanEqual => BinaryOp::GreaterThanOrEqual,
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                TokenKind::Ampersand => BinaryOp::Concatenate,
                TokenKind::QuestionQuestion => BinaryOp::Coalesce,
                TokenKind::Meta => BinaryOp::Meta,
                TokenKind::Is => BinaryOp::Is,
                TokenKind::As => BinaryOp::As,
                // If we see a comment, don't consume it - break out
                TokenKind::LineComment(_) | TokenKind::BlockComment(_) => break,
                _ => break,
            };
            
            let prec = op.precedence();
            if prec < min_prec {
                break;
            }
            
            self.advance(); // consume operator
            self.skip_trivia();
            
            let right = self.parse_binary_expression(prec + 1)?;
            
            let span = left.span.merge(right.span);
            left = Expr::new(
                ExprKind::Binary(Box::new(BinaryExpr {
                    left,
                    operator: op,
                    right,
                })),
                span,
            );
        }
        
        Ok(left)
    }
    
    /// Parse unary expression
    fn parse_unary_expression(&mut self) -> Result<Expr, Vec<ParseError>> {
        self.skip_trivia();
        let span = self.current_span();
        
        match self.current_kind() {
            TokenKind::Not => {
                self.advance();
                self.skip_trivia();
                let operand = self.parse_unary_expression()?;
                let end_span = operand.span;
                Ok(Expr::new(
                    ExprKind::Unary(Box::new(UnaryExpr {
                        operator: UnaryOp::Not,
                        operand,
                    })),
                    span.merge(end_span),
                ))
            }
            TokenKind::Minus => {
                self.advance();
                self.skip_trivia();
                let operand = self.parse_unary_expression()?;
                let end_span = operand.span;
                Ok(Expr::new(
                    ExprKind::Unary(Box::new(UnaryExpr {
                        operator: UnaryOp::Negate,
                        operand,
                    })),
                    span.merge(end_span),
                ))
            }
            TokenKind::Plus => {
                self.advance();
                self.skip_trivia();
                let operand = self.parse_unary_expression()?;
                let end_span = operand.span;
                Ok(Expr::new(
                    ExprKind::Unary(Box::new(UnaryExpr {
                        operator: UnaryOp::Positive,
                        operand,
                    })),
                    span.merge(end_span),
                ))
            }
            _ => self.parse_postfix_expression(),
        }
    }
    
    /// Parse postfix expression (field access, item access, function call)
    fn parse_postfix_expression(&mut self) -> Result<Expr, Vec<ParseError>> {
        let mut expr = self.parse_primary_expression()?;
        
        loop {
            self.skip_whitespace_only();  // Don't consume comments here
            match self.current_kind() {
                TokenKind::LeftBracket => {
                    self.advance();
                    self.skip_trivia();
                    
                    // Field access
                    let field = self.parse_identifier()?;
                    
                    self.skip_trivia();
                    self.expect(TokenKind::RightBracket)?;
                    
                    let span = expr.span.merge(self.prev_span());
                    expr = Expr::new(
                        ExprKind::FieldAccess(Box::new(FieldAccessExpr { expr, field })),
                        span,
                    );
                }
                TokenKind::LeftBrace => {
                    self.advance();
                    self.skip_trivia();
                    
                    // Item access
                    let index = self.parse_expression()?;
                    
                    self.skip_trivia();
                    self.expect(TokenKind::RightBrace)?;
                    
                    let span = expr.span.merge(self.prev_span());
                    expr = Expr::new(
                        ExprKind::ItemAccess(Box::new(ItemAccessExpr { expr, index })),
                        span,
                    );
                }
                TokenKind::LeftParen => {
                    self.advance();
                    self.skip_trivia();
                    
                    // Function call
                    let arguments = self.parse_argument_list()?;
                    
                    self.skip_trivia();
                    self.expect(TokenKind::RightParen)?;
                    
                    let span = expr.span.merge(self.prev_span());
                    expr = Expr::new(
                        ExprKind::FunctionCall(Box::new(FunctionCallExpr {
                            function: expr,
                            arguments,
                        })),
                        span,
                    );
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    /// Parse primary expression
    fn parse_primary_expression(&mut self) -> Result<Expr, Vec<ParseError>> {
        self.skip_trivia();
        let span = self.current_span();
        
        match self.current_kind() {
            TokenKind::Null => {
                self.advance();
                Ok(Expr::new(ExprKind::Null, span))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::new(ExprKind::Logical(true), span))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::new(ExprKind::Logical(false), span))
            }
            TokenKind::Number(n) => {
                let n = n;
                self.advance();
                Ok(Expr::new(ExprKind::Number(n), span))
            }
            TokenKind::Text(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::new(ExprKind::Text(s), span))
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                
                // Special case: underscore in each expressions
                if name == "_" {
                    return Ok(Expr::new(ExprKind::Underscore, span));
                }
                
                // Dotted identifiers are already handled by the lexer
                Ok(Expr::new(ExprKind::Identifier(name), span))
            }
            TokenKind::QuotedIdentifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::new(ExprKind::QuotedIdentifier(name), span))
            }
            TokenKind::At => {
                self.advance();
                self.skip_trivia();
                let ident = self.parse_identifier()?;
                let end_span = ident.span;
                // @ prefix for inclusive identifier reference
                Ok(Expr::new(
                    ExprKind::Identifier(format!("@{}", ident.name)),
                    span.merge(end_span),
                ))
            }
            TokenKind::Let => self.parse_let_expression(),
            TokenKind::If => self.parse_if_expression(),
            TokenKind::Try => self.parse_try_expression(),
            TokenKind::Error => self.parse_error_expression(),
            TokenKind::Each => self.parse_each_expression(),
            TokenKind::LeftParen => self.parse_parenthesized_or_function(),
            TokenKind::LeftBracket => self.parse_record_expression(),
            TokenKind::LeftBrace => self.parse_list_expression(),
            TokenKind::Type => self.parse_type_expression(),
            TokenKind::HashTable => self.parse_hash_table(),
            TokenKind::HashDate => self.parse_hash_date(),
            TokenKind::HashTime => self.parse_hash_time(),
            TokenKind::HashDatetime => self.parse_hash_datetime(),
            TokenKind::HashDatetimezone => self.parse_hash_datetimezone(),
            TokenKind::HashDuration => self.parse_hash_duration(),
            TokenKind::HashInfinity => {
                self.advance();
                Ok(Expr::new(ExprKind::Number(f64::INFINITY), span))
            }
            TokenKind::HashNan => {
                self.advance();
                Ok(Expr::new(ExprKind::Number(f64::NAN), span))
            }
            _ => {
                let msg = format!("Unexpected token: {:?}", self.current_kind());
                self.errors.push(ParseError::new(&msg, span));
                Err(self.errors.clone())
            }
        }
    }
    
    /// Parse let expression
    fn parse_let_expression(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance(); // consume 'let'
        self.skip_whitespace_only(); // Only skip whitespace, not comments
        
        let mut bindings = Vec::new();
        
        // Parse bindings
        loop {
            // Collect leading trivia (comments before binding)
            let leading_trivia = self.collect_trivia();
            
            if self.current_kind() == TokenKind::In {
                break;
            }
            
            let mut binding = self.parse_binding()?;
            binding.leading_trivia = self.tokens_to_trivia(&leading_trivia);
            bindings.push(binding);
            
            // Collect trailing trivia after binding value
            let trailing = self.collect_trivia();
            if let Some(last) = bindings.last_mut() {
                last.trailing_trivia = self.tokens_to_trivia(&trailing);
            }
            
            if self.current_kind() == TokenKind::Comma {
                self.advance();
                self.skip_whitespace_only(); // Skip whitespace after comma
            } else {
                break;
            }
        }
        
        self.skip_trivia();
        self.expect(TokenKind::In)?;
        self.skip_trivia();
        
        let body = self.parse_expression()?;
        let end_span = body.span;
        
        Ok(Expr::new(
            ExprKind::Let(LetExpr {
                bindings,
                body: Box::new(body),
            }),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse a binding
    fn parse_binding(&mut self) -> Result<Binding, Vec<ParseError>> {
        let start_span = self.current_span();
        let name = self.parse_identifier()?;
        
        self.skip_trivia();
        self.expect(TokenKind::Equal)?;
        self.skip_trivia();
        
        let value = self.parse_expression()?;
        let end_span = value.span;
        
        Ok(Binding {
            name,
            value,
            span: start_span.merge(end_span),
            leading_trivia: Vec::new(),
            trailing_trivia: Vec::new(),
        })
    }
    
    /// Parse if expression
    fn parse_if_expression(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance(); // consume 'if'
        self.skip_trivia();
        
        let condition = self.parse_expression()?;
        
        self.skip_trivia();
        self.expect(TokenKind::Then)?;
        self.skip_trivia();
        
        let then_branch = self.parse_expression()?;
        
        self.skip_trivia();
        self.expect(TokenKind::Else)?;
        self.skip_trivia();
        
        let else_branch = self.parse_expression()?;
        let end_span = else_branch.span;
        
        Ok(Expr::new(
            ExprKind::If(Box::new(IfExpr {
                condition,
                then_branch,
                else_branch,
            })),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse try expression
    fn parse_try_expression(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance(); // consume 'try'
        self.skip_trivia();
        
        let expr = self.parse_expression()?;
        
        self.skip_trivia();
        let otherwise = if self.current_kind() == TokenKind::Otherwise {
            self.advance();
            self.skip_trivia();
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        let end_span = otherwise.as_ref().map(|e| e.span).unwrap_or(expr.span);
        
        Ok(Expr::new(
            ExprKind::Try(Box::new(TryExpr { expr, otherwise })),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse error expression
    fn parse_error_expression(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance(); // consume 'error'
        self.skip_trivia();
        
        let expr = self.parse_expression()?;
        let end_span = expr.span;
        
        Ok(Expr::new(
            ExprKind::Error(Box::new(expr)),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse each expression
    fn parse_each_expression(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance(); // consume 'each'
        self.skip_trivia();
        
        let expr = self.parse_expression()?;
        let end_span = expr.span;
        
        Ok(Expr::new(
            ExprKind::Each(Box::new(expr)),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse parenthesized expression or function definition
    fn parse_parenthesized_or_function(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance(); // consume '('
        self.skip_trivia();
        
        // Check if this is a function definition
        // Save position for backtracking
        let saved_pos = self.pos;
        
        // Try to parse as parameters
        if self.is_function_definition() {
            self.pos = saved_pos;
            return self.parse_function_expression(start_span);
        }
        
        // It's a parenthesized expression
        self.pos = saved_pos;
        let expr = self.parse_expression()?;
        
        self.skip_trivia();
        self.expect(TokenKind::RightParen)?;
        let end_span = self.prev_span();
        
        Ok(Expr::new(
            ExprKind::Parenthesized(Box::new(expr)),
            start_span.merge(end_span),
        ))
    }
    
    /// Check if current position starts a function definition
    fn is_function_definition(&mut self) -> bool {
        // Look for pattern: (params) => or (params) as type =>
        // We're positioned after the opening '('
        let mut depth = 1;
        let start = self.pos;
        
        while !self.is_at_end() && depth > 0 {
            match self.current_kind() {
                TokenKind::LeftParen => depth += 1,
                TokenKind::RightParen => depth -= 1,
                _ => {}
            }
            if depth > 0 {
                self.advance();
            }
        }
        
        if depth == 0 {
            self.advance(); // consume ')'
            self.skip_trivia();
            
            // Check for => directly or as TYPE =>
            let is_func = match self.current_kind() {
                TokenKind::FatArrow => true,
                TokenKind::As => {
                    // Skip past 'as' and the type, then check for =>
                    self.advance(); // consume 'as'
                    self.skip_trivia();
                    // Skip the type (could be identifier, list type, etc.)
                    self.skip_type_for_lookahead();
                    self.skip_trivia();
                    self.current_kind() == TokenKind::FatArrow
                }
                _ => false,
            };
            
            self.pos = start;
            return is_func;
        }
        
        self.pos = start;
        false
    }
    
    /// Skip a type expression during lookahead (for is_function_definition)
    fn skip_type_for_lookahead(&mut self) {
        match self.current_kind() {
            TokenKind::Identifier(_) => {
                self.advance();
            }
            TokenKind::LeftBrace => {
                // List type: {type}
                self.advance();
                let mut depth = 1;
                while !self.is_at_end() && depth > 0 {
                    match self.current_kind() {
                        TokenKind::LeftBrace => depth += 1,
                        TokenKind::RightBrace => depth -= 1,
                        _ => {}
                    }
                    self.advance();
                }
            }
            TokenKind::LeftBracket => {
                // Record type: [field = type]
                self.advance();
                let mut depth = 1;
                while !self.is_at_end() && depth > 0 {
                    match self.current_kind() {
                        TokenKind::LeftBracket => depth += 1,
                        TokenKind::RightBracket => depth -= 1,
                        _ => {}
                    }
                    self.advance();
                }
            }
            _ => {}
        }
    }
    
    /// Parse function expression
    fn parse_function_expression(&mut self, start_span: Span) -> Result<Expr, Vec<ParseError>> {
        let parameters = self.parse_parameter_list()?;
        
        self.skip_trivia();
        self.expect(TokenKind::RightParen)?;
        
        // Optional return type
        self.skip_trivia();
        let return_type = if self.current_kind() == TokenKind::As {
            self.advance();
            self.skip_trivia();
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        
        self.skip_trivia();
        self.expect(TokenKind::FatArrow)?;
        self.skip_trivia();
        
        let body = self.parse_expression()?;
        let end_span = body.span;
        
        Ok(Expr::new(
            ExprKind::Function(Box::new(FunctionExpr {
                parameters,
                return_type,
                body,
            })),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse parameter list
    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>, Vec<ParseError>> {
        let mut params = Vec::new();
        
        while self.current_kind() != TokenKind::RightParen && !self.is_at_end() {
            let param = self.parse_parameter()?;
            params.push(param);
            
            self.skip_trivia();
            if self.current_kind() == TokenKind::Comma {
                self.advance();
                self.skip_trivia();
            } else {
                break;
            }
        }
        
        Ok(params)
    }
    
    /// Parse a parameter
    fn parse_parameter(&mut self) -> Result<Parameter, Vec<ParseError>> {
        let start_span = self.current_span();
        
        // Optional 'optional' keyword
        let optional = if self.current_kind() == TokenKind::Identifier("optional".to_string()) {
            self.advance();
            self.skip_trivia();
            true
        } else {
            false
        };
        
        let name = self.parse_identifier()?;
        
        // Optional type annotation
        self.skip_trivia();
        let type_annotation = if self.current_kind() == TokenKind::As {
            self.advance();
            self.skip_trivia();
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        
        let end_span = type_annotation
            .as_ref()
            .map(|t| t.span)
            .unwrap_or(name.span);
        
        Ok(Parameter {
            name,
            type_annotation,
            optional,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse type annotation
    fn parse_type_annotation(&mut self) -> Result<TypeAnnotation, Vec<ParseError>> {
        let start_span = self.current_span();
        
        let kind = match self.current_kind() {
            TokenKind::Identifier(name) => {
                let name_str = name.clone();
                self.advance();
                
                match name_str.as_str() {
                    "any" => TypeKind::Any,
                    "none" => TypeKind::None,
                    "null" => TypeKind::Null,
                    "logical" => TypeKind::Logical,
                    "number" => TypeKind::Number,
                    "time" => TypeKind::Time,
                    "date" => TypeKind::Date,
                    "datetime" => TypeKind::DateTime,
                    "datetimezone" => TypeKind::DateTimeZone,
                    "duration" => TypeKind::Duration,
                    "text" => TypeKind::Text,
                    "binary" => TypeKind::Binary,
                    "type" => TypeKind::Type,
                    "list" => {
                        self.skip_trivia();
                        if self.current_kind() == TokenKind::LeftBrace {
                            self.advance();
                            self.skip_trivia();
                            let inner = if self.current_kind() != TokenKind::RightBrace {
                                Some(Box::new(self.parse_type_annotation()?))
                            } else {
                                None
                            };
                            self.skip_trivia();
                            self.expect(TokenKind::RightBrace)?;
                            TypeKind::List(inner)
                        } else {
                            TypeKind::List(None)
                        }
                    }
                    "record" => {
                        self.skip_trivia();
                        if self.current_kind() == TokenKind::LeftBracket {
                            let fields = self.parse_type_field_list()?;
                            TypeKind::Record(fields)
                        } else {
                            TypeKind::Record(Vec::new())
                        }
                    }
                    "table" => {
                        self.skip_trivia();
                        if self.current_kind() == TokenKind::LeftBracket {
                            let fields = self.parse_type_field_list()?;
                            TypeKind::Table(fields)
                        } else {
                            TypeKind::Table(Vec::new())
                        }
                    }
                    "function" => TypeKind::Function(Vec::new(), Box::new(TypeAnnotation {
                        kind: TypeKind::Any,
                        span: start_span,
                    })),
                    "nullable" => {
                        self.skip_trivia();
                        let inner = self.parse_type_annotation()?;
                        return Ok(TypeAnnotation {
                            span: start_span.merge(inner.span),
                            kind: TypeKind::Nullable(Box::new(inner)),
                        });
                    }
                    _ => TypeKind::Custom(name_str),
                }
            }
            TokenKind::LeftBrace => {
                self.advance();
                self.skip_trivia();
                
                let inner = if self.current_kind() != TokenKind::RightBrace {
                    Some(Box::new(self.parse_type_annotation()?))
                } else {
                    None
                };
                
                self.skip_trivia();
                self.expect(TokenKind::RightBrace)?;
                
                TypeKind::List(inner)
            }
            TokenKind::LeftBracket => {
                // Record type: [Field1 = type, Field2 = type]
                let fields = self.parse_type_field_list()?;
                TypeKind::Record(fields)
            }
            _ => {
                let msg = format!("Expected type, found {:?}", self.current_kind());
                self.errors.push(ParseError::new(&msg, start_span));
                return Err(self.errors.clone());
            }
        };
        
        Ok(TypeAnnotation {
            kind,
            span: start_span.merge(self.prev_span()),
        })
    }
    
    /// Parse type field list: [Field1 = type, Field2 = type, ...]
    fn parse_type_field_list(&mut self) -> Result<Vec<FieldType>, Vec<ParseError>> {
        self.advance(); // consume '['
        self.skip_trivia();
        
        let mut fields = Vec::new();
        
        while self.current_kind() != TokenKind::RightBracket && !self.is_at_end() {
            let field_start = self.current_span();
            
            // Check for 'optional' keyword
            let optional = if let TokenKind::Identifier(name) = self.current_kind() {
                if name == "optional" {
                    self.advance();
                    self.skip_trivia();
                    true
                } else {
                    false
                }
            } else {
                false
            };
            
            // Field name
            let name = self.parse_identifier()?;
            
            self.skip_trivia();
            self.expect(TokenKind::Equal)?;
            self.skip_trivia();
            
            // Field type
            let type_annotation = self.parse_type_annotation()?;
            
            fields.push(FieldType {
                name,
                type_annotation,
                optional,
                span: field_start.merge(self.prev_span()),
            });
            
            self.skip_trivia();
            if self.current_kind() == TokenKind::Comma {
                self.advance();
                self.skip_trivia();
            } else {
                break;
            }
        }
        
        self.skip_trivia();
        self.expect(TokenKind::RightBracket)?;
        
        Ok(fields)
    }
    
    /// Parse record expression or field projection
    fn parse_record_expression(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance(); // consume '['
        self.skip_trivia();
        
        // Empty record
        if self.current_kind() == TokenKind::RightBracket {
            self.advance();
            return Ok(Expr::new(
                ExprKind::Record(RecordExpr { fields: Vec::new() }),
                start_span.merge(self.prev_span()),
            ));
        }
        
        // Check if this is a record literal or field projection
        // Look ahead: if we see "identifier =" it's a record, otherwise field projection
        let is_record = self.is_record_literal();
        
        if is_record {
            self.parse_record_fields(start_span)
        } else {
            self.parse_field_projection(start_span)
        }
    }
    
    /// Check if current position starts a record literal (has "identifier =")
    fn is_record_literal(&mut self) -> bool {
        let saved_pos = self.pos;
        
        // Try to parse identifier
        let result = match self.current_kind() {
            TokenKind::Identifier(_) | TokenKind::QuotedIdentifier(_) => {
                self.advance();
                self.skip_trivia();
                self.current_kind() == TokenKind::Equal
            }
            _ => false,
        };
        
        self.pos = saved_pos;
        result
    }
    
    /// Parse record fields (when we know it's a record literal)
    fn parse_record_fields(&mut self, start_span: Span) -> Result<Expr, Vec<ParseError>> {
        let mut fields = Vec::new();
        
        while self.current_kind() != TokenKind::RightBracket && !self.is_at_end() {
            // Collect leading trivia (comments before field)
            let leading_trivia = self.collect_trivia();
            
            if self.current_kind() == TokenKind::RightBracket {
                break;
            }
            
            let mut field = self.parse_record_field()?;
            field.leading_trivia = self.tokens_to_trivia(&leading_trivia);
            
            // Collect trailing trivia (comments after field value)
            let trailing_trivia = self.collect_trivia();
            field.trailing_trivia = self.tokens_to_trivia(&trailing_trivia);
            
            fields.push(field);
            
            if self.current_kind() == TokenKind::Comma {
                self.advance();
                self.skip_whitespace_only();
            } else {
                break;
            }
        }
        
        self.skip_trivia();
        self.expect(TokenKind::RightBracket)?;
        let end_span = self.prev_span();
        
        Ok(Expr::new(
            ExprKind::Record(RecordExpr { fields }),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse field projection: [field1, field2, ...]
    fn parse_field_projection(&mut self, start_span: Span) -> Result<Expr, Vec<ParseError>> {
        // Field projection is syntactic sugar for record field access
        // [A, B] on a record selects fields A and B
        // For now, we treat single field [A] as implicit field access on _
        
        let mut fields = Vec::new();
        
        while self.current_kind() != TokenKind::RightBracket && !self.is_at_end() {
            let ident = self.parse_identifier()?;
            fields.push(ident);
            
            self.skip_trivia();
            if self.current_kind() == TokenKind::Comma {
                self.advance();
                self.skip_trivia();
            } else {
                break;
            }
        }
        
        self.skip_trivia();
        self.expect(TokenKind::RightBracket)?;
        let end_span = self.prev_span();
        
        // Single field: treat as field access on implicit _
        // Multiple fields: treat as field projection (record with selected fields)
        if fields.len() == 1 {
            let field = fields.into_iter().next().unwrap();
            Ok(Expr::new(
                ExprKind::FieldAccess(Box::new(FieldAccessExpr {
                    expr: Expr::new(ExprKind::Underscore, start_span),
                    field,
                })),
                start_span.merge(end_span),
            ))
        } else {
            // Multiple fields - create a record projection
            // This is simplified; full implementation would be more complex
            let record_fields: Vec<RecordField> = fields
                .into_iter()
                .map(|name| RecordField {
                    span: name.span,
                    leading_trivia: Vec::new(),
                    trailing_trivia: Vec::new(),
                    value: Expr::new(
                        ExprKind::FieldAccess(Box::new(FieldAccessExpr {
                            expr: Expr::new(ExprKind::Underscore, name.span),
                            field: name.clone(),
                        })),
                        name.span,
                    ),
                    name,
                })
                .collect();
            
            Ok(Expr::new(
                ExprKind::Record(RecordExpr { fields: record_fields }),
                start_span.merge(end_span),
            ))
        }
    }
    
    /// Parse record field
    fn parse_record_field(&mut self) -> Result<RecordField, Vec<ParseError>> {
        let start_span = self.current_span();
        let name = self.parse_generalized_identifier()?;
        
        self.skip_whitespace_only();  // Don't skip comments here
        self.expect(TokenKind::Equal)?;
        self.skip_whitespace_only();  // Don't skip comments here
        
        let value = self.parse_expression()?;
        let end_span = value.span;
        
        Ok(RecordField {
            name,
            value,
            span: start_span.merge(end_span),
            leading_trivia: Vec::new(),
            trailing_trivia: Vec::new(),
        })
    }
    
    /// Parse list expression
    fn parse_list_expression(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance(); // consume '{'
        self.skip_trivia();
        
        let mut items = Vec::new();
        
        while self.current_kind() != TokenKind::RightBrace && !self.is_at_end() {
            let item = self.parse_expression()?;
            items.push(item);
            
            self.skip_trivia();
            if self.current_kind() == TokenKind::Comma {
                self.advance();
                self.skip_trivia();
            } else {
                break;
            }
        }
        
        self.skip_trivia();
        self.expect(TokenKind::RightBrace)?;
        let end_span = self.prev_span();
        
        Ok(Expr::new(
            ExprKind::List(ListExpr { items }),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse type expression
    fn parse_type_expression(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance(); // consume 'type'
        self.skip_trivia();
        
        let type_annotation = self.parse_type_annotation()?;
        let end_span = type_annotation.span;
        
        Ok(Expr::new(
            ExprKind::Type(Box::new(TypeExpr { type_annotation })),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse #table constructor
    fn parse_hash_table(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance(); // consume #table
        self.skip_trivia();
        
        self.expect(TokenKind::LeftParen)?;
        self.skip_trivia();
        
        let columns = self.parse_expression()?;
        
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let rows = self.parse_expression()?;
        
        self.skip_trivia();
        self.expect(TokenKind::RightParen)?;
        let end_span = self.prev_span();
        
        Ok(Expr::new(
            ExprKind::HashTable(Box::new(HashTableExpr { columns, rows })),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse #date constructor
    fn parse_hash_date(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance();
        self.skip_trivia();
        
        self.expect(TokenKind::LeftParen)?;
        self.skip_trivia();
        
        let year = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let month = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let day = self.parse_expression()?;
        self.skip_trivia();
        
        self.expect(TokenKind::RightParen)?;
        let end_span = self.prev_span();
        
        Ok(Expr::new(
            ExprKind::HashDate(Box::new(HashDateExpr { year, month, day })),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse #time constructor
    fn parse_hash_time(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance();
        self.skip_trivia();
        
        self.expect(TokenKind::LeftParen)?;
        self.skip_trivia();
        
        let hour = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let minute = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let second = self.parse_expression()?;
        self.skip_trivia();
        
        self.expect(TokenKind::RightParen)?;
        let end_span = self.prev_span();
        
        Ok(Expr::new(
            ExprKind::HashTime(Box::new(HashTimeExpr { hour, minute, second })),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse #datetime constructor
    fn parse_hash_datetime(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance();
        self.skip_trivia();
        
        self.expect(TokenKind::LeftParen)?;
        self.skip_trivia();
        
        let year = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let month = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let day = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let hour = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let minute = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let second = self.parse_expression()?;
        self.skip_trivia();
        
        self.expect(TokenKind::RightParen)?;
        let end_span = self.prev_span();
        
        Ok(Expr::new(
            ExprKind::HashDatetime(Box::new(HashDatetimeExpr {
                year,
                month,
                day,
                hour,
                minute,
                second,
            })),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse #datetimezone constructor
    fn parse_hash_datetimezone(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance();
        self.skip_trivia();
        
        self.expect(TokenKind::LeftParen)?;
        self.skip_trivia();
        
        let year = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let month = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let day = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let hour = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let minute = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let second = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let offset_hours = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let offset_minutes = self.parse_expression()?;
        self.skip_trivia();
        
        self.expect(TokenKind::RightParen)?;
        let end_span = self.prev_span();
        
        Ok(Expr::new(
            ExprKind::HashDatetimezone(Box::new(HashDatetimezoneExpr {
                year,
                month,
                day,
                hour,
                minute,
                second,
                offset_hours,
                offset_minutes,
            })),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse #duration constructor
    fn parse_hash_duration(&mut self) -> Result<Expr, Vec<ParseError>> {
        let start_span = self.current_span();
        self.advance();
        self.skip_trivia();
        
        self.expect(TokenKind::LeftParen)?;
        self.skip_trivia();
        
        let days = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let hours = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let minutes = self.parse_expression()?;
        self.skip_trivia();
        self.expect(TokenKind::Comma)?;
        self.skip_trivia();
        
        let seconds = self.parse_expression()?;
        self.skip_trivia();
        
        self.expect(TokenKind::RightParen)?;
        let end_span = self.prev_span();
        
        Ok(Expr::new(
            ExprKind::HashDuration(Box::new(HashDurationExpr {
                days,
                hours,
                minutes,
                seconds,
            })),
            start_span.merge(end_span),
        ))
    }
    
    /// Parse argument list
    fn parse_argument_list(&mut self) -> Result<Vec<Expr>, Vec<ParseError>> {
        let mut args = Vec::new();
        
        while self.current_kind() != TokenKind::RightParen && !self.is_at_end() {
            let arg = self.parse_expression()?;
            args.push(arg);
            
            self.skip_trivia();
            if self.current_kind() == TokenKind::Comma {
                self.advance();
                self.skip_trivia();
            } else {
                break;
            }
        }
        
        Ok(args)
    }
    
    /// Parse identifier
    fn parse_identifier(&mut self) -> Result<Identifier, Vec<ParseError>> {
        let span = self.current_span();
        
        match self.current_kind() {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Identifier::new(name, false, span))
            }
            TokenKind::QuotedIdentifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Identifier::new(name, true, span))
            }
            _ => {
                let msg = format!("Expected identifier, found {:?}", self.current_kind());
                self.errors.push(ParseError::new(&msg, span));
                Err(self.errors.clone())
            }
        }
    }
    
    /// Parse generalized identifier (for record fields)
    fn parse_generalized_identifier(&mut self) -> Result<Identifier, Vec<ParseError>> {
        let span = self.current_span();
        
        match self.current_kind() {
            TokenKind::Identifier(name) => {
                let full_name = name.clone();
                self.advance();
                Ok(Identifier::new(full_name, false, span.merge(self.prev_span())))
            }
            TokenKind::QuotedIdentifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Identifier::new(name, true, span))
            }
            // Allow keywords as field names in records
            TokenKind::Type => {
                self.advance();
                Ok(Identifier::new("type".to_string(), false, span))
            }
            TokenKind::If => {
                self.advance();
                Ok(Identifier::new("if".to_string(), false, span))
            }
            TokenKind::Then => {
                self.advance();
                Ok(Identifier::new("then".to_string(), false, span))
            }
            TokenKind::Else => {
                self.advance();
                Ok(Identifier::new("else".to_string(), false, span))
            }
            TokenKind::Let => {
                self.advance();
                Ok(Identifier::new("let".to_string(), false, span))
            }
            TokenKind::In => {
                self.advance();
                Ok(Identifier::new("in".to_string(), false, span))
            }
            TokenKind::And => {
                self.advance();
                Ok(Identifier::new("and".to_string(), false, span))
            }
            TokenKind::Or => {
                self.advance();
                Ok(Identifier::new("or".to_string(), false, span))
            }
            TokenKind::Not => {
                self.advance();
                Ok(Identifier::new("not".to_string(), false, span))
            }
            TokenKind::Each => {
                self.advance();
                Ok(Identifier::new("each".to_string(), false, span))
            }
            TokenKind::Try => {
                self.advance();
                Ok(Identifier::new("try".to_string(), false, span))
            }
            TokenKind::Error => {
                self.advance();
                Ok(Identifier::new("error".to_string(), false, span))
            }
            TokenKind::As => {
                self.advance();
                Ok(Identifier::new("as".to_string(), false, span))
            }
            TokenKind::Is => {
                self.advance();
                Ok(Identifier::new("is".to_string(), false, span))
            }
            TokenKind::Otherwise => {
                self.advance();
                Ok(Identifier::new("otherwise".to_string(), false, span))
            }
            TokenKind::Meta => {
                self.advance();
                Ok(Identifier::new("meta".to_string(), false, span))
            }
            TokenKind::Section => {
                self.advance();
                Ok(Identifier::new("section".to_string(), false, span))
            }
            TokenKind::Shared => {
                self.advance();
                Ok(Identifier::new("shared".to_string(), false, span))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Identifier::new("null".to_string(), false, span))
            }
            TokenKind::True => {
                self.advance();
                Ok(Identifier::new("true".to_string(), false, span))
            }
            TokenKind::False => {
                self.advance();
                Ok(Identifier::new("false".to_string(), false, span))
            }
            _ => {
                let msg = format!("Expected identifier, found {:?}", self.current_kind());
                self.errors.push(ParseError::new(&msg, span));
                Err(self.errors.clone())
            }
        }
    }
    
    // Helper methods
    
    fn current_kind(&self) -> TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }
    
    fn current_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span)
            .unwrap_or_default()
    }
    
    fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens
                .get(self.pos - 1)
                .map(|t| t.span)
                .unwrap_or_default()
        } else {
            Span::default()
        }
    }
    
    fn advance(&mut self) -> Option<&Token> {
        if self.pos < self.tokens.len() {
            let token = &self.tokens[self.pos];
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }
    
    fn skip_trivia(&mut self) {
        while self.pos < self.tokens.len() && self.tokens[self.pos].kind.is_trivia() {
            self.pos += 1;
        }
    }
    
    /// Skip only whitespace and newlines (not comments)
    fn skip_whitespace_only(&mut self) {
        while self.pos < self.tokens.len() {
            match &self.tokens[self.pos].kind {
                TokenKind::Whitespace(_) | TokenKind::Newline => {
                    self.pos += 1;
                }
                _ => break,
            }
        }
    }
    
    /// Skip trivia and collect comment tokens
    fn collect_trivia(&mut self) -> Vec<Token> {
        let mut trivia = Vec::new();
        while self.pos < self.tokens.len() && self.tokens[self.pos].kind.is_trivia() {
            let token = self.tokens[self.pos].clone();
            // Only collect comments, not whitespace/newlines
            if matches!(token.kind, TokenKind::LineComment(_) | TokenKind::BlockComment(_)) {
                trivia.push(token);
            }
            self.pos += 1;
        }
        trivia
    }
    
    /// Convert tokens to Trivia structs
    fn tokens_to_trivia(&self, tokens: &[Token]) -> Vec<Trivia> {
        tokens.iter().map(|t| {
            match &t.kind {
                TokenKind::LineComment(s) => Trivia::LineComment(s.clone()),
                TokenKind::BlockComment(s) => Trivia::BlockComment(s.clone()),
                _ => Trivia::LineComment(String::new()),
            }
        }).collect()
    }
    
    fn is_at_end(&self) -> bool {
        self.current_kind() == TokenKind::Eof
    }
    
    fn expect(&mut self, expected: TokenKind) -> Result<(), Vec<ParseError>> {
        if std::mem::discriminant(&self.current_kind()) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            let msg = format!("Expected {:?}, found {:?}", expected, self.current_kind());
            self.errors.push(ParseError::new(&msg, self.current_span()));
            Err(self.errors.clone())
        }
    }
}
