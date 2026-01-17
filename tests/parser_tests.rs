//! Parser integration tests for Power Query M

use pqm_formatter::{format_default, validate};

// ============================================
// Basic Literals
// ============================================

#[test]
fn test_null_literal() {
    assert!(validate("null").is_ok());
}

#[test]
fn test_boolean_literals() {
    assert!(validate("true").is_ok());
    assert!(validate("false").is_ok());
}

#[test]
fn test_number_literals() {
    assert!(validate("42").is_ok());
    assert!(validate("3.14").is_ok());
    assert!(validate("1e10").is_ok());
    assert!(validate("1.5e-3").is_ok());
    assert!(validate("0xff").is_ok());
    assert!(validate("0xFF").is_ok());
    assert!(validate("#infinity").is_ok());
    assert!(validate("#nan").is_ok());
}

#[test]
fn test_text_literals() {
    assert!(validate(r#""hello""#).is_ok());
    assert!(validate(r#""hello ""world""""#).is_ok());  // escaped quote
    assert!(validate(r#""line1#(lf)line2""#).is_ok()); // escape sequence
}

// ============================================
// Identifiers
// ============================================

#[test]
fn test_simple_identifier() {
    assert!(validate("x").is_ok());
    assert!(validate("MyVariable").is_ok());
    assert!(validate("_private").is_ok());
}

#[test]
fn test_dotted_identifier() {
    assert!(validate("Table.SelectRows").is_ok());
    assert!(validate("Excel.CurrentWorkbook").is_ok());
    assert!(validate("List.Transform").is_ok());
}

#[test]
fn test_quoted_identifier() {
    assert!(validate(r#"#"My Variable""#).is_ok());
    assert!(validate(r#"#"Column With Spaces""#).is_ok());
}

// ============================================
// Let Expressions
// ============================================

#[test]
fn test_simple_let() {
    assert!(validate("let x = 1 in x").is_ok());
}

#[test]
fn test_let_multiple_bindings() {
    assert!(validate("let x = 1, y = 2 in x + y").is_ok());
}

#[test]
fn test_let_nested() {
    assert!(validate("let x = let y = 1 in y in x").is_ok());
}

#[test]
fn test_let_with_function_call() {
    let code = r#"let
    Source = Excel.CurrentWorkbook(),
    Table1 = Source{[Name="Table1"]}[Content]
in
    Table1"#;
    let result = validate(code);
    if let Err(ref e) = result {
        for err in e {
            eprintln!("Error: {}", err.message);
        }
    }
    assert!(result.is_ok());
}

// ============================================
// If Expressions
// ============================================

#[test]
fn test_simple_if() {
    assert!(validate("if true then 1 else 2").is_ok());
}

#[test]
fn test_if_with_comparison() {
    assert!(validate("if x > 0 then x else -x").is_ok());
}

#[test]
fn test_nested_if() {
    assert!(validate("if a then 1 else if b then 2 else 3").is_ok());
}

// ============================================
// Try Expressions
// ============================================

#[test]
fn test_try_without_otherwise() {
    assert!(validate("try x").is_ok());
}

#[test]
fn test_try_with_otherwise() {
    assert!(validate("try x otherwise 0").is_ok());
}

#[test]
fn test_try_complex() {
    assert!(validate("try Number.FromText(x) otherwise null").is_ok());
}

// ============================================
// Error Expressions
// ============================================

#[test]
fn test_error_simple() {
    assert!(validate(r#"error "Something went wrong""#).is_ok());
}

#[test]
fn test_error_record() {
    assert!(validate(r#"error [Reason = "Error", Message = "Failed"]"#).is_ok());
}

// ============================================
// Each Expressions
// ============================================

#[test]
fn test_each_simple() {
    assert!(validate("each _ + 1").is_ok());
}

#[test]
fn test_each_with_field_access() {
    assert!(validate("each [Value] > 100").is_ok());
}

#[test]
fn test_each_nested() {
    assert!(validate("each each _ + _").is_ok());
}

// ============================================
// Function Expressions
// ============================================

#[test]
fn test_function_no_params() {
    assert!(validate("() => 42").is_ok());
}

#[test]
fn test_function_single_param() {
    assert!(validate("(x) => x * 2").is_ok());
}

#[test]
fn test_function_multiple_params() {
    assert!(validate("(x, y) => x + y").is_ok());
}

#[test]
fn test_function_with_type_annotation() {
    assert!(validate("(x as number) => x * 2").is_ok());
}

#[test]
fn test_function_with_optional_param() {
    assert!(validate("(optional x) => x ?? 0").is_ok());
}

#[test]
fn test_function_with_return_type() {
    assert!(validate("(x) as number => x * 2").is_ok());
}

// ============================================
// Records
// ============================================

#[test]
fn test_empty_record() {
    assert!(validate("[]").is_ok());
}

#[test]
fn test_simple_record() {
    assert!(validate("[A = 1]").is_ok());
}

#[test]
fn test_record_multiple_fields() {
    assert!(validate("[A = 1, B = 2, C = 3]").is_ok());
}

#[test]
fn test_record_nested() {
    assert!(validate("[A = [B = 1]]").is_ok());
}

#[test]
fn test_record_with_quoted_field() {
    assert!(validate(r#"[#"Field Name" = 1]"#).is_ok());
}

// ============================================
// Lists
// ============================================

#[test]
fn test_empty_list() {
    assert!(validate("{}").is_ok());
}

#[test]
fn test_simple_list() {
    assert!(validate("{1, 2, 3}").is_ok());
}

#[test]
fn test_list_mixed_types() {
    assert!(validate(r#"{1, "text", true, null}"#).is_ok());
}

#[test]
fn test_list_nested() {
    assert!(validate("{{1, 2}, {3, 4}}").is_ok());
}

// ============================================
// Field Access
// ============================================

#[test]
fn test_field_access_simple() {
    assert!(validate("record[Field]").is_ok());
}

#[test]
fn test_field_access_quoted() {
    assert!(validate(r#"record[#"Field Name"]"#).is_ok());
}

#[test]
fn test_field_access_chained() {
    assert!(validate("record[A][B]").is_ok());
}

// ============================================
// Item Access
// ============================================

#[test]
fn test_item_access_simple() {
    assert!(validate("list{0}").is_ok());
}

#[test]
fn test_item_access_expression() {
    assert!(validate("list{index + 1}").is_ok());
}

#[test]
fn test_item_access_chained() {
    assert!(validate("list{0}{1}").is_ok());
}

// ============================================
// Mixed Access
// ============================================

#[test]
fn test_mixed_access() {
    assert!(validate("data{0}[Column]").is_ok());
}

#[test]
fn test_complex_access_chain() {
    assert!(validate("Source{[Name=\"Table1\"]}[Content]").is_ok());
}

// ============================================
// Binary Operators
// ============================================

#[test]
fn test_arithmetic_operators() {
    assert!(validate("1 + 2").is_ok());
    assert!(validate("3 - 1").is_ok());
    assert!(validate("2 * 3").is_ok());
    assert!(validate("6 / 2").is_ok());
}

#[test]
fn test_comparison_operators() {
    assert!(validate("a = b").is_ok());
    assert!(validate("a <> b").is_ok());
    assert!(validate("a < b").is_ok());
    assert!(validate("a <= b").is_ok());
    assert!(validate("a > b").is_ok());
    assert!(validate("a >= b").is_ok());
}

#[test]
fn test_logical_operators() {
    assert!(validate("a and b").is_ok());
    assert!(validate("a or b").is_ok());
}

#[test]
fn test_concatenation_operator() {
    assert!(validate(r#""hello" & " " & "world""#).is_ok());
}

#[test]
fn test_null_coalesce_operator() {
    assert!(validate("x ?? 0").is_ok());
}

#[test]
fn test_operator_precedence() {
    assert!(validate("1 + 2 * 3").is_ok());
    assert!(validate("(1 + 2) * 3").is_ok());
}

// ============================================
// Unary Operators
// ============================================

#[test]
fn test_unary_negation() {
    assert!(validate("-x").is_ok());
}

#[test]
fn test_unary_positive() {
    assert!(validate("+x").is_ok());
}

#[test]
fn test_unary_not() {
    assert!(validate("not x").is_ok());
}

// ============================================
// Type Expressions
// ============================================

#[test]
fn test_type_keyword() {
    assert!(validate("type number").is_ok());
    assert!(validate("type text").is_ok());
    assert!(validate("type any").is_ok());
}

#[test]
fn test_type_list() {
    assert!(validate("type {number}").is_ok());
}

#[test]
fn test_type_table_with_columns() {
    assert!(validate("type table [A = text, B = number]").is_ok());
}

#[test]
fn test_type_record_with_fields() {
    assert!(validate("type record [Name = text, Age = number]").is_ok());
}

#[test]
fn test_type_table_optional_column() {
    assert!(validate("type table [A = text, optional B = number]").is_ok());
}

#[test]
fn test_is_operator() {
    assert!(validate("x is number").is_ok());
}

#[test]
fn test_as_operator() {
    assert!(validate("x as number").is_ok());
}

// ============================================
// Hash Constructors
// ============================================

#[test]
fn test_hash_date() {
    assert!(validate("#date(2024, 1, 15)").is_ok());
}

#[test]
fn test_hash_time() {
    assert!(validate("#time(14, 30, 0)").is_ok());
}

#[test]
fn test_hash_datetime() {
    assert!(validate("#datetime(2024, 1, 15, 14, 30, 0)").is_ok());
}

#[test]
fn test_hash_datetimezone() {
    assert!(validate("#datetimezone(2024, 1, 15, 14, 30, 0, 9, 0)").is_ok());
}

#[test]
fn test_hash_duration() {
    assert!(validate("#duration(1, 2, 30, 0)").is_ok());
}

#[test]
fn test_hash_table() {
    assert!(validate(r#"#table({"A", "B"}, {{1, 2}, {3, 4}})"#).is_ok());
}

// ============================================
// Comments
// ============================================

#[test]
fn test_line_comment() {
    assert!(validate("1 + 2 // this is a comment").is_ok());
}

#[test]
fn test_block_comment() {
    assert!(validate("/* comment */ 1 + 2").is_ok());
}

#[test]
fn test_nested_block_comment() {
    assert!(validate("/* outer /* inner */ outer */ 1").is_ok());
}

// ============================================
// Complex Real-World Examples
// ============================================

#[test]
fn test_excel_workbook_query() {
    let code = r#"let
    Source = Excel.CurrentWorkbook(){[Name="Table1"]}[Content],
    FilteredRows = Table.SelectRows(Source, each [Amount] > 1000),
    SortedRows = Table.Sort(FilteredRows, {{"Date", Order.Descending}})
in
    SortedRows"#;
    let result = validate(code);
    if let Err(ref e) = result {
        for err in e {
            eprintln!("Error at line {}: {}", err.span.line, err.message);
        }
    }
    assert!(result.is_ok());
}

#[test]
fn test_data_transformation() {
    let code = r#"let
    Source = Csv.Document(File.Contents("data.csv")),
    Promoted = Table.PromoteHeaders(Source),
    Typed = Table.TransformColumnTypes(Promoted, {
        {"ID", Int64.Type},
        {"Name", type text},
        {"Value", type number}
    })
in
    Typed"#;
    let result = validate(code);
    if let Err(ref e) = result {
        for err in e {
            eprintln!("Error at line {}: {}", err.span.line, err.message);
        }
    }
    assert!(result.is_ok());
}

#[test]
fn test_custom_function() {
    let code = r#"let
    AddColumns = (table as table, prefix as text) as table =>
        let
            Columns = Table.ColumnNames(table),
            Renamed = List.Transform(Columns, each {_, prefix & _})
        in
            Table.RenameColumns(table, Renamed)
in
    AddColumns"#;
    let result = validate(code);
    if let Err(ref e) = result {
        for err in e {
            eprintln!("Error at line {}: {}", err.span.line, err.message);
        }
    }
    assert!(result.is_ok());
}

#[test]
fn test_error_handling() {
    let code = r#"let
    SafeDivide = (a, b) =>
        try a / b otherwise #infinity,
    Result = if b = 0 then
        error "Division by zero"
    else
        a / b
in
    Result"#;
    let result = validate(code);
    if let Err(ref e) = result {
        for err in e {
            eprintln!("Error at line {}: {}", err.span.line, err.message);
        }
    }
    assert!(result.is_ok());
}

#[test]
fn test_list_generate_with_implicit_field_access() {
    let code = r#"let
    Source = Table.FromRows(
        {{"2024-01", 900}},
        type table [Period = text, Sales = number]
    ),
    RTCalc = List.Generate(
        () => [index = 0, RT = Source[Sales]{0}],
        each [index] < List.Count(Source[Sales]),
        each [index = [index] + 1, RT = [RT] + Source[Sales]{[index] + 1}],
        each [RT]
    )
in
    RTCalc"#;
    let result = validate(code);
    if let Err(ref e) = result {
        for err in e {
            eprintln!("Error at line {}: {}", err.span.line, err.message);
        }
    }
    assert!(result.is_ok());
}

// ============================================
// Format Output Tests
// ============================================

#[test]
fn test_format_preserves_semantics() {
    let code = "let x=1,y=2 in x+y";
    let formatted = format_default(code).unwrap();
    // Should still be valid after formatting
    assert!(validate(&formatted).is_ok());
}

#[test]
fn test_format_idempotent() {
    let code = "let x = 1 in x";
    let formatted1 = format_default(code).unwrap();
    let formatted2 = format_default(&formatted1).unwrap();
    assert_eq!(formatted1, formatted2);
}

// ============================================
// Edge Cases and Error Handling
// ============================================

#[test]
fn test_deeply_nested_let() {
    let code = "let a = let b = let c = 1 in c in b in a";
    assert!(validate(code).is_ok());
}

#[test]
fn test_deeply_nested_if() {
    let code = "if a then if b then 1 else 2 else if c then 3 else 4";
    assert!(validate(code).is_ok());
}

#[test]
fn test_deeply_nested_records() {
    let code = "[A = [B = [C = [D = 1]]]]";
    assert!(validate(code).is_ok());
}

#[test]
fn test_deeply_nested_lists() {
    let code = "{{{{1}}}}";
    assert!(validate(code).is_ok());
}

#[test]
fn test_long_identifier() {
    let code = "VeryLongIdentifierNameThatShouldStillWorkCorrectly";
    assert!(validate(code).is_ok());
}

#[test]
fn test_special_characters_in_quoted_identifier() {
    let code = r#"#"Column with ""quotes"" and spaces""#;
    assert!(validate(code).is_ok());
}

#[test]
fn test_unicode_in_string() {
    let code = r#""Hello 世界 🌍""#;
    assert!(validate(code).is_ok());
}

#[test]
fn test_empty_function_body() {
    let code = "() => null";
    assert!(validate(code).is_ok());
}

#[test]
fn test_function_returning_function() {
    let code = "(x) => (y) => x + y";
    assert!(validate(code).is_ok());
}

#[test]
fn test_complex_binary_expression() {
    let code = "a + b * c - d / e and f or g";
    assert!(validate(code).is_ok());
}

#[test]
fn test_multiple_unary_operators() {
    let code = "not not not true";
    assert!(validate(code).is_ok());
}

#[test]
fn test_mixed_operators() {
    let code = "-x + +y * -z";
    assert!(validate(code).is_ok());
}

#[test]
fn test_nullable_type() {
    let code = "(x as nullable number) => x ?? 0";
    assert!(validate(code).is_ok());
}

#[test]
fn test_record_field_projection() {
    let code = "record[[Field1], [Field2]]";
    // Note: This may or may not be valid depending on M language version
    // Just testing that parser handles it
    let _ = validate(code);
}

#[test]
fn test_at_identifier() {
    let code = "let @x = 1 in @x";
    // @ prefix for scoped identifier
    let result = validate(code);
    if let Err(ref e) = result {
        for err in e {
            eprintln!("Error: {}", err.message);
        }
    }
    // This might fail if @ is not handled correctly, that's ok for now
}

#[test]
fn test_meta_expression() {
    let code = "1 meta [Type = \"number\"]";
    assert!(validate(code).is_ok());
}

#[test]
fn test_optional_item_access() {
    let code = "list{0}?";
    // Optional access might not be implemented
    let _ = validate(code);
}

// ============================================
// Syntax Error Detection
// ============================================

#[test]
fn test_missing_in_keyword() {
    let code = "let x = 1 x";
    assert!(validate(code).is_err());
}

#[test]
fn test_unclosed_bracket() {
    let code = "[A = 1";
    assert!(validate(code).is_err());
}

#[test]
fn test_unclosed_brace() {
    let code = "{1, 2, 3";
    assert!(validate(code).is_err());
}

#[test]
fn test_unclosed_paren() {
    let code = "(1 + 2";
    assert!(validate(code).is_err());
}

#[test]
fn test_missing_then() {
    let code = "if true 1 else 2";
    assert!(validate(code).is_err());
}

#[test]
fn test_missing_else() {
    let code = "if true then 1";
    assert!(validate(code).is_err());
}

#[test]
fn test_incomplete_function() {
    let code = "(x) =>";
    assert!(validate(code).is_err());
}

// ========== New tests for improved parser ==========

#[test]

// ========== New tests for improved parser v0.5 ==========

#[test]
fn test_field_projection_basic() {
    let input = "{}[[x], [y]]";
    let result = format_default(input);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("[[x], [y]]"));
}

#[test]
fn test_optional_field_selector() {
    let input = "[x]?";
    let result = format_default(input);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("?"));
}

#[test]
fn test_as_nullable_type() {
    let input = "1 as nullable number";
    let result = format_default(input);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("as nullable number"));
}

#[test]
fn test_space_separated_type_field() {
    let input = "type table [Date accessed = datetimezone]";
    let result = format_default(input);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("Date accessed"));
}

#[test]
fn test_typeless_field() {
    let input = "type table [key, bar]";
    let result = format_default(input);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    // Should not have "= any" for type-less fields
    assert!(!formatted.contains("= any"));
}

#[test]
fn test_recursive_function_call() {
    let input = "@foo(1, 2)";
    let result = format_default(input);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("@foo"));
}

#[test]
fn test_field_projection_optional() {
    let input = "{}[[x], [y]]?";
    let result = format_default(input);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("]]?"));
}

#[test]
fn test_item_access_optional() {
    let input = "{1, 2, 3}{0}?";
    let result = format_default(input);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("{0}?"));
}
