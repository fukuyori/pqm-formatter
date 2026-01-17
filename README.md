# pqm-formatter

A code formatter for the **Power Query M** formula language used in Microsoft Excel Power Query and Power BI.

[日本語版 README](README.ja.md)

## Features

- **Automatic Indentation**: Consistent 4-space indentation
- **Intelligent Line Wrapping**: Based on expression complexity and configurable line length
- **Comment Preservation**: Both line comments (`//`) and block comments (`/* */`) are preserved
- **Multiple Formatting Modes**: Default, Compact, and Expanded
- **Keyword-as-Field Support**: Reserved words like `type`, `error`, `each` can be used as record field names
- **Unicode Support**: Full support for non-ASCII identifiers (e.g., Japanese variable names)
- **Clipboard Integration**: Format code directly from clipboard on Windows, macOS, and Linux

## Installation

### From Source

```bash
git clone https://github.com/fukuyori/pqm-formatter.git
cd pqm-formatter
cargo build --release
```

The executable will be at `target/release/pqmfmt` (or `pqmfmt.exe` on Windows).

### Pre-built Binaries

Download from the [Releases](https://github.com/fukuyori/pqm-formatter/releases) page.

## Usage

### Command Line

```bash
# Format a file and print to stdout
pqmfmt input.pq

# Format and write back to file
pqmfmt -w input.pq

# Format and write to different file
pqmfmt -o output.pq input.pq

# Read from stdin
cat input.pq | pqmfmt --stdin

# Check if file is formatted (exit code 1 if not)
pqmfmt -c input.pq

# Use compact mode
pqmfmt --compact input.pq

# Use expanded mode
pqmfmt --expanded input.pq

# Custom indent size
pqmfmt --indent 2 input.pq
```

### Clipboard Mode (Default)

When run without arguments, pqmfmt reads from the clipboard, formats the code, and writes the result back to the clipboard:

```bash
# Copy Power Query M code to clipboard, then run:
pqmfmt

# Formatted code is now in clipboard
```

### As a Library

```rust
use pqm_formatter::{format, Config};

let code = r#"let x=1,y=2,z=x+y in z"#;

// Default formatting
let formatted = format(code, Config::default()).unwrap();
println!("{}", formatted);

// Compact formatting
let compact = format(code, Config::compact()).unwrap();
println!("{}", compact);
```

## Formatting Modes

### Default Mode

Standard formatting with reasonable line breaks and 4-space indentation.

**Input:**
```m
let Source=Table.FromRows({{"A",1},{"B",2}},{"Name","Value"}),Filtered=Table.SelectRows(Source,each [Value]>1) in Filtered
```

**Output:**
```m
let
    Source = 
        Table.FromRows(
            {{"A", 1}, {"B", 2}},
            {"Name", "Value"}
        ),
    Filtered = Table.SelectRows(Source, each _[Value] > 1)
in
    Filtered
```

### Compact Mode (`--compact`)

Minimizes line breaks. Keeps simple expressions on one line when they fit within the line length limit.

**Output:**
```m
let Source = Table.FromRows({{"A", 1}, {"B", 2}}, {"Name", "Value"}), Filtered = Table.SelectRows(Source, each _[Value] > 1) in Filtered
```

### Expanded Mode (`--expanded`)

Maximizes readability by expanding all lists, records, and function calls.

**Output:**
```m
let
    Source = 
        Table.FromRows(
            {
                {"A", 1},
                {"B", 2}
            },
            {
                "Name",
                "Value"
            }
        ),
    Filtered = 
        Table.SelectRows(
            Source,
            each _[Value] > 1
        )
in
    Filtered
```

## Options

| Option | Description |
|--------|-------------|
| `-c, --check` | Check if file is formatted (exit 1 if not) |
| `-w, --write` | Write formatted output back to the input file |
| `-o, --output FILE` | Write output to specified file |
| `--stdin` | Read input from stdin |
| `--compact` | Use compact formatting mode |
| `--expanded` | Use expanded formatting mode |
| `--indent SIZE` | Set indent size (default: 4) |
| `--tabs` | Use tabs instead of spaces for indentation |
| `-h, --help` | Show help message |
| `-V, --version` | Show version |

## Supported Syntax

- Let expressions
- If-then-else expressions
- Try-catch expressions (with `otherwise`)
- Function definitions and calls
- Records and lists
- Field access (`[field]`) and item access (`{index}`)
- Type annotations (`as type`)
- Type expressions (`type table [Column = text]`)
- Binary and unary operators
- Each expressions
- Section expressions
- Metadata (`meta`)
- All Power Query M keywords as field names

## Error Handling

When a syntax error is encountered, pqmfmt reports the error with line and column information:

```
Error in input.pq:
Line 5: Expected identifier, found RightParen
```

In clipboard mode, the error message is prepended to the original code as a comment.

## Integration

### Visual Studio Code

Create a task in `.vscode/tasks.json`:

```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Format Power Query M",
            "type": "shell",
            "command": "pqmfmt",
            "args": ["-w", "${file}"],
            "problemMatcher": []
        }
    ]
}
```

### Editor Shortcut (Windows)

Use AutoHotkey or similar to bind pqmfmt to a keyboard shortcut for clipboard formatting.

## Building from Source

Requirements:
- Rust 1.70 or later

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test
```

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Changelog

### v0.4.0

- Added support for keywords as record field names (`type`, `error`, etc.)
- Improved list formatting for simple elements (numbers, strings, types)
- Fixed nested function formatting
- Fixed clipboard mode to accept function expressions
- Added Unicode (UTF-8) clipboard support on Windows
- Improved compact mode behavior
