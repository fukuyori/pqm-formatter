//! # Power Query M Formatter
//!
//! A code formatter for the Power Query M formula language used in
//! Microsoft Excel Power Query and Power BI.
//!
//! ## Features
//!
//! - **Automatic indentation**: Consistent 4-space indentation
//! - **Intelligent line wrapping**: Based on expression complexity and line length
//! - **Comment preservation**: Line (`//`) and block (`/* */`) comments are preserved
//! - **Multiple formatting modes**: Default, Compact, and Expanded
//! - **Keyword-as-field support**: Reserved words like `type` can be used as field names
//! - **Unicode support**: Full support for non-ASCII identifiers and strings
//! - **Clipboard integration**: Format code directly from clipboard (Windows/macOS/Linux)
//!
//! ## Quick Start
//!
//! ```rust
//! use pqm_formatter::{format, Config};
//!
//! let code = "let x=1,y=2 in x+y";
//! let formatted = format(code, Config::default()).unwrap();
//! println!("{}", formatted);
//! ```
//!
//! ## Formatting Modes
//!
//! - **Default**: Standard formatting with reasonable line breaks
//! - **Compact**: Minimizes line breaks, keeps simple expressions on one line
//! - **Expanded**: Maximizes readability by expanding all structures

pub mod ast;
pub mod config;
pub mod formatter;
pub mod lexer;
pub mod parser;
pub mod token;

pub use config::Config;
pub use formatter::Formatter;
pub use lexer::Lexer;
pub use parser::{ParseError, Parser};

/// Format Power Query M code with the given configuration.
///
/// This is the main entry point for formatting Power Query M code.
///
/// # Arguments
///
/// * `code` - The Power Query M source code to format
/// * `config` - Formatting configuration
///
/// # Returns
///
/// * `Ok(String)` - The formatted source code
/// * `Err(Vec<ParseError>)` - A list of parsing errors if the code is invalid
///
/// # Example
///
/// ```rust
/// use pqm_formatter::{format, Config};
///
/// let code = "let x = 1, y = 2 in x + y";
/// let formatted = format(code, Config::default()).unwrap();
/// println!("{}", formatted);
/// ```
pub fn format(code: &str, config: Config) -> Result<String, Vec<ParseError>> {
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize();
    
    let mut parser = Parser::new(tokens);
    let document = parser.parse()?;
    
    let mut formatter = Formatter::new(config);
    Ok(formatter.format(&document))
}

/// Format Power Query M code with default configuration.
///
/// Convenience function equivalent to `format(code, Config::default())`.
pub fn format_default(code: &str) -> Result<String, Vec<ParseError>> {
    format(code, Config::default())
}

/// Validate Power Query M code syntax without formatting.
///
/// # Returns
///
/// * `Ok(())` - The code is syntactically valid
/// * `Err(Vec<ParseError>)` - A list of parsing errors
pub fn validate(code: &str) -> Result<(), Vec<ParseError>> {
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize();
    
    let mut parser = Parser::new(tokens);
    parser.parse()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_simple() {
        let code = "let x = 1 in x";
        let result = format_default(code);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_format_record() {
        let code = "[A = 1, B = 2]";
        let result = format_default(code);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_format_list() {
        let code = "{1, 2, 3}";
        let result = format_default(code);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_format_function() {
        let code = "(x, y) => x + y";
        let result = format_default(code);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_format_if() {
        let code = "if true then 1 else 2";
        let result = format_default(code);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate() {
        let code = "let x = 1 in x";
        assert!(validate(code).is_ok());
        
        let invalid = "let x = in x";
        assert!(validate(invalid).is_err());
    }
    
    #[test]
    fn test_format_complex() {
        let code = r#"let
    Source = Excel.CurrentWorkbook(){[Name="Table1"]}[Content],
    Filtered = Table.SelectRows(Source, each [Value] > 100)
in
    Filtered"#;
        let result = format_default(code);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_compact_mode() {
        let code = "let x = 1, y = 2 in x + y";
        let result = format(code, Config::compact());
        assert!(result.is_ok());
    }
}
