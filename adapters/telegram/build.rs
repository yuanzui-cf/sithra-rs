use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use serde_json::{Map, Value};

fn main() {
    println!("cargo:rerun-if-changed=spec/api.json");

    let api_json = fs::read_to_string("spec/api.json").expect("Failed to read spec/api.json");

    let api: Value = serde_json::from_str(&api_json).expect("Failed to parse spec/api.json");

    let types = api["types"].as_object().expect("Missing types section in API spec");

    let mut rust_code = String::new();

    // File header
    rust_code.push_str("// Telegram Bot API Types\n");
    rust_code.push_str("//\n");
    rust_code.push_str("// Auto-generated from the Telegram Bot API specification.\n");
    rust_code.push_str("// DO NOT EDIT MANUALLY!\n\n");
    rust_code.push_str("use serde::{Deserialize, Serialize};\n\n");

    // Generate all types
    for (type_name, type_info) in types {
        let type_obj = type_info.as_object().unwrap();
        rust_code.push_str(&generate_rust_struct(type_name, type_obj));
        rust_code.push_str("\n\n");
    }

    // Write to output file
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("tgapi_types.rs");
    let mut f = File::create(&dest_path).unwrap();
    f.write_all(rust_code.as_bytes()).unwrap();

    println!("Generated {} types", types.len());
}

fn generate_rust_struct(name: &str, type_info: &Map<String, Value>) -> String {
    let mut lines = Vec::new();

    // Generate documentation
    if let Some(description) = type_info.get("description") {
        if let Some(desc_array) = description.as_array() {
            for desc_line in desc_array {
                if let Some(desc_str) = desc_line.as_str() {
                    let desc_str = desc_str.trim();
                    if !desc_str.is_empty() {
                        // Split long description lines
                        for line in wrap_text(desc_str, 77) {
                            lines.push(format!("/// {line}"));
                        }
                    }
                }
            }
        }
    }

    // Generate struct
    lines.push("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]".to_owned());
    lines.push(format!("pub struct {name} {{"));

    // Generate fields
    if let Some(fields) = type_info.get("fields") {
        if let Some(fields_array) = fields.as_array() {
            for field in fields_array {
                if let Some(field_obj) = field.as_object() {
                    lines.extend(generate_rust_field(field_obj));
                }
            }
        }
    }

    lines.push("}".to_owned());

    lines.join("\n")
}

fn generate_rust_field(field: &Map<String, Value>) -> Vec<String> {
    let mut lines = Vec::new();

    let field_name = field.get("name").unwrap().as_str().unwrap();
    let field_types = field.get("types").unwrap().as_array().unwrap();
    let required = field.get("required").unwrap().as_bool().unwrap();
    let description = field.get("description").map(|d| d.as_str().unwrap_or(""));

    let rust_field_name = to_snake_case(field_name);
    let rust_field_name = if is_rust_keyword(&rust_field_name) {
        format!("{rust_field_name}_")
    } else {
        rust_field_name
    };

    // Generate field documentation
    if let Some(desc) = description {
        if !desc.is_empty() {
            for line in wrap_text(desc, 73) {
                lines.push(format!("    /// {line}"));
            }
        }
    }

    // Add serde rename if needed
    if rust_field_name != field_name {
        lines.push(format!("    #[serde(rename = \"{field_name}\")]"));
    }

    // Generate field type
    let rust_type = if field_types.len() == 1 {
        map_telegram_type_to_rust(field_types[0].as_str().unwrap())
    } else {
        // For union types, use the first non-String type or String if all are String
        let types_str: Vec<&str> = field_types.iter().map(|t| t.as_str().unwrap()).collect();

        for t in &types_str {
            if *t != "String" {
                let union_types = types_str.join(" | ");
                return vec![format!("    // TODO: Handle union type: {}", union_types)];
            }
        }
        "String".to_owned()
    };

    // Handle recursive types by boxing them
    // For simplicity, we'll box all Message fields in other structs to break cycles
    let final_type = if should_box_type(&rust_type) {
        if required {
            format!("Box<{rust_type}>")
        } else {
            format!("Option<Box<{rust_type}>>")
        }
    } else if required {
        rust_type
    } else {
        format!("Option<{rust_type}>")
    };

    lines.push(format!("    pub {rust_field_name}: {final_type},"));

    lines
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let chars = s.chars();
    let mut first = true;

    for ch in chars {
        if ch.is_uppercase() {
            if !first {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
        first = false;
    }

    result
}

fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "async"
            | "await"
            | "try"
    )
}

fn map_telegram_type_to_rust(tg_type: &str) -> String {
    // Handle arrays
    if let Some(inner_type) = tg_type.strip_prefix("Array of ") {
        return format!("Vec<{}>", map_telegram_type_to_rust(inner_type));
    }

    // Handle primitive types
    match tg_type {
        "Integer" | "Boolean" | "Float" | "True" => match tg_type {
            "Integer" => "i64".to_owned(),
            "Boolean" | "True" => "bool".to_owned(),
            "Float" => "f64".to_owned(),
            _ => unreachable!(),
        },
        "String" | "InputFile" => "String".to_owned(), // Simplified
        _ => {
            // Handle union types
            if tg_type.contains(" or ") {
                let types: Vec<&str> = tg_type.split(" or ").collect();
                // Use the first non-String type, or String if all are String
                for t in &types {
                    let mapped = map_telegram_type_to_rust(t.trim());
                    if mapped != "String" {
                        return mapped;
                    }
                }
                return "String".to_owned();
            }

            // For custom types, assume they are structs
            tg_type.to_owned()
        }
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in words {
        if current_line.is_empty() {
            // First word in line
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
        } else {
            lines.push(std::mem::take(&mut current_line));
        }
        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn should_box_type(rust_type: &str) -> bool {
    // Box certain types that are commonly involved in recursive relationships
    // This is a conservative approach to break potential cycles
    matches!(
        rust_type,
        "Message"
            | "ChecklistTasksDone"
            | "ChecklistTasksAdded"
            | "ChecklistTasks"
            | "Story"
            | "ForwardOrigin"
            | "MessageOrigin"
            | "ExternalReplyInfo"
    )
}
