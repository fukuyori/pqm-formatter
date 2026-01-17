# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0] - 2025-01-17

### Added
- Support for keywords as record field names (`type`, `error`, `if`, `then`, `else`, `each`, etc.)
- Unicode (UTF-8) clipboard support on Windows
- Clipboard mode now accepts function expressions (`(params) => ...`), records, and lists
- New `is_simple_expr` function for better formatting decisions

### Changed
- Improved list formatting: simple elements (numbers, strings, types) kept on one line when they fit
- Improved function formatting: `=` followed by function expression on same line
- Improved nested function formatting with proper let-expression indentation
- Compact mode now uses 4-space indentation (same as default)

### Fixed
- Fixed nested let expressions inside functions getting wrong indentation
- Fixed type expressions (`type text`, `type nullable number`) being treated as complex
- Fixed clipboard mode error message for non-let expressions

## [0.3.0] - 2025-01-17

### Added
- Comment preservation for line and block comments
- Trivia collection in parser for leading/trailing comments
- Clipboard mode with UTF-8 support

### Changed
- Improved record field comment handling
- Better multiline threshold handling

## [0.2.0] - 2025-01-17

### Added
- Type annotation support (`as type`, `type table [...]`)
- Compact and expanded formatting modes
- Line-length-aware formatting

### Changed
- Improved operator spacing
- Better function call formatting

## [0.1.0] - 2025-01-17

### Added
- Initial release
- Lexer for Power Query M tokens
- Parser for Power Query M syntax
- Formatter with configurable options
- CLI tool with file and clipboard support
