//! Lexer for Power Query M language

use crate::token::{Span, Token, TokenKind};

/// Lexer for tokenizing Power Query M source code
pub struct Lexer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }
    
    /// Get the next token (excluding trivia)
    pub fn next_non_trivia_token(&mut self) -> Token {
        loop {
            let token = self.next_token();
            if !token.kind.is_trivia() {
                return token;
            }
        }
    }
    
    /// Get the next token (including trivia)
    pub fn next_token(&mut self) -> Token {
        let start_pos = self.position;
        let start_line = self.line;
        let start_col = self.column;
        
        let kind = match self.peek_char() {
            None => TokenKind::Eof,
            Some(c) => match c {
                // Whitespace
                ' ' | '\t' => self.lex_whitespace(),
                '\r' | '\n' => self.lex_newline(),
                
                // String literal
                '"' => self.lex_string(),
                
                // Hash prefix (keywords, quoted identifiers, escape sequences)
                '#' => self.lex_hash_prefix(),
                
                // Numbers
                '0'..='9' => self.lex_number(),
                '.' if self.peek_next_char().map(|c| c.is_ascii_digit()).unwrap_or(false) => {
                    self.lex_number()
                }
                
                // Operators and punctuation
                '+' => { self.advance(); TokenKind::Plus }
                '-' => { self.advance(); TokenKind::Minus }
                '*' => { self.advance(); TokenKind::Star }
                '/' => self.lex_slash(),
                '&' => { self.advance(); TokenKind::Ampersand }
                '=' => self.lex_equal(),
                '<' => self.lex_less_than(),
                '>' => self.lex_greater_than(),
                '?' => self.lex_question(),
                '.' => self.lex_dot(),
                ',' => { self.advance(); TokenKind::Comma }
                ';' => { self.advance(); TokenKind::Semicolon }
                '(' => { self.advance(); TokenKind::LeftParen }
                ')' => { self.advance(); TokenKind::RightParen }
                '[' => { self.advance(); TokenKind::LeftBracket }
                ']' => { self.advance(); TokenKind::RightBracket }
                '{' => { self.advance(); TokenKind::LeftBrace }
                '}' => { self.advance(); TokenKind::RightBrace }
                '@' => { self.advance(); TokenKind::At }
                '!' => { self.advance(); TokenKind::Bang }
                
                // Identifiers and keywords
                c if is_identifier_start(c) => self.lex_identifier(),
                
                // Unknown character
                c => {
                    self.advance();
                    TokenKind::Invalid(c.to_string())
                }
            }
        };
        
        Token::new(
            kind,
            Span::new(start_pos, self.position, start_line, start_col),
        )
    }
    
    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|&(_, c)| c)
    }
    
    fn peek_next_char(&self) -> Option<char> {
        let mut iter = self.input[self.position..].char_indices();
        iter.next();
        iter.next().map(|(_, c)| c)
    }
    
    fn advance(&mut self) -> Option<char> {
        if let Some((pos, c)) = self.chars.next() {
            self.position = pos + c.len_utf8();
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(c)
        } else {
            None
        }
    }
    
    fn advance_while<F>(&mut self, predicate: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while let Some(c) = self.peek_char() {
            if predicate(c) {
                result.push(c);
                self.advance();
            } else {
                break;
            }
        }
        result
    }
    
    fn lex_whitespace(&mut self) -> TokenKind {
        let ws = self.advance_while(|c| c == ' ' || c == '\t');
        TokenKind::Whitespace(ws)
    }
    
    fn lex_newline(&mut self) -> TokenKind {
        let c = self.advance().unwrap();
        // Handle CRLF as single newline
        if c == '\r' && self.peek_char() == Some('\n') {
            self.advance();
        }
        TokenKind::Newline
    }
    
    fn lex_string(&mut self) -> TokenKind {
        self.advance(); // consume opening "
        let mut result = String::new();
        
        loop {
            match self.peek_char() {
                None => {
                    return TokenKind::Invalid("Unterminated string".to_string());
                }
                Some('"') => {
                    self.advance();
                    // Check for escaped quote ""
                    if self.peek_char() == Some('"') {
                        result.push('"');
                        self.advance();
                    } else {
                        break;
                    }
                }
                Some('#') => {
                    self.advance();
                    if self.peek_char() == Some('(') {
                        // Escape sequence
                        self.advance();
                        match self.lex_escape_sequence() {
                            Ok(s) => result.push_str(&s),
                            Err(e) => return TokenKind::Invalid(e),
                        }
                    } else {
                        result.push('#');
                    }
                }
                Some(c) => {
                    result.push(c);
                    self.advance();
                }
            }
        }
        
        TokenKind::Text(result)
    }
    
    fn lex_escape_sequence(&mut self) -> Result<String, String> {
        let mut result = String::new();
        
        loop {
            let escape_content = self.advance_while(|c| c != ',' && c != ')');
            
            let unescaped = match escape_content.as_str() {
                "cr" => "\r".to_string(),
                "lf" => "\n".to_string(),
                "tab" => "\t".to_string(),
                "#" => "#".to_string(),
                s if s.len() == 4 || s.len() == 8 => {
                    // Unicode escape
                    if let Ok(code) = u32::from_str_radix(s, 16) {
                        if let Some(c) = char::from_u32(code) {
                            c.to_string()
                        } else {
                            return Err(format!("Invalid unicode code point: {}", s));
                        }
                    } else {
                        return Err(format!("Invalid escape sequence: {}", s));
                    }
                }
                _ => return Err(format!("Unknown escape sequence: {}", escape_content)),
            };
            
            result.push_str(&unescaped);
            
            match self.peek_char() {
                Some(',') => {
                    self.advance();
                }
                Some(')') => {
                    self.advance();
                    break;
                }
                _ => return Err("Unterminated escape sequence".to_string()),
            }
        }
        
        Ok(result)
    }
    
    fn lex_hash_prefix(&mut self) -> TokenKind {
        self.advance(); // consume #
        
        match self.peek_char() {
            Some('"') => {
                // Quoted identifier
                self.advance();
                let mut ident = String::new();
                
                loop {
                    match self.peek_char() {
                        None => {
                            return TokenKind::Invalid("Unterminated quoted identifier".to_string());
                        }
                        Some('"') => {
                            self.advance();
                            if self.peek_char() == Some('"') {
                                ident.push('"');
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        Some(c) => {
                            ident.push(c);
                            self.advance();
                        }
                    }
                }
                
                TokenKind::QuotedIdentifier(ident)
            }
            Some(c) if is_identifier_start(c) => {
                // Hash keyword
                let ident = self.advance_while(is_identifier_continue);
                match ident.as_str() {
                    "binary" => TokenKind::HashBinary,
                    "date" => TokenKind::HashDate,
                    "datetime" => TokenKind::HashDatetime,
                    "datetimezone" => TokenKind::HashDatetimezone,
                    "duration" => TokenKind::HashDuration,
                    "infinity" => TokenKind::HashInfinity,
                    "nan" => TokenKind::HashNan,
                    "sections" => TokenKind::HashSections,
                    "shared" => TokenKind::HashShared,
                    "table" => TokenKind::HashTable,
                    "time" => TokenKind::HashTime,
                    _ => TokenKind::Invalid(format!("Unknown hash keyword: #{}", ident)),
                }
            }
            _ => TokenKind::Invalid("#".to_string()),
        }
    }
    
    fn lex_number(&mut self) -> TokenKind {
        let start = self.position;
        
        // Check for hex number
        if self.peek_char() == Some('0') {
            self.advance();
            if self.peek_char() == Some('x') || self.peek_char() == Some('X') {
                self.advance();
                let hex_digits = self.advance_while(|c| c.is_ascii_hexdigit());
                if hex_digits.is_empty() {
                    return TokenKind::Invalid("Invalid hex number".to_string());
                }
                if let Ok(value) = i64::from_str_radix(&hex_digits, 16) {
                    return TokenKind::Number(value as f64);
                } else {
                    return TokenKind::Invalid("Hex number out of range".to_string());
                }
            } else {
                // Put back the position to handle as decimal
                // We already consumed '0', continue with decimal parsing
            }
        }
        
        // Parse decimal part before the dot (if any)
        let whole = if self.position == start {
            self.advance_while(|c| c.is_ascii_digit())
        } else {
            "0".to_string() + &self.advance_while(|c| c.is_ascii_digit())
        };
        
        let mut number_str = whole;
        
        // Fractional part
        if self.peek_char() == Some('.') {
            if self.peek_next_char().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                self.advance(); // consume .
                let frac = self.advance_while(|c| c.is_ascii_digit());
                number_str.push('.');
                number_str.push_str(&frac);
            }
        }
        
        // Exponent part
        if self.peek_char() == Some('e') || self.peek_char() == Some('E') {
            self.advance();
            number_str.push('e');
            
            if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
                number_str.push(self.advance().unwrap());
            }
            
            let exp = self.advance_while(|c| c.is_ascii_digit());
            if exp.is_empty() {
                return TokenKind::Invalid("Invalid number: missing exponent".to_string());
            }
            number_str.push_str(&exp);
        }
        
        match number_str.parse::<f64>() {
            Ok(n) => TokenKind::Number(n),
            Err(_) => TokenKind::Invalid(format!("Invalid number: {}", number_str)),
        }
    }
    
    fn lex_slash(&mut self) -> TokenKind {
        self.advance(); // consume first /
        
        match self.peek_char() {
            Some('/') => {
                // Line comment
                self.advance();
                let content = self.advance_while(|c| c != '\n' && c != '\r');
                TokenKind::LineComment(content)
            }
            Some('*') => {
                // Block comment
                self.advance();
                let mut content = String::new();
                let mut depth = 1;
                
                while depth > 0 {
                    match self.peek_char() {
                        None => {
                            return TokenKind::Invalid("Unterminated block comment".to_string());
                        }
                        Some('*') => {
                            self.advance();
                            if self.peek_char() == Some('/') {
                                self.advance();
                                depth -= 1;
                                if depth > 0 {
                                    content.push_str("*/");
                                }
                            } else {
                                content.push('*');
                            }
                        }
                        Some('/') => {
                            self.advance();
                            if self.peek_char() == Some('*') {
                                self.advance();
                                depth += 1;
                                content.push_str("/*");
                            } else {
                                content.push('/');
                            }
                        }
                        Some(c) => {
                            content.push(c);
                            self.advance();
                        }
                    }
                }
                
                TokenKind::BlockComment(content)
            }
            _ => TokenKind::Slash,
        }
    }
    
    fn lex_equal(&mut self) -> TokenKind {
        self.advance(); // consume =
        if self.peek_char() == Some('>') {
            self.advance();
            TokenKind::FatArrow
        } else {
            TokenKind::Equal
        }
    }
    
    fn lex_less_than(&mut self) -> TokenKind {
        self.advance(); // consume <
        match self.peek_char() {
            Some('=') => {
                self.advance();
                TokenKind::LessThanEqual
            }
            Some('>') => {
                self.advance();
                TokenKind::NotEqual
            }
            _ => TokenKind::LessThan,
        }
    }
    
    fn lex_greater_than(&mut self) -> TokenKind {
        self.advance(); // consume >
        if self.peek_char() == Some('=') {
            self.advance();
            TokenKind::GreaterThanEqual
        } else {
            TokenKind::GreaterThan
        }
    }
    
    fn lex_question(&mut self) -> TokenKind {
        self.advance(); // consume ?
        if self.peek_char() == Some('?') {
            self.advance();
            TokenKind::QuestionQuestion
        } else {
            TokenKind::Question
        }
    }
    
    fn lex_dot(&mut self) -> TokenKind {
        self.advance(); // consume first .
        if self.peek_char() == Some('.') {
            self.advance();
            if self.peek_char() == Some('.') {
                self.advance();
                TokenKind::DotDotDot
            } else {
                TokenKind::DotDot
            }
        } else {
            // Single dot - valid for member access
            TokenKind::Dot
        }
    }
    
    fn lex_identifier(&mut self) -> TokenKind {
        let mut ident = self.advance_while(is_identifier_continue);
        
        // Check for dot-separated identifier (e.g., Table.SelectRows)
        while self.peek_char() == Some('.') {
            if self.peek_next_char().map(is_identifier_start).unwrap_or(false) {
                self.advance(); // consume '.'
                ident.push('.');
                let next_part = self.advance_while(is_identifier_continue);
                ident.push_str(&next_part);
            } else {
                break;
            }
        }
        
        // Only check for keywords if there's no dot
        if !ident.contains('.') {
            match ident.as_str() {
                "and" => return TokenKind::And,
                "as" => return TokenKind::As,
                "each" => return TokenKind::Each,
                "else" => return TokenKind::Else,
                "error" => return TokenKind::Error,
                "false" => return TokenKind::False,
                "if" => return TokenKind::If,
                "in" => return TokenKind::In,
                "is" => return TokenKind::Is,
                "let" => return TokenKind::Let,
                "meta" => return TokenKind::Meta,
                "not" => return TokenKind::Not,
                "null" => return TokenKind::Null,
                "or" => return TokenKind::Or,
                "otherwise" => return TokenKind::Otherwise,
                "section" => return TokenKind::Section,
                "shared" => return TokenKind::Shared,
                "then" => return TokenKind::Then,
                "true" => return TokenKind::True,
                "try" => return TokenKind::Try,
                "type" => return TokenKind::Type,
                _ => {}
            }
        }
        
        TokenKind::Identifier(ident)
    }
}

fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_identifier_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("let x = 1 in x");
        let tokens: Vec<_> = lexer.tokenize().into_iter()
            .filter(|t| !t.kind.is_trivia())
            .collect();
        
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Equal);
        assert_eq!(tokens[3].kind, TokenKind::Number(1.0));
        assert_eq!(tokens[4].kind, TokenKind::In);
        assert_eq!(tokens[5].kind, TokenKind::Identifier("x".to_string()));
    }
    
    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("\"Hello \"\"World\"\"\"");
        let token = lexer.next_non_trivia_token();
        assert_eq!(token.kind, TokenKind::Text("Hello \"World\"".to_string()));
    }
    
    #[test]
    fn test_quoted_identifier() {
        let mut lexer = Lexer::new("#\"My Variable\"");
        let token = lexer.next_non_trivia_token();
        assert_eq!(token.kind, TokenKind::QuotedIdentifier("My Variable".to_string()));
    }
    
    #[test]
    fn test_hex_number() {
        let mut lexer = Lexer::new("0xff");
        let token = lexer.next_non_trivia_token();
        assert_eq!(token.kind, TokenKind::Number(255.0));
    }
    
    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("=> <> <= >= ?? ...");
        let tokens: Vec<_> = lexer.tokenize().into_iter()
            .filter(|t| !t.kind.is_trivia())
            .collect();
        
        assert_eq!(tokens[0].kind, TokenKind::FatArrow);
        assert_eq!(tokens[1].kind, TokenKind::NotEqual);
        assert_eq!(tokens[2].kind, TokenKind::LessThanEqual);
        assert_eq!(tokens[3].kind, TokenKind::GreaterThanEqual);
        assert_eq!(tokens[4].kind, TokenKind::QuestionQuestion);
        assert_eq!(tokens[5].kind, TokenKind::DotDotDot);
    }
}
