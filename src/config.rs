//! Configuration for the Power Query M formatter

/// Formatter configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Number of spaces per indentation level
    pub indent_size: usize,
    
    /// Use tabs instead of spaces
    pub use_tabs: bool,
    
    /// Maximum line length before wrapping
    pub max_line_length: usize,
    
    /// Add trailing commas in lists and records
    pub trailing_comma: bool,
    
    /// Add space inside brackets: [ A = 1 ] vs [A = 1]
    pub space_in_brackets: bool,
    
    /// Add space inside braces: { 1, 2 } vs {1, 2}
    pub space_in_braces: bool,
    
    /// Add space inside parentheses: ( x + y ) vs (x + y)
    pub space_in_parens: bool,
    
    /// Align equals signs in let bindings and records
    pub align_equals: bool,
    
    /// Threshold for multiline expansion (number of elements)
    pub multiline_threshold: usize,
    
    /// Always expand let bindings to multiple lines
    pub always_expand_let: bool,
    
    /// Always expand records to multiple lines
    pub always_expand_records: bool,
    
    /// Always expand lists to multiple lines
    pub always_expand_lists: bool,
    
    /// Preserve blank lines between bindings
    pub preserve_blank_lines: bool,
    
    /// Maximum consecutive blank lines to preserve
    pub max_blank_lines: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            indent_size: 4,
            use_tabs: false,
            max_line_length: 120,
            trailing_comma: false,
            space_in_brackets: false,
            space_in_braces: false,
            space_in_parens: false,
            align_equals: false,
            multiline_threshold: 1,  // 2要素以上で展開 (> 1)
            always_expand_let: true,
            always_expand_records: false,
            always_expand_lists: false,
            preserve_blank_lines: true,
            max_blank_lines: 2,
        }
    }
}

impl Config {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a compact config (minimal whitespace, single line when possible)
    pub fn compact() -> Self {
        Self {
            indent_size: 4,  // Same as default
            max_line_length: 200,  // Allow longer lines
            multiline_threshold: 100,  // Almost never expand based on element count
            always_expand_let: false,
            always_expand_records: false,
            always_expand_lists: false,
            ..Self::default()
        }
    }
    
    /// Create an expanded config (maximum readability)
    pub fn expanded() -> Self {
        Self {
            always_expand_let: true,
            always_expand_records: true,
            always_expand_lists: true,
            multiline_threshold: 1,
            ..Self::default()
        }
    }
    
    /// Get the indentation string
    pub fn indent_str(&self) -> String {
        if self.use_tabs {
            "\t".to_string()
        } else {
            " ".repeat(self.indent_size)
        }
    }
    
    /// Get indentation at a specific level
    pub fn indent_at(&self, level: usize) -> String {
        self.indent_str().repeat(level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.indent_size, 4);
        assert!(!config.use_tabs);
        assert_eq!(config.max_line_length, 120);
    }
    
    #[test]
    fn test_indent_str() {
        let config = Config::default();
        assert_eq!(config.indent_str(), "    ");
        assert_eq!(config.indent_at(2), "        ");
        
        let tab_config = Config {
            use_tabs: true,
            ..Config::default()
        };
        assert_eq!(tab_config.indent_str(), "\t");
    }
}
