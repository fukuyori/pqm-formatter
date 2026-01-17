//! Formatter for Power Query M language

use crate::ast::*;
use crate::config::Config;

/// Formatter for Power Query M code
pub struct Formatter {
    config: Config,
    output: String,
    indent_level: usize,
    current_line_length: usize,
}

impl Formatter {
    /// Create a new formatter with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            output: String::new(),
            indent_level: 0,
            current_line_length: 0,
        }
    }
    
    /// Format a document
    pub fn format(&mut self, doc: &Document) -> String {
        self.output.clear();
        self.indent_level = 0;
        self.current_line_length = 0;
        
        self.format_expr(&doc.expression);
        
        // Ensure file ends with newline
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        
        self.output.clone()
    }
    
    /// Format an expression
    fn format_expr(&mut self, expr: &Expr) {
        // Format leading trivia (comments)
        self.format_trivia(&expr.leading_trivia);
        
        match &expr.kind {
            ExprKind::Null => self.write("null"),
            ExprKind::Logical(b) => self.write(if *b { "true" } else { "false" }),
            ExprKind::Number(n) => self.format_number(*n),
            ExprKind::Text(s) => self.format_text(s),
            ExprKind::Identifier(name) => self.write(name),
            ExprKind::QuotedIdentifier(name) => {
                self.write("#\"");
                self.write(&escape_identifier(name));
                self.write("\"");
            }
            ExprKind::Let(let_expr) => self.format_let(let_expr),
            ExprKind::If(if_expr) => self.format_if(if_expr),
            ExprKind::Try(try_expr) => self.format_try(try_expr),
            ExprKind::Error(inner) => {
                self.write("error ");
                self.format_expr(inner);
            }
            ExprKind::Each(inner) => {
                self.write("each ");
                self.format_expr(inner);
            }
            ExprKind::Function(func) => self.format_function(func),
            ExprKind::FunctionCall(call) => self.format_function_call(call),
            ExprKind::Record(record) => self.format_record(record),
            ExprKind::List(list) => self.format_list(list),
            ExprKind::FieldAccess(access) => self.format_field_access(access),
            ExprKind::FieldProjection(proj) => self.format_field_projection(proj),
            ExprKind::ItemAccess(access) => self.format_item_access(access),
            ExprKind::Binary(binary) => self.format_binary(binary),
            ExprKind::Unary(unary) => self.format_unary(unary),
            ExprKind::Parenthesized(inner) => {
                self.write("(");
                self.format_expr(inner);
                self.write(")");
            }
            ExprKind::Type(type_expr) => self.format_type_expr(type_expr),
            ExprKind::Metadata(meta) => self.format_metadata(meta),
            ExprKind::Underscore => self.write("_"),
            ExprKind::HashTable(table) => self.format_hash_table(table),
            ExprKind::HashDate(date) => self.format_hash_date(date),
            ExprKind::HashTime(time) => self.format_hash_time(time),
            ExprKind::HashDatetime(dt) => self.format_hash_datetime(dt),
            ExprKind::HashDatetimezone(dtz) => self.format_hash_datetimezone(dtz),
            ExprKind::HashDuration(dur) => self.format_hash_duration(dur),
        }
        
        // Format trailing trivia (comments)
        self.format_trivia(&expr.trailing_trivia);
    }
    
    /// Format trivia (comments)
    fn format_trivia(&mut self, trivia: &[Trivia]) {
        for t in trivia {
            match t {
                Trivia::LineComment(content) => {
                    self.write("//");
                    if !content.starts_with(' ') && !content.is_empty() {
                        self.write(" ");
                    }
                    self.write(content);
                    self.newline();
                }
                Trivia::BlockComment(content) => {
                    self.write("/*");
                    self.write(content);
                    self.write("*/");
                }
                Trivia::Newline => {
                    // Handled separately
                }
                Trivia::Whitespace(_) => {
                    // Ignored - we control whitespace
                }
            }
        }
    }
    
    /// Format a number
    fn format_number(&mut self, n: f64) {
        if n.is_infinite() {
            if n.is_sign_positive() {
                self.write("#infinity");
            } else {
                self.write("-#infinity");
            }
        } else if n.is_nan() {
            self.write("#nan");
        } else if n.fract() == 0.0 && n.abs() < 1e15 {
            // Integer-like number
            self.write(&format!("{}", n as i64));
        } else {
            // Format with appropriate precision
            let s = format!("{}", n);
            self.write(&s);
        }
    }
    
    /// Format a text literal
    fn format_text(&mut self, s: &str) {
        self.write("\"");
        self.write(&escape_text(s));
        self.write("\"");
    }
    
    /// Format let expression
    fn format_let(&mut self, let_expr: &LetExpr) {
        // In compact mode with always_expand_let=false, try to fit on single line
        let estimated_len = self.estimate_let_length(let_expr);
        let single_line = !self.config.always_expand_let 
            && estimated_len <= self.config.max_line_length
            && !let_expr.bindings.iter().any(|b| self.is_complex_expr(&b.value))
            && let_expr.bindings.iter().all(|b| b.leading_trivia.is_empty() && b.trailing_trivia.is_empty());
        
        if single_line {
            self.format_let_single_line(let_expr);
        } else {
            self.format_let_multi_line(let_expr);
        }
    }
    
    fn format_let_single_line(&mut self, let_expr: &LetExpr) {
        self.write("let ");
        
        for (i, binding) in let_expr.bindings.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.format_identifier(&binding.name);
            self.write(" = ");
            self.format_expr(&binding.value);
        }
        
        self.write(" in ");
        self.format_expr(&let_expr.body);
    }
    
    fn format_let_multi_line(&mut self, let_expr: &LetExpr) {
        self.write("let");
        self.newline();
        self.indent_level += 1;
        
        for (i, binding) in let_expr.bindings.iter().enumerate() {
            // Format leading trivia (comments) for binding
            if !binding.leading_trivia.is_empty() {
                for t in &binding.leading_trivia {
                    self.write_indent();
                    match t {
                        Trivia::LineComment(content) => {
                            self.write("//");
                            if !content.starts_with(' ') && !content.is_empty() {
                                self.write(" ");
                            }
                            self.write(content);
                            self.newline();
                        }
                        Trivia::BlockComment(content) => {
                            self.write("/*");
                            self.write(content);
                            self.write("*/");
                            self.newline();
                        }
                        _ => {}
                    }
                }
            }
            
            self.write_indent();
            self.format_identifier(&binding.name);
            self.write(" = ");
            
            // Special handling for function expressions
            // Put function on same line as `=`, let the function handle its own formatting
            if matches!(&binding.value.kind, ExprKind::Function(_)) {
                self.format_expr(&binding.value);
            } else {
                // Check if value needs to be on new line
                let value_complex = self.is_complex_expr(&binding.value);
                let value_length = self.estimate_expr_length(&binding.value);
                let current_pos = self.current_line_length;
                let would_exceed = current_pos + value_length > self.config.max_line_length;
                
                if value_complex || would_exceed {
                    self.newline();
                    self.indent_level += 1;
                    self.write_indent();
                    self.format_expr(&binding.value);
                    self.indent_level -= 1;
                } else {
                    self.format_expr(&binding.value);
                }
            }
            
            // Format trailing trivia (comments after value, on same line)
            for t in &binding.trailing_trivia {
                match t {
                    Trivia::LineComment(content) => {
                        self.write(" //");
                        if !content.starts_with(' ') && !content.is_empty() {
                            self.write(" ");
                        }
                        self.write(content);
                    }
                    Trivia::BlockComment(content) => {
                        self.write(" /*");
                        self.write(content);
                        self.write("*/");
                    }
                    _ => {}
                }
            }
            
            // Add comma if not last binding
            if i < let_expr.bindings.len() - 1 {
                self.write(",");
            }
            
            self.newline();
        }
        
        self.indent_level -= 1;
        self.write_indent();
        self.write("in");
        self.newline();
        self.indent_level += 1;
        self.write_indent();
        self.format_expr(&let_expr.body);
        self.indent_level -= 1;
    }
    
    /// Format if expression
    fn format_if(&mut self, if_expr: &IfExpr) {
        let single_line = self.estimate_if_length(if_expr) <= self.config.max_line_length
            && !self.is_complex_expr(&if_expr.condition)
            && !self.is_complex_expr(&if_expr.then_branch)
            && !self.is_complex_expr(&if_expr.else_branch);
        
        if single_line {
            self.format_if_single_line(if_expr);
        } else {
            self.format_if_multi_line(if_expr);
        }
    }
    
    fn format_if_single_line(&mut self, if_expr: &IfExpr) {
        self.write("if ");
        self.format_expr(&if_expr.condition);
        self.write(" then ");
        self.format_expr(&if_expr.then_branch);
        self.write(" else ");
        self.format_expr(&if_expr.else_branch);
    }
    
    fn format_if_multi_line(&mut self, if_expr: &IfExpr) {
        self.write("if ");
        self.format_expr(&if_expr.condition);
        self.write(" then");
        self.newline();
        self.indent_level += 1;
        self.write_indent();
        self.format_expr(&if_expr.then_branch);
        self.indent_level -= 1;
        self.newline();
        self.write_indent();
        
        // Check for else-if chain
        if let ExprKind::If(_) = &if_expr.else_branch.kind {
            self.write("else ");
            self.format_expr(&if_expr.else_branch);
        } else {
            self.write("else");
            self.newline();
            self.indent_level += 1;
            self.write_indent();
            self.format_expr(&if_expr.else_branch);
            self.indent_level -= 1;
        }
    }
    
    /// Format try expression
    fn format_try(&mut self, try_expr: &TryExpr) {
        self.write("try ");
        self.format_expr(&try_expr.expr);
        
        if let Some(ref otherwise) = try_expr.otherwise {
            self.write(" otherwise ");
            self.format_expr(otherwise);
        }
    }
    
    /// Format function expression
    fn format_function(&mut self, func: &FunctionExpr) {
        self.write("(");
        
        for (i, param) in func.parameters.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            
            if param.optional {
                self.write("optional ");
            }
            
            self.format_identifier(&param.name);
            
            if let Some(ref type_ann) = param.type_annotation {
                self.write(" as ");
                self.format_type_annotation(type_ann);
            }
        }
        
        self.write(")");
        
        if let Some(ref return_type) = func.return_type {
            self.write(" as ");
            self.format_type_annotation(return_type);
        }
        
        self.write(" =>");
        
        // Check if body is a let expression
        if let ExprKind::Let(let_expr) = &func.body.kind {
            // In compact mode, try to format let on same line if it would fit
            if !self.config.always_expand_let {
                let let_len = self.estimate_let_length(let_expr);
                let has_comments = let_expr.bindings.iter()
                    .any(|b| !b.leading_trivia.is_empty() || !b.trailing_trivia.is_empty());
                let has_complex = let_expr.bindings.iter()
                    .any(|b| self.is_complex_expr(&b.value));
                
                if !has_comments && !has_complex && self.current_line_length + 1 + let_len <= self.config.max_line_length {
                    self.write(" ");
                    self.format_expr(&func.body);
                    return;
                }
            }
            
            // At top level (indent_level == 0), put let on new line without indent
            // Inside other expressions, indent the let
            if self.indent_level == 0 {
                self.newline();
                self.format_expr(&func.body);
            } else {
                self.newline();
                self.indent_level += 1;
                self.write_indent();
                self.format_expr(&func.body);
                self.indent_level -= 1;
            }
        } else if self.is_complex_expr(&func.body) {
            self.write(" ");
            self.newline();
            self.indent_level += 1;
            self.write_indent();
            self.format_expr(&func.body);
            self.indent_level -= 1;
        } else {
            self.write(" ");
            self.format_expr(&func.body);
        }
    }
    
    /// Format function call
    fn format_function_call(&mut self, call: &FunctionCallExpr) {
        self.format_expr(&call.function);
        self.write("(");
        
        // Estimate total length of arguments
        let args_length: usize = call.arguments.iter().enumerate()
            .map(|(i, a)| {
                let len = self.estimate_expr_length(a);
                if i > 0 { len + 2 } else { len } // add ", " for non-first args
            })
            .sum();
        
        // Check if all arguments are simple
        let all_simple = call.arguments.iter().all(|arg| self.is_simple_expr(arg));
        
        // Decide whether to expand
        // Don't expand if all arguments are simple and would fit on line
        let multiline = call.arguments.iter().any(|a| self.is_complex_expr(a))
            || (!all_simple && call.arguments.len() > self.config.multiline_threshold)
            || self.would_exceed_line_length(args_length + 1); // +1 for ")"
        
        if multiline && !call.arguments.is_empty() {
            self.newline();
            self.indent_level += 1;
            
            for (i, arg) in call.arguments.iter().enumerate() {
                self.write_indent();
                self.format_expr(arg);
                
                if i < call.arguments.len() - 1 {
                    self.write(",");
                } else if self.config.trailing_comma {
                    self.write(",");
                }
                self.newline();
            }
            
            self.indent_level -= 1;
            self.write_indent();
        } else {
            for (i, arg) in call.arguments.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.format_expr(arg);
            }
        }
        
        self.write(")");
    }
    
    /// Format record expression
    fn format_record(&mut self, record: &RecordExpr) {
        if record.fields.is_empty() {
            self.write("[]");
            return;
        }
        
        // Estimate total length of fields
        let fields_length: usize = record.fields.iter().enumerate()
            .map(|(i, field)| {
                let len = field.name.name.len() + 3 + self.estimate_expr_length(&field.value);
                if i > 0 { len + 2 } else { len } // add ", " for non-first fields
            })
            .sum();
        
        // Check if any field has comments
        let has_comments = record.fields.iter()
            .any(|f| !f.leading_trivia.is_empty() || !f.trailing_trivia.is_empty());
        
        // Decide whether to expand
        let multiline = self.config.always_expand_records
            || record.fields.len() > self.config.multiline_threshold
            || record.fields.iter().any(|f| self.is_complex_expr(&f.value))
            || has_comments
            || self.would_exceed_line_length(fields_length + 2); // +2 for "[]"
        
        self.write("[");
        
        if multiline {
            self.newline();
            self.indent_level += 1;
            
            for (i, field) in record.fields.iter().enumerate() {
                // Format leading trivia (comments before field)
                if !field.leading_trivia.is_empty() {
                    for t in &field.leading_trivia {
                        self.write_indent();
                        match t {
                            Trivia::LineComment(content) => {
                                self.write("//");
                                if !content.starts_with(' ') && !content.is_empty() {
                                    self.write(" ");
                                }
                                self.write(content);
                                self.newline();
                            }
                            Trivia::BlockComment(content) => {
                                self.write("/*");
                                self.write(content);
                                self.write("*/");
                                self.newline();
                            }
                            _ => {}
                        }
                    }
                }
                
                self.write_indent();
                self.format_identifier(&field.name);
                self.write(" = ");
                
                // Check if field value needs to be on new line
                let value_complex = self.is_complex_expr(&field.value);
                let value_length = self.estimate_expr_length(&field.value);
                let would_exceed = self.current_line_length + value_length > self.config.max_line_length;
                
                if value_complex || would_exceed {
                    self.newline();
                    self.indent_level += 1;
                    self.write_indent();
                    self.format_expr(&field.value);
                    self.indent_level -= 1;
                } else {
                    self.format_expr(&field.value);
                }
                
                // Format trailing trivia (comments after value, on same line)
                for t in &field.trailing_trivia {
                    match t {
                        Trivia::LineComment(content) => {
                            self.write("  //");
                            if !content.starts_with(' ') && !content.is_empty() {
                                self.write(" ");
                            }
                            self.write(content);
                        }
                        Trivia::BlockComment(content) => {
                            self.write(" /*");
                            self.write(content);
                            self.write("*/");
                        }
                        _ => {}
                    }
                }
                
                if i < record.fields.len() - 1 {
                    self.write(",");
                } else if self.config.trailing_comma {
                    self.write(",");
                }
                
                self.newline();
            }
            
            self.indent_level -= 1;
            self.write_indent();
        } else {
            if self.config.space_in_brackets {
                self.write(" ");
            }
            
            for (i, field) in record.fields.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.format_identifier(&field.name);
                self.write(" = ");
                self.format_expr(&field.value);
            }
            
            if self.config.space_in_brackets {
                self.write(" ");
            }
        }
        
        self.write("]");
    }
    
    /// Format list expression
    fn format_list(&mut self, list: &ListExpr) {
        if list.items.is_empty() {
            self.write("{}");
            return;
        }
        
        // Estimate total length of items
        let items_length: usize = list.items.iter().enumerate()
            .map(|(i, item)| {
                let len = self.estimate_expr_length(item);
                if i > 0 { len + 2 } else { len } // add ", " for non-first items
            })
            .sum();
        
        // Check if all items are simple (numbers, strings, identifiers, types)
        let all_simple = list.items.iter().all(|item| self.is_simple_expr(item));
        
        // Decide whether to expand
        // Keep simple short lists on one line if they fit
        let multiline = self.config.always_expand_lists
            || list.items.iter().any(|i| self.is_complex_expr(i))
            || (!all_simple && list.items.len() > self.config.multiline_threshold)
            || self.would_exceed_line_length(items_length + 2); // +2 for "{}"
        
        self.write("{");
        
        if multiline {
            self.newline();
            self.indent_level += 1;
            
            for (i, item) in list.items.iter().enumerate() {
                self.write_indent();
                self.format_expr(item);
                
                if i < list.items.len() - 1 {
                    self.write(",");
                } else if self.config.trailing_comma {
                    self.write(",");
                }
                self.newline();
            }
            
            self.indent_level -= 1;
            self.write_indent();
        } else {
            if self.config.space_in_braces {
                self.write(" ");
            }
            
            for (i, item) in list.items.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.format_expr(item);
            }
            
            if self.config.space_in_braces {
                self.write(" ");
            }
        }
        
        self.write("}");
    }
    
    /// Format field access
    fn format_field_access(&mut self, access: &FieldAccessExpr) {
        self.format_expr(&access.expr);
        self.write("[");
        self.format_identifier(&access.field);
        self.write("]");
        if access.optional {
            self.write("?");
        }
    }
    
    /// Format field projection
    fn format_field_projection(&mut self, proj: &FieldProjectionExpr) {
        self.format_expr(&proj.expr);
        self.write("[");
        for (i, field) in proj.fields.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.write("[");
            self.format_identifier(field);
            self.write("]");
        }
        self.write("]");
        if proj.optional {
            self.write("?");
        }
    }
    
    /// Format item access
    fn format_item_access(&mut self, access: &ItemAccessExpr) {
        self.format_expr(&access.expr);
        self.write("{");
        self.format_expr(&access.index);
        self.write("}");
        if access.optional {
            self.write("?");
        }
    }
    
    /// Format binary expression
    fn format_binary(&mut self, binary: &BinaryExpr) {
        // Add parentheses if needed based on precedence
        let needs_left_parens = self.needs_parens_for_binary(&binary.left, &binary.operator, true);
        let needs_right_parens = self.needs_parens_for_binary(&binary.right, &binary.operator, false);
        
        if needs_left_parens {
            self.write("(");
        }
        self.format_expr(&binary.left);
        if needs_left_parens {
            self.write(")");
        }
        
        self.write(" ");
        self.write(binary.operator.as_str());
        self.write(" ");
        
        // For 'as' and 'is' operators, format the type annotation directly (without 'type' keyword)
        if matches!(binary.operator, BinaryOp::As | BinaryOp::Is) {
            if let ExprKind::Type(type_expr) = &binary.right.kind {
                self.format_type_annotation(&type_expr.type_annotation);
                return;
            }
        }
        
        if needs_right_parens {
            self.write("(");
        }
        self.format_expr(&binary.right);
        if needs_right_parens {
            self.write(")");
        }
    }
    
    fn needs_parens_for_binary(&self, expr: &Expr, parent_op: &BinaryOp, is_left: bool) -> bool {
        if let ExprKind::Binary(inner) = &expr.kind {
            let inner_prec = inner.operator.precedence();
            let parent_prec = parent_op.precedence();
            
            if inner_prec < parent_prec {
                return true;
            }
            
            // Right associativity check for same precedence
            if inner_prec == parent_prec && !is_left {
                return true;
            }
        }
        false
    }
    
    /// Format unary expression
    fn format_unary(&mut self, unary: &UnaryExpr) {
        match unary.operator {
            UnaryOp::Not => {
                self.write("not ");
                self.format_expr(&unary.operand);
            }
            UnaryOp::Negate => {
                self.write("-");
                self.format_expr(&unary.operand);
            }
            UnaryOp::Positive => {
                self.write("+");
                self.format_expr(&unary.operand);
            }
        }
    }
    
    /// Format type expression
    fn format_type_expr(&mut self, type_expr: &TypeExpr) {
        self.write("type ");
        self.format_type_annotation(&type_expr.type_annotation);
    }
    
    /// Format type annotation
    fn format_type_annotation(&mut self, type_ann: &TypeAnnotation) {
        match &type_ann.kind {
            TypeKind::Any => self.write("any"),
            TypeKind::None => self.write("none"),
            TypeKind::Null => self.write("null"),
            TypeKind::Logical => self.write("logical"),
            TypeKind::Number => self.write("number"),
            TypeKind::Time => self.write("time"),
            TypeKind::Date => self.write("date"),
            TypeKind::DateTime => self.write("datetime"),
            TypeKind::DateTimeZone => self.write("datetimezone"),
            TypeKind::Duration => self.write("duration"),
            TypeKind::Text => self.write("text"),
            TypeKind::Binary => self.write("binary"),
            TypeKind::Type => self.write("type"),
            TypeKind::List(inner) => {
                self.write("{");
                if let Some(inner) = inner {
                    self.format_type_annotation(inner);
                }
                self.write("}");
            }
            TypeKind::Record(fields) => {
                self.write("[");
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    if field.optional {
                        self.write("optional ");
                    }
                    self.format_identifier(&field.name);
                    // Only output "= type" if the type is not Any (type-less field)
                    if !matches!(&field.type_annotation.kind, TypeKind::Any) {
                        self.write(" = ");
                        self.format_type_annotation(&field.type_annotation);
                    }
                }
                self.write("]");
            }
            TypeKind::Table(fields) => {
                self.write("table [");
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    if field.optional {
                        self.write("optional ");
                    }
                    self.format_identifier(&field.name);
                    // Only output "= type" if the type is not Any (type-less field)
                    if !matches!(&field.type_annotation.kind, TypeKind::Any) {
                        self.write(" = ");
                        self.format_type_annotation(&field.type_annotation);
                    }
                }
                self.write("]");
            }
            TypeKind::Function(params, ret) => {
                self.write("function (");
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.format_type_annotation(param);
                }
                self.write(") as ");
                self.format_type_annotation(ret);
            }
            TypeKind::Custom(name) => self.write(name),
            TypeKind::Nullable(inner) => {
                self.write("nullable ");
                self.format_type_annotation(inner);
            }
        }
    }
    
    /// Format metadata expression
    fn format_metadata(&mut self, meta: &MetadataExpr) {
        self.format_expr(&meta.expr);
        self.write(" meta ");
        self.format_expr(&meta.metadata);
    }
    
    /// Format #table constructor
    fn format_hash_table(&mut self, table: &HashTableExpr) {
        self.write("#table(");
        self.format_expr(&table.columns);
        self.write(", ");
        self.format_expr(&table.rows);
        self.write(")");
    }
    
    /// Format #date constructor
    fn format_hash_date(&mut self, date: &HashDateExpr) {
        self.write("#date(");
        self.format_expr(&date.year);
        self.write(", ");
        self.format_expr(&date.month);
        self.write(", ");
        self.format_expr(&date.day);
        self.write(")");
    }
    
    /// Format #time constructor
    fn format_hash_time(&mut self, time: &HashTimeExpr) {
        self.write("#time(");
        self.format_expr(&time.hour);
        self.write(", ");
        self.format_expr(&time.minute);
        self.write(", ");
        self.format_expr(&time.second);
        self.write(")");
    }
    
    /// Format #datetime constructor
    fn format_hash_datetime(&mut self, dt: &HashDatetimeExpr) {
        self.write("#datetime(");
        self.format_expr(&dt.year);
        self.write(", ");
        self.format_expr(&dt.month);
        self.write(", ");
        self.format_expr(&dt.day);
        self.write(", ");
        self.format_expr(&dt.hour);
        self.write(", ");
        self.format_expr(&dt.minute);
        self.write(", ");
        self.format_expr(&dt.second);
        self.write(")");
    }
    
    /// Format #datetimezone constructor
    fn format_hash_datetimezone(&mut self, dtz: &HashDatetimezoneExpr) {
        self.write("#datetimezone(");
        self.format_expr(&dtz.year);
        self.write(", ");
        self.format_expr(&dtz.month);
        self.write(", ");
        self.format_expr(&dtz.day);
        self.write(", ");
        self.format_expr(&dtz.hour);
        self.write(", ");
        self.format_expr(&dtz.minute);
        self.write(", ");
        self.format_expr(&dtz.second);
        self.write(", ");
        self.format_expr(&dtz.offset_hours);
        self.write(", ");
        self.format_expr(&dtz.offset_minutes);
        self.write(")");
    }
    
    /// Format #duration constructor
    fn format_hash_duration(&mut self, dur: &HashDurationExpr) {
        self.write("#duration(");
        self.format_expr(&dur.days);
        self.write(", ");
        self.format_expr(&dur.hours);
        self.write(", ");
        self.format_expr(&dur.minutes);
        self.write(", ");
        self.format_expr(&dur.seconds);
        self.write(")");
    }
    
    /// Format identifier
    fn format_identifier(&mut self, ident: &Identifier) {
        if ident.quoted {
            self.write("#\"");
            self.write(&escape_identifier(&ident.name));
            self.write("\"");
        } else {
            self.write(&ident.name);
        }
    }
    
    // Helper methods
    
    fn write(&mut self, s: &str) {
        self.output.push_str(s);
        self.current_line_length += s.len();
    }
    
    fn newline(&mut self) {
        self.output.push('\n');
        self.current_line_length = 0;
    }
    
    fn write_indent(&mut self) {
        let indent = self.config.indent_at(self.indent_level);
        self.output.push_str(&indent);
        self.current_line_length = indent.len();
    }
    
    /// Check if expression is simple (suitable for single-line formatting)
    fn is_simple_expr(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Number(_) 
            | ExprKind::Text(_) 
            | ExprKind::Identifier(_)
            | ExprKind::QuotedIdentifier(_)
            | ExprKind::Null
            | ExprKind::Logical(_)
            | ExprKind::Type(_)
            | ExprKind::Underscore => true,
            // Field access like _[Name] is simple
            ExprKind::FieldAccess(fa) => self.is_simple_expr(&fa.expr),
            // Item access like list{0} is simple if both parts are simple
            ExprKind::ItemAccess(ia) => self.is_simple_expr(&ia.expr) && self.is_simple_expr(&ia.index),
            _ => false,
        }
    }
    
    fn is_complex_expr(&self, expr: &Expr) -> bool {
        matches!(
            &expr.kind,
            ExprKind::Let(_)
                | ExprKind::If(_)
                | ExprKind::Try(_)
                | ExprKind::Function(_)
        ) || match &expr.kind {
            ExprKind::Record(r) => r.fields.len() > self.config.multiline_threshold,
            // Lists are complex only if they contain complex items
            ExprKind::List(l) => l.items.iter().any(|i| self.is_complex_expr(i)),
            ExprKind::FunctionCall(c) => {
                c.arguments.len() > self.config.multiline_threshold
                    || c.arguments.iter().any(|a| self.is_complex_expr(a))
                    || c.arguments.iter().any(|a| self.estimate_expr_length(a) > 30)
            }
            _ => false,
        }
    }
    
    fn estimate_let_length(&self, let_expr: &LetExpr) -> usize {
        // Rough estimate
        let mut len = 8; // "let " + " in "
        for binding in &let_expr.bindings {
            len += binding.name.name.len() + 3; // " = "
            len += self.estimate_expr_length(&binding.value);
            len += 2; // ", "
        }
        len + self.estimate_expr_length(&let_expr.body)
    }
    
    fn estimate_if_length(&self, if_expr: &IfExpr) -> usize {
        let mut len = 15; // "if " + " then " + " else "
        len += self.estimate_expr_length(&if_expr.condition);
        len += self.estimate_expr_length(&if_expr.then_branch);
        len += self.estimate_expr_length(&if_expr.else_branch);
        len
    }
    
    fn estimate_expr_length(&self, expr: &Expr) -> usize {
        match &expr.kind {
            ExprKind::Null => 4,
            ExprKind::Logical(b) => if *b { 4 } else { 5 },
            ExprKind::Number(n) => format!("{}", n).len(),
            ExprKind::Text(s) => s.len() + 2,
            ExprKind::Identifier(s) => s.len(),
            ExprKind::QuotedIdentifier(s) => s.len() + 3,
            ExprKind::Underscore => 1,
            ExprKind::FieldAccess(access) => {
                self.estimate_expr_length(&access.expr) + access.field.name.len() + 2
            }
            ExprKind::ItemAccess(access) => {
                self.estimate_expr_length(&access.expr) + self.estimate_expr_length(&access.index) + 2
            }
            ExprKind::FunctionCall(call) => {
                let mut len = self.estimate_expr_length(&call.function) + 2; // "()"
                for (i, arg) in call.arguments.iter().enumerate() {
                    if i > 0 {
                        len += 2; // ", "
                    }
                    len += self.estimate_expr_length(arg);
                }
                len
            }
            ExprKind::List(list) => {
                let mut len = 2; // "{}"
                for (i, item) in list.items.iter().enumerate() {
                    if i > 0 {
                        len += 2; // ", "
                    }
                    len += self.estimate_expr_length(item);
                }
                len
            }
            ExprKind::Record(record) => {
                let mut len = 2; // "[]"
                for (i, field) in record.fields.iter().enumerate() {
                    if i > 0 {
                        len += 2; // ", "
                    }
                    len += field.name.name.len() + 3; // " = "
                    len += self.estimate_expr_length(&field.value);
                }
                len
            }
            ExprKind::Binary(binary) => {
                self.estimate_expr_length(&binary.left) + 3 + self.estimate_expr_length(&binary.right)
            }
            ExprKind::Unary(unary) => {
                1 + self.estimate_expr_length(&unary.operand)
            }
            ExprKind::Parenthesized(inner) => {
                2 + self.estimate_expr_length(inner)
            }
            ExprKind::Type(type_expr) => {
                5 + self.estimate_type_length(&type_expr.type_annotation) // "type "
            }
            ExprKind::Each(inner) => {
                5 + self.estimate_expr_length(inner) // "each "
            }
            ExprKind::Error(inner) => {
                6 + self.estimate_expr_length(inner) // "error "
            }
            // Complex expressions - return large value to force expansion
            ExprKind::Let(_) | ExprKind::If(_) | ExprKind::Try(_) | ExprKind::Function(_) => 200,
            _ => 50, // Conservative estimate for other complex expressions
        }
    }
    
    fn estimate_type_length(&self, type_ann: &TypeAnnotation) -> usize {
        match &type_ann.kind {
            TypeKind::Any => 3,
            TypeKind::None => 4,
            TypeKind::Null => 4,
            TypeKind::Logical => 7,
            TypeKind::Number => 6,
            TypeKind::Time => 4,
            TypeKind::Date => 4,
            TypeKind::DateTime => 8,
            TypeKind::DateTimeZone => 12,
            TypeKind::Duration => 8,
            TypeKind::Text => 4,
            TypeKind::Binary => 6,
            TypeKind::Type => 4,
            TypeKind::List(inner) => {
                4 + inner.as_ref().map(|i| self.estimate_type_length(i)).unwrap_or(0)
            }
            TypeKind::Record(fields) | TypeKind::Table(fields) => {
                let base = if matches!(&type_ann.kind, TypeKind::Table(_)) { 5 } else { 6 };
                let mut len = base + 2; // "table " or "record " + "[]"
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        len += 2; // ", "
                    }
                    len += field.name.name.len() + 3; // " = "
                    len += self.estimate_type_length(&field.type_annotation);
                }
                len
            }
            TypeKind::Function(_, _) => 10,
            TypeKind::Nullable(inner) => 9 + self.estimate_type_length(inner),
            TypeKind::Custom(name) => name.len(),
        }
    }
    
    /// Check if expression would exceed line length when formatted inline
    fn would_exceed_line_length(&self, estimated_len: usize) -> bool {
        self.current_line_length + estimated_len > self.config.max_line_length
    }
}

/// Escape special characters in text literals
fn escape_text(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '"' => result.push_str("\"\""),
            '\r' => result.push_str("#(cr)"),
            '\n' => result.push_str("#(lf)"),
            '\t' => result.push_str("#(tab)"),
            _ => result.push(c),
        }
    }
    result
}

/// Escape special characters in identifiers
fn escape_identifier(s: &str) -> String {
    s.replace('"', "\"\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    
    fn format_code(code: &str) -> String {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let doc = parser.parse().unwrap();
        let mut formatter = Formatter::new(Config::default());
        formatter.format(&doc)
    }
    
    #[test]
    fn test_format_simple_let() {
        let input = "let x=1,y=2 in x+y";
        let output = format_code(input);
        assert!(output.contains("let"));
        assert!(output.contains("in"));
    }
    
    #[test]
    fn test_format_record() {
        let input = "[A=1,B=2]";
        let output = format_code(input);
        assert!(output.contains("["));
        assert!(output.contains("]"));
    }
}
