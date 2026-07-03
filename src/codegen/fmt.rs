use crate::ast::*;

pub fn format_gp(program: &Program, original_source: &str) -> String {
    let mut f = GpFormatter::new(original_source);
    f.emit_program(program);
    f.output
}

/// A comment scanned from the original source, with its byte range and text.
#[derive(Clone, Debug)]
pub struct Comment {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub is_line: bool,
}

struct GpFormatter<'a> {
    output: String,
    source: &'a str,
    comments: Vec<Comment>,
    /// Index of the next comment not yet emitted or consumed.
    next: usize,
}

impl<'a> GpFormatter<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            output: String::new(),
            source,
            comments: scan_comments(source),
            next: 0,
        }
    }

    // ---- comment interleaving helpers ----

    /// Emit every pending comment that starts before `before`, each on its own
    /// line at the given indent.
    fn flush_leading(&mut self, before: usize, tabs: &str) {
        while self.next < self.comments.len() && self.comments[self.next].start < before {
            let text = self.comments[self.next].text.trim_end().to_string();
            self.output.push_str(tabs);
            self.output.push_str(&text);
            self.output.push('\n');
            self.next += 1;
        }
    }

    /// Discard pending comments that start before `end` because they are already
    /// present verbatim in a raw/pass-through region that was just emitted.
    fn consume_within(&mut self, end: usize) {
        while self.next < self.comments.len() && self.comments[self.next].start < end {
            self.next += 1;
        }
    }

    /// If the next pending comment sits on the same source line as `after` (the
    /// source end of the construct just emitted), splice it inline at the end of
    /// the current output line.
    fn flush_trailing(&mut self, after: usize) {
        if self.next >= self.comments.len() {
            return;
        }
        let start = self.comments[self.next].start;
        if start < after || after > self.source.len() || start > self.source.len() {
            return;
        }
        if self.source[after..start].contains('\n') {
            return;
        }
        let text = self.comments[self.next].text.trim_end().to_string();
        if self.output.ends_with('\n') {
            self.output.pop();
        }
        self.output.push(' ');
        self.output.push_str(&text);
        self.output.push('\n');
        self.next += 1;
    }

    fn emit_program(&mut self, program: &Program) {
        // File-header comments that appear before `package`.
        self.flush_leading(program.span.start, "");
        self.output
            .push_str(&format!("package {}\n", program.package));

        if !program.imports.is_empty() {
            self.output.push('\n');
            self.flush_leading(program.imports[0].span.start, "");
            if program.imports.len() == 1 {
                self.emit_import_single(&program.imports[0]);
            } else {
                self.emit_import_group(&program.imports);
            }
        }

        for item in &program.items {
            self.output.push('\n');
            self.flush_leading(item_span(item).start, "");
            self.emit_item(item);
            self.output.push('\n');
        }

        // Footer comments after the last item.
        self.flush_leading(self.source.len(), "");
    }

    fn emit_import_single(&mut self, import: &ImportDecl) {
        self.output.push_str("import ");
        if let Some(alias) = &import.alias {
            self.output.push_str(alias);
            self.output.push(' ');
        }
        self.output.push_str(&format!("\"{}\"", import.path));
        self.output.push('\n');
    }

    fn emit_import_group(&mut self, imports: &[ImportDecl]) {
        self.output.push_str("import (\n");
        for import in imports {
            self.flush_leading(import.span.start, "\t");
            self.output.push('\t');
            if let Some(alias) = &import.alias {
                self.output.push_str(alias);
                self.output.push(' ');
            }
            self.output.push_str(&format!("\"{}\"", import.path));
            self.output.push('\n');
            self.flush_trailing(import.span.end);
        }
        self.output.push_str(")\n");
    }

    fn emit_item(&mut self, item: &Item) {
        match item {
            Item::Struct(decl) => self.emit_struct(decl),
            Item::Enum(decl) => self.emit_enum(decl),
            Item::Function(decl) => self.emit_function(decl),
            Item::Impl(decl) => self.emit_impl(decl),
            Item::Raw(decl) => self.emit_raw(decl),
        }
    }

    fn emit_derives(&mut self, derives: &[DeriveKind]) {
        if derives.is_empty() {
            return;
        }
        let names: Vec<&str> = derives
            .iter()
            .map(|d| match d {
                DeriveKind::String => "String",
                DeriveKind::Debug => "Debug",
                DeriveKind::Equal => "Equal",
                DeriveKind::JsonMarshal => "JsonMarshal",
                DeriveKind::JsonUnmarshal => "JsonUnmarshal",
                DeriveKind::Clone => "Clone",
            })
            .collect();
        self.output
            .push_str(&format!("@derive({})\n", names.join(", ")));
    }

    fn emit_struct(&mut self, decl: &StructDecl) {
        self.emit_derives(&decl.derives);
        self.output.push_str(&format!("struct {} {{\n", decl.name));
        for field in &decl.fields {
            self.flush_leading(field.span.start, "\t");
            let tag = field
                .tag
                .as_ref()
                .map(|tag| format!(" {}", tag))
                .unwrap_or_default();
            self.output
                .push_str(&format!("\t{}: {}{}\n", field.name, field.ty.raw, tag));
            self.flush_trailing(field.span.end);
        }
        self.flush_leading(decl.span.end, "\t");
        self.output.push('}');
    }

    fn emit_enum(&mut self, decl: &EnumDecl) {
        self.emit_derives(&decl.derives);
        self.output.push_str("enum ");
        self.output.push_str(&decl.name);
        if !decl.type_params.is_empty() {
            self.output.push('<');
            self.output.push_str(&decl.type_params.join(", "));
            self.output.push('>');
        }
        self.output.push_str(" {\n");
        for variant in &decl.variants {
            self.flush_leading(variant.span.start, "\t");
            self.output.push('\t');
            self.output.push_str(&variant.name);
            if !variant.payload.is_empty() {
                self.output.push('(');
                let types: Vec<&str> = variant.payload.iter().map(|ty| ty.raw.as_str()).collect();
                self.output.push_str(&types.join(", "));
                self.output.push(')');
            }
            self.output.push('\n');
            self.flush_trailing(variant.span.end);
        }
        self.flush_leading(decl.span.end, "\t");
        self.output.push('}');
    }

    fn emit_decorators(&mut self, decorators: &[Decorator]) {
        for dec in decorators {
            self.output.push('@');
            self.output.push_str(&dec.name);
            if !dec.args.is_empty() {
                self.output.push('(');
                self.output.push_str(&dec.args.join(", "));
                self.output.push(')');
            }
            self.output.push('\n');
        }
    }

    fn emit_function(&mut self, decl: &FnDecl) {
        self.emit_decorators(&decl.decorators);
        self.output.push_str("fn ");
        self.output.push_str(&decl.name);
        if !decl.type_params.is_empty() {
            self.output.push('<');
            self.output.push_str(&decl.type_params.join(", "));
            self.output.push('>');
        }
        self.output.push('(');
        let params: Vec<String> = decl
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.ty.raw))
            .collect();
        self.output.push_str(&params.join(", "));
        self.output.push(')');
        self.emit_return_type(&decl.ret);
        self.output.push_str(" {\n");
        self.emit_block(&decl.body, 1);
        self.output.push('}');
    }

    fn emit_impl(&mut self, decl: &ImplBlock) {
        self.output.push_str(&format!("impl {} {{\n", decl.target));
        for (idx, method) in decl.methods.iter().enumerate() {
            if idx > 0 {
                self.output.push('\n');
            }
            self.flush_leading(method.span.start, "\t");
            self.emit_method(method);
        }
        self.flush_leading(decl.span.end, "\t");
        self.output.push('}');
    }

    fn emit_method(&mut self, method: &MethodDecl) {
        self.output.push('\t');
        for dec in &method.decorators {
            self.output.push('@');
            self.output.push_str(&dec.name);
            if !dec.args.is_empty() {
                self.output.push('(');
                self.output.push_str(&dec.args.join(", "));
                self.output.push(')');
            }
            self.output.push('\n');
            self.output.push('\t');
        }
        self.output.push_str("fn ");
        self.output.push_str(&method.name);
        if !method.type_params.is_empty() {
            self.output.push('<');
            self.output.push_str(&method.type_params.join(", "));
            self.output.push('>');
        }
        self.output.push('(');
        // The receiver is written as the first parameter (`self` / `mut self`),
        // matching the parser (`fn m(self, ...)`), not a `fn mut ` prefix.
        let mut params: Vec<String> = vec![match method.receiver {
            ReceiverKind::Value => "self".to_string(),
            ReceiverKind::Pointer => "mut self".to_string(),
        }];
        params.extend(
            method
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, p.ty.raw)),
        );
        self.output.push_str(&params.join(", "));
        self.output.push(')');
        self.emit_return_type(&method.ret);
        self.output.push_str(" {\n");
        self.emit_block(&method.body, 2);
        self.output.push('\t');
        self.output.push_str("}\n");
    }

    fn emit_return_type(&mut self, ret: &ReturnType) {
        match ret {
            ReturnType::Void => {}
            ReturnType::Type(ty) => {
                self.output.push_str(" -> ");
                self.output.push_str(&ty.raw);
            }
            ReturnType::ErrorOnly => {
                self.output.push_str(" -> !");
            }
            ReturnType::TypeWithError(ty) | ReturnType::GoTypeWithError(ty) => {
                self.output.push_str(" -> ");
                self.output.push_str(&ty.raw);
                self.output.push('!');
            }
        }
    }

    fn emit_block(&mut self, block: &Block, indent: usize) {
        let tabs = "\t".repeat(indent);
        for stmt in &block.stmts {
            self.flush_leading(stmt_span(stmt).start, &tabs);
            self.emit_stmt(stmt, &tabs, indent);
            self.flush_trailing(stmt_span(stmt).end);
        }
        // Comments between the last statement and the block's closing brace.
        self.flush_leading(block.span.end.saturating_sub(1), &tabs);
    }

    fn emit_stmt(&mut self, stmt: &Stmt, tabs: &str, indent: usize) {
        match stmt {
            Stmt::VarDecl(v) => {
                let expr_text = if v.expr.has_try {
                    format!("{}?", v.expr.text)
                } else {
                    v.expr.text.clone()
                };
                self.output
                    .push_str(&format!("{}{} := {}\n", tabs, v.name, expr_text));
            }
            Stmt::Assign(a) => {
                self.output.push_str(&format!("{}{}\n", tabs, a.text));
            }
            Stmt::Return(r) => {
                if r.exprs.is_empty() {
                    self.output.push_str(&format!("{}return\n", tabs));
                } else {
                    let exprs: Vec<String> = r
                        .exprs
                        .iter()
                        .map(|e| {
                            if e.has_try {
                                format!("{}?", e.text)
                            } else {
                                e.text.clone()
                            }
                        })
                        .collect();
                    self.output
                        .push_str(&format!("{}return {}\n", tabs, exprs.join(", ")));
                }
            }
            Stmt::Defer(raw) | Stmt::Go(raw) | Stmt::Raw(raw) => {
                self.output.push_str(&format!("{}{}\n", tabs, raw.text));
                // The raw text is a verbatim source slice that already contains
                // any comments inside it; drop them from the pending list so they
                // are not emitted twice.
                self.consume_within(raw.span.end);
            }
            Stmt::For(for_stmt) => {
                let hdr = if for_stmt.header.is_empty() {
                    String::new()
                } else {
                    format!("{} ", for_stmt.header)
                };
                self.output.push_str(&format!("{}for {}{{\n", tabs, hdr));
                // A comment inside the header is already emitted verbatim as part
                // of `hdr`; drop it from the pending list.
                self.consume_within(for_stmt.body.span.start);
                self.emit_block(&for_stmt.body, indent + 1);
                self.output.push_str(&format!("{}}}\n", tabs));
            }
            Stmt::Switch(switch_stmt) => {
                let hdr = if switch_stmt.header.is_empty() {
                    String::new()
                } else {
                    format!("{} ", switch_stmt.header)
                };
                self.output.push_str(&format!("{}switch {}{{\n", tabs, hdr));
                // Header comments are already verbatim in `hdr`.
                self.consume_within(switch_stmt.header_span.end);
                for case in &switch_stmt.cases {
                    self.flush_leading(case.label_span.start, tabs);
                    self.output.push_str(&format!("{}{}\n", tabs, case.label));
                    // A comment inside the label is already verbatim in it.
                    self.consume_within(case.label_span.end);
                    self.flush_trailing(case.label_span.end);
                    self.emit_block(&case.body, indent + 1);
                }
                self.flush_leading(switch_stmt.span.end.saturating_sub(1), tabs);
                self.output.push_str(&format!("{}}}\n", tabs));
            }
            Stmt::Select(select_stmt) => {
                self.output.push_str(&format!("{}select {{\n", tabs));
                for case in &select_stmt.cases {
                    self.flush_leading(case.label_span.start, tabs);
                    self.output.push_str(&format!("{}{}\n", tabs, case.label));
                    self.consume_within(case.label_span.end);
                    self.flush_trailing(case.label_span.end);
                    self.emit_block(&case.body, indent + 1);
                }
                self.flush_leading(select_stmt.span.end.saturating_sub(1), tabs);
                self.output.push_str(&format!("{}}}\n", tabs));
            }
            Stmt::Expr(e) => {
                let text = if e.expr.has_try {
                    format!("{}?", e.expr.text)
                } else {
                    e.expr.text.clone()
                };
                self.output.push_str(&format!("{}{}\n", tabs, text));
            }
            Stmt::Match(m) => self.emit_match(m, tabs, indent),
            Stmt::If(i) => self.emit_if(i, tabs, indent),
        }
    }

    fn emit_match(&mut self, m: &MatchStmt, tabs: &str, indent: usize) {
        let value = if m.value.has_try {
            format!("{}?", m.value.text)
        } else {
            m.value.text.clone()
        };
        self.output
            .push_str(&format!("{}match {} {{\n", tabs, value));
        for arm in &m.arms {
            self.flush_leading(arm.span.start, &format!("{}\t", tabs));
            self.emit_match_arm(arm, tabs, indent);
        }
        self.output.push_str(&format!("{}}}\n", tabs));
    }

    fn emit_match_arm(&mut self, arm: &MatchArm, tabs: &str, indent: usize) {
        let inner_tabs = format!("{}\t", tabs);
        self.output.push_str(&inner_tabs);
        match &arm.pattern {
            Pattern::Wildcard { .. } => self.output.push('_'),
            Pattern::TypedVariant {
                enum_name,
                variant,
                bindings,
                ..
            } => {
                self.output.push_str(enum_name);
                self.output.push_str("::");
                self.output.push_str(variant);
                if !bindings.is_empty() {
                    self.output.push('(');
                    self.output.push_str(&bindings.join(", "));
                    self.output.push(')');
                }
            }
            Pattern::Variant {
                variant, bindings, ..
            } => {
                self.output.push_str(variant);
                if !bindings.is_empty() {
                    self.output.push('(');
                    self.output.push_str(&bindings.join(", "));
                    self.output.push(')');
                }
            }
        }
        self.output.push_str(" => ");
        match &arm.body {
            MatchArmBody::Expr(expr) => {
                let text = if expr.has_try {
                    format!("{}?", expr.text)
                } else {
                    expr.text.clone()
                };
                self.output.push_str(&text);
                self.output.push('\n');
                self.flush_trailing(arm.span.end);
            }
            MatchArmBody::Block(block) => {
                self.output.push_str("{\n");
                self.emit_block(block, indent + 2);
                self.output.push_str(&format!("{}}}\n", inner_tabs));
            }
        }
    }

    fn emit_if(&mut self, if_stmt: &IfStmt, tabs: &str, indent: usize) {
        let cond = if if_stmt.condition.has_try {
            format!("{}?", if_stmt.condition.text)
        } else {
            if_stmt.condition.text.clone()
        };
        self.output.push_str(&format!("{}if {} {{\n", tabs, cond));
        self.emit_block(&if_stmt.then_block, indent + 1);
        self.output.push_str(&format!("{}}}", tabs));
        if let Some(else_branch) = &if_stmt.else_branch {
            match else_branch {
                ElseBranch::Block(block) => {
                    self.output.push_str(" else {\n");
                    self.emit_block(block, indent + 1);
                    self.output.push_str(&format!("{}}}\n", tabs));
                }
                ElseBranch::If(nested) => {
                    self.output.push_str(" else ");
                    self.emit_if(nested, "", indent);
                }
            }
        } else {
            self.output.push('\n');
        }
    }

    fn emit_raw(&mut self, decl: &RawDecl) {
        self.output.push_str(&decl.text);
        // Verbatim source slice: drop any comments it already contains.
        self.consume_within(decl.span.end);
    }
}

fn item_span(item: &Item) -> &Span {
    match item {
        Item::Struct(d) => &d.span,
        Item::Enum(d) => &d.span,
        Item::Function(d) => &d.span,
        Item::Impl(d) => &d.span,
        Item::Raw(d) => &d.span,
    }
}

fn stmt_span(stmt: &Stmt) -> &Span {
    match stmt {
        Stmt::VarDecl(s) => &s.span,
        Stmt::Assign(s) => &s.span,
        Stmt::Return(s) => &s.span,
        Stmt::Defer(s) | Stmt::Go(s) | Stmt::Raw(s) => &s.span,
        Stmt::For(s) => &s.span,
        Stmt::Switch(s) => &s.span,
        Stmt::Select(s) => &s.span,
        Stmt::Expr(s) => &s.span,
        Stmt::Match(s) => &s.span,
        Stmt::If(s) => &s.span,
    }
}

/// Scan `src` for `//` line comments and `/* */` block comments that are not
/// inside a string or rune literal, returning them in source order.
///
/// The scan is string/rune-literal aware so that `//` inside a literal (e.g. a
/// `"http://..."` URL) is not mistaken for a comment.
pub fn scan_comments(src: &str) -> Vec<Comment> {
    let bytes = src.as_bytes();
    let n = bytes.len();
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        match bytes[i] {
            // interpreted string literal: skip to the closing quote, honoring escapes.
            b'"' => {
                i += 1;
                while i < n {
                    match bytes[i] {
                        b'\\' => i += 2,
                        b'"' => {
                            i += 1;
                            break;
                        }
                        _ => i += 1,
                    }
                }
            }
            // raw string literal: skip to the closing backtick (no escapes).
            b'`' => {
                i += 1;
                while i < n && bytes[i] != b'`' {
                    i += 1;
                }
                i += 1;
            }
            // rune literal: skip to the closing quote, honoring escapes.
            b'\'' => {
                i += 1;
                while i < n {
                    match bytes[i] {
                        b'\\' => i += 2,
                        b'\'' => {
                            i += 1;
                            break;
                        }
                        _ => i += 1,
                    }
                }
            }
            b'/' if i + 1 < n && bytes[i + 1] == b'/' => {
                let start = i;
                while i < n && bytes[i] != b'\n' {
                    i += 1;
                }
                out.push(Comment {
                    start,
                    end: i,
                    text: src[start..i].to_string(),
                    is_line: true,
                });
            }
            b'/' if i + 1 < n && bytes[i + 1] == b'*' => {
                let start = i;
                i += 2;
                while i + 1 < n && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                // include the closing */
                let end = (i + 2).min(n);
                i = end;
                out.push(Comment {
                    start,
                    end,
                    text: src[start..end].to_string(),
                    is_line: false,
                });
            }
            _ => i += 1,
        }
    }
    out
}

/// Returns `true` if `src` contains any comment outside string/rune literals.
pub fn source_has_comments(src: &str) -> bool {
    !scan_comments(src).is_empty()
}

/// The trimmed texts of all comments in `src`, sorted — used to check that a
/// reformat preserved every comment (as a multiset) before overwriting a file.
pub fn comment_texts(src: &str) -> Vec<String> {
    let mut v: Vec<String> = scan_comments(src)
        .into_iter()
        .map(|c| c.text.trim().to_string())
        .collect();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_program;

    fn format_source(input: &str) -> String {
        let program = parse_program(input).expect("parse");
        format_gp(&program, input)
    }

    #[test]
    fn format_round_trip_simple() {
        let input = "package main\n\nimport \"fmt\"\n\nfn main() {\n\tfmt.Println(\"hello\")\n}\n";
        let formatted = format_source(input);
        assert!(formatted.contains("package main"));
        assert!(formatted.contains("import \"fmt\""));
        assert!(formatted.contains("fn main()"));
    }

    #[test]
    fn format_enum_with_derive() {
        let input = r#"package main

@derive(String)
enum Status {
    Pending
    Running
    Done
}
"#;
        let formatted = format_source(input);
        assert!(formatted.contains("@derive(String)"));
        assert!(formatted.contains("enum Status {"));
        assert!(formatted.contains("\tPending"));
    }

    #[test]
    fn format_decorated_function() {
        let input = r#"package main

import "fmt"

@log
@retry(3, 100)
fn readName() -> string! {
    return "goplus"
}
"#;
        let formatted = format_source(input);
        assert!(formatted.contains("@log"));
        assert!(formatted.contains("@retry(3, 100)"));
        assert!(formatted.contains("fn readName() -> string!"));
    }

    // ---- comment preservation ----

    #[test]
    fn preserves_leading_comment_on_item() {
        let input = "package main\n\n// greet builds a greeting\nfn greet() {\n}\n";
        let formatted = format_source(input);
        assert!(
            formatted.contains("// greet builds a greeting"),
            "leading comment lost:\n{formatted}"
        );
        assert!(comment_texts(input) == comment_texts(&formatted));
    }

    #[test]
    fn preserves_trailing_comment_on_stmt() {
        let input = "package main\n\nfn main() {\n\tx := 1 // count\n}\n";
        let formatted = format_source(input);
        assert!(
            formatted.contains("x := 1 // count"),
            "trailing comment lost:\n{formatted}"
        );
        assert!(comment_texts(input) == comment_texts(&formatted));
    }

    #[test]
    fn preserves_field_comments() {
        let input = "package main\n\nstruct Point {\n\t// the x coordinate\n\tx: int\n\ty: int // vertical\n}\n";
        let formatted = format_source(input);
        assert!(formatted.contains("// the x coordinate"), "{formatted}");
        assert!(formatted.contains("y: int // vertical"), "{formatted}");
        assert!(comment_texts(input) == comment_texts(&formatted));
    }

    #[test]
    fn preserves_url_in_string_not_treated_as_comment() {
        let input = "package main\n\nfn main() {\n\turl := \"http://example.com\"\n}\n";
        let formatted = format_source(input);
        assert!(formatted.contains("http://example.com"));
        assert!(comment_texts(&formatted).is_empty());
    }

    #[test]
    fn scan_finds_line_and_block_comments() {
        let src = "a // one\n/* two */ b\n";
        let cs = scan_comments(src);
        assert_eq!(cs.len(), 2);
        assert_eq!(cs[0].text, "// one");
        assert!(cs[0].is_line);
        assert_eq!(cs[1].text, "/* two */");
        assert!(!cs[1].is_line);
    }

    #[test]
    fn scan_ignores_slashes_in_strings() {
        assert!(scan_comments("u := \"a//b\"\n").is_empty());
        assert!(scan_comments("p := `a // b /* c */`\n").is_empty());
    }

    #[test]
    fn source_has_comments_matches_scan() {
        assert!(source_has_comments("x // y"));
        assert!(!source_has_comments("x / y"));
    }

    fn gp_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().and_then(|n| n.to_str()) == Some(".goplusgen") {
                    continue;
                }
                gp_files(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("gp") {
                out.push(path);
            }
        }
    }

    /// Every example must format back to valid GoPlus that (a) keeps all its
    /// comments and (b) is idempotent: `fmt(fmt(x)) == fmt(x)`. This is the
    /// golden guard that keeps the rewriting formatter trustworthy.
    #[test]
    fn examples_round_trip_and_preserve_comments() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
        let mut files = Vec::new();
        gp_files(&dir, &mut files);
        assert!(
            !files.is_empty(),
            "no example .gp files found under {dir:?}"
        );

        let mut checked = 0;
        for file in files {
            let src = std::fs::read_to_string(&file).expect("read example");
            let Ok(prog1) = parse_program(&src) else {
                continue; // not parseable standalone; skip
            };
            let f1 = format_gp(&prog1, &src);

            assert_eq!(
                comment_texts(&src),
                comment_texts(&f1),
                "formatting changed the comment set for {file:?}"
            );

            let prog2 = parse_program(&f1).unwrap_or_else(|diags| {
                panic!("reformatted output of {file:?} does not re-parse: {diags:?}\n---\n{f1}")
            });
            let f2 = format_gp(&prog2, &f1);
            assert_eq!(f1, f2, "formatter is not idempotent for {file:?}");
            checked += 1;
        }
        assert!(checked > 0, "no examples were round-trip checked");
    }
}
