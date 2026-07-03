use super::*;

impl<'a> GoGenerator<'a> {
    /// Emit all derive implementations for a struct.
    pub(super) fn emit_struct_derives(&mut self, decl: &StructDecl) -> String {
        let mut out = String::new();
        for derive in &decl.derives {
            match derive {
                DeriveKind::String => {} // Handled in emit_struct directly
                DeriveKind::Debug => {
                    out.push_str("\n\n");
                    out.push_str(&self.emit_struct_debug(decl));
                }
                DeriveKind::Equal => {
                    out.push_str("\n\n");
                    out.push_str(&self.emit_struct_equal(decl));
                }
                DeriveKind::JsonMarshal => {
                    out.push_str("\n\n");
                    out.push_str(&self.emit_struct_json_marshal(decl));
                }
                DeriveKind::JsonUnmarshal => {
                    out.push_str("\n\n");
                    out.push_str(&self.emit_struct_json_unmarshal(decl));
                }
                DeriveKind::Clone => {
                    out.push_str("\n\n");
                    out.push_str(&self.emit_struct_clone(decl));
                }
            }
        }
        out
    }

    /// Emit all derive implementations for an enum.
    pub(super) fn emit_enum_derives(&mut self, decl: &EnumDecl) -> String {
        let mut out = String::new();
        for derive in &decl.derives {
            match derive {
                DeriveKind::String => {} // Handled in emit_enum directly
                DeriveKind::Debug => {
                    out.push_str("\n\n");
                    if decl.is_tagged() {
                        out.push_str(&self.emit_tagged_enum_debug(decl));
                    } else {
                        out.push_str(&self.emit_simple_enum_debug(decl));
                    }
                }
                DeriveKind::Equal => {
                    out.push_str("\n\n");
                    if decl.is_tagged() {
                        out.push_str(&self.emit_tagged_enum_equal(decl));
                    } else {
                        out.push_str(&self.emit_simple_enum_equal(decl));
                    }
                }
                DeriveKind::JsonMarshal => {
                    out.push_str("\n\n");
                    out.push_str(&self.emit_enum_json_marshal(decl));
                }
                DeriveKind::JsonUnmarshal => {
                    out.push_str("\n\n");
                    out.push_str(&self.emit_enum_json_unmarshal(decl));
                }
                DeriveKind::Clone => {
                    out.push_str("\n\n");
                    out.push_str(&self.emit_enum_clone(decl));
                }
            }
        }
        out
    }

    // --- Debug derive ---

    fn emit_struct_debug(&mut self, decl: &StructDecl) -> String {
        let fmt_name = self.import_binding("fmt", "fmt");
        if decl.fields.is_empty() {
            return format!(
                "func (s {name}) Debug() string {{\n\treturn {fmt}.Sprintf(\"{name}{{}}\")\n}}",
                name = decl.name,
                fmt = fmt_name
            );
        }
        let format_parts: Vec<String> = decl
            .fields
            .iter()
            .map(|f| format!("{}:%+v", f.name))
            .collect();
        let args: Vec<String> = decl
            .fields
            .iter()
            .map(|f| format!("s.{}", f.name))
            .collect();
        format!(
            "func (s {name}) Debug() string {{\n\treturn {fmt}.Sprintf(\"{name}{{{parts}}}\", {args})\n}}",
            name = decl.name,
            fmt = fmt_name,
            parts = format_parts.join(", "),
            args = args.join(", ")
        )
    }

    fn emit_simple_enum_debug(&mut self, decl: &EnumDecl) -> String {
        let fmt_name = self.import_binding("fmt", "fmt");
        let mut out = String::new();
        out.push_str(&format!(
            "func (e {}) Debug() string {{\n\treturn {}.Sprintf(\"%s(%d)\", e.String(), int(e))\n}}",
            decl.name, fmt_name
        ));
        out
    }

    fn emit_tagged_enum_debug(&mut self, decl: &EnumDecl) -> String {
        let fmt_name = self.import_binding("fmt", "fmt");
        let mut out = String::new();
        out.push_str(&format!(
            "func (e {}{}) Debug() string {{\n",
            decl.name,
            render_type_params(&decl.type_params)
        ));
        out.push_str("\tswitch e.tag {\n");
        for variant in &decl.variants {
            let tag = format!("{}Tag{}", decl.name, variant.name);
            out.push_str(&format!("\tcase {}:\n", tag));
            if variant.payload.is_empty() {
                out.push_str(&format!(
                    "\t\treturn \"{}::{}()\"\n",
                    decl.name, variant.name
                ));
            } else {
                let fields: Vec<String> = variant
                    .payload
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!("e.{}{}", lower_ident(&variant.name), i))
                    .collect();
                let placeholders: Vec<&str> = variant.payload.iter().map(|_| "%+v").collect();
                out.push_str(&format!(
                    "\t\treturn {}.Sprintf(\"{}::{}({})\", {})\n",
                    fmt_name,
                    decl.name,
                    variant.name,
                    placeholders.join(", "),
                    fields.join(", ")
                ));
            }
        }
        out.push_str(&format!(
            "\tdefault:\n\t\treturn \"{}(?)\"\n\t}}\n}}",
            decl.name
        ));
        out
    }

    // --- Equal derive ---

    fn emit_struct_equal(&self, decl: &StructDecl) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "func (s {name}) Equal(other {name}) bool {{\n",
            name = decl.name
        ));
        if decl.fields.is_empty() {
            out.push_str("\treturn true\n}");
            return out;
        }
        let conditions: Vec<String> = decl
            .fields
            .iter()
            .map(|f| format!("s.{} == other.{}", f.name, f.name))
            .collect();
        out.push_str(&format!("\treturn {}\n}}", conditions.join(" && ")));
        out
    }

    fn emit_simple_enum_equal(&self, decl: &EnumDecl) -> String {
        format!(
            "func (e {name}) Equal(other {name}) bool {{\n\treturn e == other\n}}",
            name = decl.name
        )
    }

    fn emit_tagged_enum_equal(&self, decl: &EnumDecl) -> String {
        let type_params = render_type_params(&decl.type_params);
        let mut out = String::new();
        out.push_str(&format!(
            "func (e {name}{tp}) Equal(other {name}{tp}) bool {{\n",
            name = decl.name,
            tp = type_params
        ));
        out.push_str("\tif e.tag != other.tag {\n\t\treturn false\n\t}\n");
        out.push_str("\tswitch e.tag {\n");
        for variant in &decl.variants {
            let tag = format!("{}Tag{}", decl.name, variant.name);
            out.push_str(&format!("\tcase {}:\n", tag));
            if variant.payload.is_empty() {
                out.push_str("\t\treturn true\n");
            } else {
                let conditions: Vec<String> = variant
                    .payload
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let field = format!("{}{}", lower_ident(&variant.name), i);
                        format!("e.{} == other.{}", field, field)
                    })
                    .collect();
                out.push_str(&format!("\t\treturn {}\n", conditions.join(" && ")));
            }
        }
        out.push_str("\tdefault:\n\t\treturn false\n\t}\n}");
        out
    }

    // --- Clone derive ---

    /// Deep-copies top-level slice and map fields (which a plain struct copy
    /// would share); scalar/struct/pointer fields keep Go's value-copy semantics.
    fn emit_struct_clone(&self, decl: &StructDecl) -> String {
        let name = &decl.name;
        let mut out = format!("func (s {name}) Clone() {name} {{\n\tout := s\n");
        for field in &decl.fields {
            let ty = render_type_ref(&field.ty);
            let ty = ty.trim();
            let f = &field.name;
            if ty.starts_with("[]") {
                out.push_str(&format!("\tout.{f} = append({ty}(nil), s.{f}...)\n"));
            } else if ty.starts_with("map[") {
                out.push_str(&format!(
                    "\tif s.{f} != nil {{\n\t\tout.{f} = make({ty}, len(s.{f}))\n\t\tfor k, v := range s.{f} {{\n\t\t\tout.{f}[k] = v\n\t\t}}\n\t}}\n"
                ));
            }
        }
        out.push_str("\treturn out\n}");
        out
    }

    /// A `Clone` for enums returns a value copy. Simple enums are integers, so
    /// this is a full copy; tagged-enum payloads are copied shallowly.
    fn emit_enum_clone(&self, decl: &EnumDecl) -> String {
        let type_params = render_type_params(&decl.type_params);
        format!(
            "func (e {name}{tp}) Clone() {name}{tp} {{\n\treturn e\n}}",
            name = decl.name,
            tp = type_params
        )
    }

    // --- JSON derives ---

    fn emit_struct_json_marshal(&mut self, decl: &StructDecl) -> String {
        let json_name = self.import_binding("encoding/json", "json");
        format!(
            "func (s {name}) MarshalJSON() ([]byte, error) {{\n\treturn {json}.Marshal(struct {{\n{fields}\n\t}}{{{inits}}})\n}}",
            name = decl.name,
            json = json_name,
            fields = decl
                .fields
                .iter()
                .map(|f| format!(
                    "\t\t{} {} {}",
                    f.name,
                    render_type_ref(&f.ty),
                    json_field_tag(f)
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            inits = decl
                .fields
                .iter()
                .map(|f| format!("s.{}", f.name))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn emit_struct_json_unmarshal(&mut self, decl: &StructDecl) -> String {
        let json_name = self.import_binding("encoding/json", "json");
        let mut out = String::new();
        out.push_str(&format!(
            "func (s *{name}) UnmarshalJSON(data []byte) error {{\n",
            name = decl.name
        ));
        out.push_str("\tvar raw struct {\n");
        for field in &decl.fields {
            out.push_str(&format!(
                "\t\t{} {} {}\n",
                field.name,
                render_type_ref(&field.ty),
                json_field_tag(field)
            ));
        }
        out.push_str("\t}\n");
        out.push_str(&format!(
            "\tif err := {}.Unmarshal(data, &raw); err != nil {{\n\t\treturn err\n\t}}\n",
            json_name
        ));
        for field in &decl.fields {
            out.push_str(&format!("\ts.{} = raw.{}\n", field.name, field.name));
        }
        out.push_str("\treturn nil\n}");
        out
    }

    fn emit_enum_json_marshal(&mut self, decl: &EnumDecl) -> String {
        let json_name = self.import_binding("encoding/json", "json");
        if decl.is_tagged() {
            let mut out = String::new();
            out.push_str(&format!(
                "func (e {}{}) MarshalJSON() ([]byte, error) {{\n",
                decl.name,
                render_type_params(&decl.type_params)
            ));
            out.push_str("\tswitch e.tag {\n");
            for variant in &decl.variants {
                let tag = format!("{}Tag{}", decl.name, variant.name);
                out.push_str(&format!("\tcase {}:\n", tag));
                if variant.payload.is_empty() {
                    out.push_str(&format!(
                        "\t\treturn {}.Marshal(map[string]any{{\"type\": \"{}\"}})\n",
                        json_name, variant.name
                    ));
                } else {
                    let fields: Vec<String> = variant
                        .payload
                        .iter()
                        .enumerate()
                        .map(|(i, _)| {
                            format!("\"value{}\": e.{}{}", i, lower_ident(&variant.name), i)
                        })
                        .collect();
                    out.push_str(&format!(
                        "\t\treturn {}.Marshal(map[string]any{{\"type\": \"{}\", {}}})\n",
                        json_name,
                        variant.name,
                        fields.join(", ")
                    ));
                }
            }
            let fmt_name = self.import_binding("fmt", "fmt");
            out.push_str(&format!(
                "\tdefault:\n\t\treturn nil, {}.Errorf(\"unknown {} tag: %d\", e.tag)\n\t}}\n}}",
                fmt_name, decl.name
            ));
            out
        } else {
            format!(
                "func (e {name}) MarshalJSON() ([]byte, error) {{\n\treturn {json}.Marshal(e.String())\n}}",
                name = decl.name,
                json = json_name
            )
        }
    }

    fn emit_enum_json_unmarshal(&mut self, decl: &EnumDecl) -> String {
        let json_name = self.import_binding("encoding/json", "json");
        // Both branches emit an error in their `default:`; `Errorf` lives in fmt.
        let fmt_name = self.import_binding("fmt", "fmt");
        if decl.is_tagged() {
            let mut out = String::new();
            out.push_str(&format!(
                "func (e *{}{}) UnmarshalJSON(data []byte) error {{\n",
                decl.name,
                render_type_params(&decl.type_params)
            ));
            out.push_str(&format!(
                "\tvar raw map[string]{}interface{{}}{}\n",
                "{", "}"
            ));
            out.push_str(&format!(
                "\tif err := {}.Unmarshal(data, &raw); err != nil {{\n\t\treturn err\n\t}}\n",
                json_name
            ));
            out.push_str("\tswitch raw[\"type\"] {\n");
            for variant in &decl.variants {
                out.push_str(&format!("\tcase \"{}\":\n", variant.name));
                out.push_str(&format!("\t\te.tag = {}Tag{}\n", decl.name, variant.name));
            }
            out.push_str(&format!(
                "\tdefault:\n\t\treturn {}.Errorf(\"unknown {} type: %v\", raw[\"type\"])\n\t}}\n",
                fmt_name, decl.name
            ));
            out.push_str("\treturn nil\n}");
            out
        } else {
            let mut out = String::new();
            out.push_str(&format!(
                "func (e *{name}) UnmarshalJSON(data []byte) error {{\n",
                name = decl.name
            ));
            out.push_str("\tvar s string\n");
            out.push_str(&format!(
                "\tif err := {}.Unmarshal(data, &s); err != nil {{\n\t\treturn err\n\t}}\n",
                json_name
            ));
            out.push_str("\tswitch s {\n");
            for variant in &decl.variants {
                out.push_str(&format!(
                    "\tcase \"{}\":\n\t\t*e = {}{}\n",
                    variant.name, decl.name, variant.name
                ));
            }
            out.push_str(&format!(
                "\tdefault:\n\t\treturn {}.Errorf(\"unknown {} value: %s\", s)\n\t}}\n",
                fmt_name, decl.name
            ));
            out.push_str("\treturn nil\n}");
            out
        }
    }
}

/// The struct tag to use for a field in a JSON derive: the field's explicit tag
/// verbatim if it has one (so `json:"id"`, `omitempty`, etc. are honored), else a
/// generated `json:"<snake_case>"`.
fn json_field_tag(field: &FieldDecl) -> String {
    match &field.tag {
        Some(tag) => tag.clone(),
        None => format!("`json:\"{}\"`", to_json_key(&field.name)),
    }
}

fn to_json_key(name: &str) -> String {
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        if i == 0 {
            out.push(ch.to_ascii_lowercase());
        } else if ch.is_uppercase() {
            out.push('_');
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}
