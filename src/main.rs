//! Power Query M Formatter CLI
//!
//! Usage:
//!   pqmfmt [OPTIONS] [FILE]
//!
//! Options:
//!   -c, --check      Check if the file is formatted (exit 1 if not)
//!   -w, --write      Write formatted output back to file
//!   -o, --output     Write output to specified file
//!   --stdin          Read from stdin
//!   --compact        Use compact formatting
//!   --expanded       Use expanded formatting
//!   --indent SIZE    Set indent size (default: 4)
//!   --tabs           Use tabs for indentation
//!   -h, --help       Print help
//!   -V, --version    Print version
//!
//! If no file is specified, reads from clipboard (if content starts with "let")
//! and writes formatted result back to clipboard.

use pqm_formatter::{format, Config};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::{self, Command};

#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::io::Write;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::process::Stdio;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Options {
    check: bool,
    write: bool,
    output: Option<String>,
    stdin: bool,
    compact: bool,
    expanded: bool,
    indent_size: Option<usize>,
    use_tabs: bool,
    files: Vec<String>,
}

fn parse_args() -> Options {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut opts = Options {
        check: false,
        write: false,
        output: None,
        stdin: false,
        compact: false,
        expanded: false,
        indent_size: None,
        use_tabs: false,
        files: Vec::new(),
    };
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            "-V" | "--version" => {
                println!("pqmfmt {}", VERSION);
                process::exit(0);
            }
            "-c" | "--check" => opts.check = true,
            "-w" | "--write" => opts.write = true,
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    opts.output = Some(args[i].clone());
                } else {
                    eprintln!("Error: --output requires a file path");
                    process::exit(1);
                }
            }
            "--stdin" => opts.stdin = true,
            "--compact" => opts.compact = true,
            "--expanded" => opts.expanded = true,
            "--indent" => {
                i += 1;
                if i < args.len() {
                    opts.indent_size = args[i].parse().ok();
                } else {
                    eprintln!("Error: --indent requires a number");
                    process::exit(1);
                }
            }
            "--tabs" => opts.use_tabs = true,
            arg if arg.starts_with('-') => {
                eprintln!("Unknown option: {}", arg);
                process::exit(1);
            }
            _ => opts.files.push(args[i].clone()),
        }
        i += 1;
    }
    
    opts
}

fn print_help() {
    println!(
        r#"pqmfmt - Power Query M Formatter

USAGE:
    pqmfmt [OPTIONS] [FILE]...

OPTIONS:
    -c, --check       Check if files are formatted (exit 1 if not)
    -w, --write       Write formatted output back to files
    -o, --output FILE Write output to specified file
    --stdin           Read from standard input
    --compact         Use compact formatting style
    --expanded        Use expanded formatting style
    --indent SIZE     Set indent size (default: 4)
    --tabs            Use tabs for indentation
    -h, --help        Print help information
    -V, --version     Print version information

CLIPBOARD MODE:
    If no file is specified, pqmfmt reads from clipboard.
    If clipboard content starts with "let", it formats the code
    and writes the result back to clipboard.
    On error, clipboard will contain the error message followed by
    the original code.

EXAMPLES:
    pqmfmt query.pq              Format and print to stdout
    pqmfmt -w query.pq           Format and write back to file
    pqmfmt --check query.pq      Check if file is formatted
    cat query.pq | pqmfmt --stdin    Format from stdin
    pqmfmt                       Format from clipboard to clipboard
"#
    );
}

fn build_config(opts: &Options) -> Config {
    let mut config = if opts.compact {
        Config::compact()
    } else if opts.expanded {
        Config::expanded()
    } else {
        Config::default()
    };
    
    if let Some(size) = opts.indent_size {
        config.indent_size = size;
    }
    
    if opts.use_tabs {
        config.use_tabs = true;
    }
    
    config
}

fn format_content(content: &str, config: Config) -> Result<String, String> {
    format(content, config).map_err(|errors| {
        errors
            .iter()
            .map(|e| format!("Line {}: {}", e.span.line, e.message))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

/// Get clipboard content using native commands
fn get_clipboard() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Use Windows Forms Clipboard API and save to temp file with UTF-8
        let temp_path = std::env::temp_dir().join("pqmfmt_clipboard_in.txt");
        let temp_path_str = temp_path.to_string_lossy().replace('\\', "\\\\");
        
        let ps_script = format!(r#"
Add-Type -AssemblyName System.Windows.Forms
$text = [System.Windows.Forms.Clipboard]::GetText()
[System.IO.File]::WriteAllText('{}', $text, [System.Text.Encoding]::UTF8)
"#, temp_path_str);
        
        let output = Command::new("powershell")
            .args(["-NoProfile", "-STA", "-Command", &ps_script])
            .output()
            .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("PowerShell error: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        // Read the UTF-8 file
        let content = std::fs::read_to_string(&temp_path)
            .map_err(|e| format!("Failed to read clipboard content: {}", e))?;
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
        
        // Remove BOM if present
        let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
        Ok(content.to_string())
    }
    
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pbpaste")
            .output()
            .map_err(|e| format!("Failed to execute pbpaste: {}", e))?;
        
        if output.status.success() {
            String::from_utf8(output.stdout)
                .map_err(|e| format!("Invalid UTF-8 in clipboard: {}", e))
        } else {
            Err(format!("pbpaste error: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Try xclip first, then xsel
        let output = Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
            .or_else(|_| {
                Command::new("xsel")
                    .args(["--clipboard", "--output"])
                    .output()
            })
            .map_err(|e| format!("Failed to execute xclip/xsel: {}", e))?;
        
        if output.status.success() {
            String::from_utf8(output.stdout)
                .map_err(|e| format!("Invalid UTF-8 in clipboard: {}", e))
        } else {
            Err(format!("Clipboard error: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Clipboard not supported on this platform".to_string())
    }
}

/// Set clipboard content using native commands
fn set_clipboard(content: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Write content to a temp file with UTF-8 encoding
        let temp_path = std::env::temp_dir().join("pqmfmt_clipboard_out.txt");
        
        std::fs::write(&temp_path, content.as_bytes())
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
        
        let temp_path_str = temp_path.to_string_lossy().replace('\\', "\\\\");
        
        // Use Windows Forms Clipboard API for proper Unicode support
        let ps_script = format!(r#"
Add-Type -AssemblyName System.Windows.Forms
$text = [System.IO.File]::ReadAllText('{}', [System.Text.Encoding]::UTF8)
[System.Windows.Forms.Clipboard]::SetText($text)
"#, temp_path_str);
        
        let output = Command::new("powershell")
            .args(["-NoProfile", "-STA", "-Command", &ps_script])
            .output()
            .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
        
        if output.status.success() {
            Ok(())
        } else {
            Err(format!("PowerShell Set-Clipboard failed: {}", 
                String::from_utf8_lossy(&output.stderr)))
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        let mut child = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to execute pbcopy: {}", e))?;
        
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(content.as_bytes())
                .map_err(|e| format!("Failed to write to pbcopy: {}", e))?;
        }
        
        let status = child.wait()
            .map_err(|e| format!("Failed to wait for pbcopy: {}", e))?;
        
        if status.success() {
            Ok(())
        } else {
            Err("pbcopy failed".to_string())
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Try xclip first, then xsel
        let result = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn();
        
        let mut child = match result {
            Ok(child) => child,
            Err(_) => {
                Command::new("xsel")
                    .args(["--clipboard", "--input"])
                    .stdin(Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to execute xclip/xsel: {}", e))?
            }
        };
        
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(content.as_bytes())
                .map_err(|e| format!("Failed to write to clipboard: {}", e))?;
        }
        
        let status = child.wait()
            .map_err(|e| format!("Failed to wait for clipboard command: {}", e))?;
        
        if status.success() {
            Ok(())
        } else {
            Err("Clipboard command failed".to_string())
        }
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Clipboard not supported on this platform".to_string())
    }
}

/// Process clipboard: read, format, and write back
fn process_clipboard(config: Config) {
    let content = match get_clipboard() {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error reading clipboard: {}", e);
            process::exit(1);
        }
    };
    
    // Check if content looks like Power Query M code
    let trimmed = content.trim();
    let lower = trimmed.to_lowercase();
    let is_pqm = lower.starts_with("let")
        || lower.starts_with("section")
        || trimmed.starts_with('(')   // Function expression
        || trimmed.starts_with('[')   // Record expression
        || trimmed.starts_with('{');  // List expression
    
    if !is_pqm {
        eprintln!("Clipboard does not contain Power Query M code");
        eprintln!("(Expected to start with 'let', '(', '[', '{{', or 'section')");
        if !trimmed.is_empty() {
            eprintln!("Clipboard content preview: {}...", 
                &trimmed.chars().take(50).collect::<String>());
        }
        process::exit(1);
    }
    
    match format_content(&content, config) {
        Ok(formatted) => {
            if let Err(e) = set_clipboard(&formatted) {
                eprintln!("Error writing to clipboard: {}", e);
                process::exit(1);
            }
            eprintln!("Formatted code copied to clipboard.");
        }
        Err(error_msg) => {
            // On error, put error message + original code in clipboard
            let error_output = format!(
                "// Format Error:\n// {}\n\n{}",
                error_msg.replace('\n', "\n// "),
                content
            );
            if let Err(e) = set_clipboard(&error_output) {
                eprintln!("Error writing to clipboard: {}", e);
                process::exit(1);
            }
            eprintln!("Format error. Error message and original code copied to clipboard.");
            eprintln!("{}", error_msg);
            process::exit(1);
        }
    }
}

fn main() {
    let opts = parse_args();
    let config = build_config(&opts);
    
    if opts.stdin {
        // Read from stdin
        let mut content = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut content) {
            eprintln!("Error reading stdin: {}", e);
            process::exit(1);
        }
        
        match format_content(&content, config) {
            Ok(formatted) => {
                if opts.check {
                    if formatted.trim() != content.trim() {
                        eprintln!("Input is not formatted");
                        process::exit(1);
                    }
                } else if let Some(ref output_path) = opts.output {
                    if let Err(e) = fs::write(output_path, &formatted) {
                        eprintln!("Error writing to {}: {}", output_path, e);
                        process::exit(1);
                    }
                } else {
                    print!("{}", formatted);
                }
            }
            Err(e) => {
                eprintln!("Parse error:\n{}", e);
                process::exit(1);
            }
        }
        return;
    }
    
    // No files specified - use clipboard mode
    if opts.files.is_empty() {
        process_clipboard(config);
        return;
    }
    
    let mut has_errors = false;
    let mut not_formatted = false;
    
    for file_path in &opts.files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {}: {}", file_path, e);
                has_errors = true;
                continue;
            }
        };
        
        match format_content(&content, config.clone()) {
            Ok(formatted) => {
                if opts.check {
                    if formatted.trim() != content.trim() {
                        eprintln!("{}: not formatted", file_path);
                        not_formatted = true;
                    }
                } else if opts.write {
                    if let Err(e) = fs::write(file_path, &formatted) {
                        eprintln!("Error writing {}: {}", file_path, e);
                        has_errors = true;
                    } else {
                        eprintln!("Formatted: {}", file_path);
                    }
                } else if let Some(ref output_path) = opts.output {
                    if let Err(e) = fs::write(output_path, &formatted) {
                        eprintln!("Error writing {}: {}", output_path, e);
                        has_errors = true;
                    }
                } else {
                    print!("{}", formatted);
                }
            }
            Err(e) => {
                eprintln!("Error in {}:\n{}", file_path, e);
                has_errors = true;
            }
        }
    }
    
    if has_errors {
        process::exit(1);
    }
    
    if not_formatted {
        process::exit(1);
    }
}
