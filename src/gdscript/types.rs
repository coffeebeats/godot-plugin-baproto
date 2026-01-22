use std::collections::HashSet;

use baproto::NativeType;

/* -------------------------------------------------------------------------- */
/*                                Fn: type_name                               */
/* -------------------------------------------------------------------------- */

/// `type_name` returns the GDScript type name for a native type.
pub fn type_name(native: &NativeType) -> String {
    match native {
        NativeType::Bool => "bool".to_string(),
        NativeType::Int { .. } => "int".to_string(),
        NativeType::Float { .. } => "float".to_string(),
        NativeType::String => "String".to_string(),
        NativeType::Bytes => "PackedByteArray".to_string(),
        NativeType::Array { element } => {
            format!("Array[{}]", type_name(&element.native))
        }
        NativeType::Map { .. } => "Dictionary".to_string(),
        // For message types, we use the file stem as the type name.
        NativeType::Message { descriptor } => descriptor.path.join("_"),
        // Enums are represented as int in GDScript.
        NativeType::Enum { .. } => "int".to_string(),
    }
}

/* -------------------------------------------------------------------------- */
/*                              Fn: default_value                             */
/* -------------------------------------------------------------------------- */

/// `default_value` returns the GDScript default value for a native type.
pub fn default_value(native: &NativeType) -> String {
    match native {
        NativeType::Bool => "false".to_string(),
        NativeType::Int { .. } => "0".to_string(),
        NativeType::Float { .. } => "0.0".to_string(),
        NativeType::String => "\"\"".to_string(),
        NativeType::Bytes => "PackedByteArray()".to_string(),
        NativeType::Array { .. } => "[]".to_string(),
        NativeType::Map { .. } => "{}".to_string(),
        NativeType::Message { .. } => "null".to_string(),
        NativeType::Enum { .. } => "0".to_string(),
    }
}

/* -------------------------------------------------------------------------- */
/*                               Fn: pkg_to_path                              */
/* -------------------------------------------------------------------------- */

/// `pkg_to_path` converts a package name (as slice) to a directory path.
pub fn pkg_to_path(pkg: &[String]) -> String {
    pkg.join("/")
}

/* -------------------------------------------------------------------------- */
/*                       Fn: collect_field_dependencies                       */
/* -------------------------------------------------------------------------- */

/// `collect_field_dependencies` collects all external type dependencies from
/// the fields of a message (message and enum types that need preloads).
///
/// Returns a vector of `(const_name, file_stem, preload_path)` tuples.
pub fn collect_field_dependencies(
    fields: &[baproto::Field],
    current_pkg: &[String],
    current_file_stem: &str,
) -> Vec<(String, String, String)> {
    let mut seen = HashSet::new();
    let mut deps = Vec::new();

    for field in fields {
        collect_native_dependencies(
            &field.encoding.native,
            current_pkg,
            current_file_stem,
            &mut seen,
            &mut deps,
        );
    }

    deps
}

/* -------------------------------------------------------------------------- */
/*                       Fn: collect_native_dependencies                      */
/* -------------------------------------------------------------------------- */

/// `collect_native_dependencies` recursively collects type dependencies from
/// a native type.
fn collect_native_dependencies(
    native: &NativeType,
    current_pkg: &[String],
    current_file_stem: &str,
    seen: &mut HashSet<String>,
    deps: &mut Vec<(String, String, String)>,
) {
    match native {
        NativeType::Message { descriptor } | NativeType::Enum { descriptor } => {
            let file_stem = descriptor.path.join("_");

            // Skip if this is a nested type within the current message.
            if file_stem.starts_with(&format!("{}_", current_file_stem)) {
                return;
            }

            // Skip if already seen.
            if !seen.insert(file_stem.clone()) {
                return;
            }

            let path = resolve_preload_path(&descriptor.package, &descriptor.path, current_pkg);
            let const_name = file_stem.clone();
            deps.push((const_name, file_stem, path));
        }
        NativeType::Array { element } => {
            collect_native_dependencies(
                &element.native,
                current_pkg,
                current_file_stem,
                seen,
                deps,
            );
        }
        NativeType::Map { key, value } => {
            collect_native_dependencies(&key.native, current_pkg, current_file_stem, seen, deps);
            collect_native_dependencies(&value.native, current_pkg, current_file_stem, seen, deps);
        }
        _ => {}
    }
}

/* -------------------------------------------------------------------------- */
/*                          Fn: resolve_preload_path                          */
/* -------------------------------------------------------------------------- */

/// `resolve_preload_path` computes the relative preload path from a type in
/// `current_pkg` to a type at `target_pkg` with the given `target_path`.
fn resolve_preload_path(
    target_pkg: &[String],
    target_path: &[String],
    current_pkg: &[String],
) -> String {
    let target_stem = target_path.join("_");

    if target_pkg == current_pkg {
        // Same package - sibling file.
        format!("./{}.gd", target_stem)
    } else {
        // Cross-package - compute relative path.
        let current_depth = current_pkg.len();
        let up = "../".repeat(current_depth);
        let target_dir = pkg_to_path(target_pkg);
        format!("{}{}/{}.gd", up, target_dir, target_stem)
    }
}

/* -------------------------------------------------------------------------- */
/*                             Fn: escape_keyword                             */
/* -------------------------------------------------------------------------- */

/// GDScript reserved keywords.
const GDSCRIPT_KEYWORDS: &[&str] = &[
    "and",
    "as",
    "assert",
    "await",
    "break",
    "breakpoint",
    "class",
    "class_name",
    "const",
    "continue",
    "elif",
    "else",
    "enum",
    "extends",
    "false",
    "for",
    "func",
    "get",
    "if",
    "in",
    "is",
    "match",
    "not",
    "null",
    "or",
    "pass",
    "preload",
    "return",
    "self",
    "set",
    "signal",
    "static",
    "super",
    "true",
    "var",
    "while",
    "yield",
];

/// `escape_keyword` appends an underscore to identifiers that conflict with
/// GDScript keywords.
pub fn escape_keyword(name: &str) -> String {
    if GDSCRIPT_KEYWORDS.contains(&name) {
        format!("{}_", name)
    } else {
        name.to_string()
    }
}

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use baproto::{Encoding, WireFormat};

    /* -------------------------- Tests: type_name -------------------------- */

    #[test]
    fn test_type_name_bool() {
        // Given: A bool native type.
        let native = NativeType::Bool;

        // When: Getting the type name.
        let result = type_name(&native);

        // Then: It should be "bool".
        assert_eq!(result, "bool");
    }

    #[test]
    fn test_type_name_int() {
        // Given: An int native type.
        let native = NativeType::Int {
            bits: 32,
            signed: true,
        };

        // When: Getting the type name.
        let result = type_name(&native);

        // Then: It should be "int".
        assert_eq!(result, "int");
    }

    #[test]
    fn test_type_name_float() {
        // Given: A float native type.
        let native = NativeType::Float { bits: 32 };

        // When: Getting the type name.
        let result = type_name(&native);

        // Then: It should be "float".
        assert_eq!(result, "float");
    }

    #[test]
    fn test_type_name_string() {
        // Given: A string native type.
        let native = NativeType::String;

        // When: Getting the type name.
        let result = type_name(&native);

        // Then: It should be "String".
        assert_eq!(result, "String");
    }

    #[test]
    fn test_type_name_bytes() {
        // Given: A bytes native type.
        let native = NativeType::Bytes;

        // When: Getting the type name.
        let result = type_name(&native);

        // Then: It should be "PackedByteArray".
        assert_eq!(result, "PackedByteArray");
    }

    #[test]
    fn test_type_name_array() {
        // Given: An array of ints.
        let native = NativeType::Array {
            element: Box::new(Encoding {
                wire: WireFormat::Bits { count: 32 },
                native: NativeType::Int {
                    bits: 32,
                    signed: true,
                },
                transforms: vec![],
                padding_bits: None,
            }),
        };

        // When: Getting the type name.
        let result = type_name(&native);

        // Then: It should be "Array[int]".
        assert_eq!(result, "Array[int]");
    }

    #[test]
    fn test_type_name_map() {
        // Given: A map type.
        let native = NativeType::Map {
            key: Box::new(Encoding {
                wire: WireFormat::LengthPrefixed { prefix_bits: 16 },
                native: NativeType::String,
                transforms: vec![],
                padding_bits: None,
            }),
            value: Box::new(Encoding {
                wire: WireFormat::Bits { count: 32 },
                native: NativeType::Int {
                    bits: 32,
                    signed: true,
                },
                transforms: vec![],
                padding_bits: None,
            }),
        };

        // When: Getting the type name.
        let result = type_name(&native);

        // Then: It should be "Dictionary".
        assert_eq!(result, "Dictionary");
    }

    #[test]
    fn test_type_name_enum() {
        // Given: An enum reference type (we can't construct Descriptor directly,
        // but we can test other aspects of the function).
        let native = NativeType::Int {
            bits: 8,
            signed: false,
        };

        // When: Getting the type name for an int (which is how enums are represented).
        let result = type_name(&native);

        // Then: It should be "int".
        assert_eq!(result, "int");
    }

    /* ------------------------ Tests: default_value ------------------------ */

    #[test]
    fn test_default_value_bool() {
        // Given: A bool native type.
        let native = NativeType::Bool;

        // When: Getting the default value.
        let result = default_value(&native);

        // Then: It should be "false".
        assert_eq!(result, "false");
    }

    #[test]
    fn test_default_value_int() {
        // Given: An int native type.
        let native = NativeType::Int {
            bits: 32,
            signed: true,
        };

        // When: Getting the default value.
        let result = default_value(&native);

        // Then: It should be "0".
        assert_eq!(result, "0");
    }

    #[test]
    fn test_default_value_string() {
        // Given: A string native type.
        let native = NativeType::String;

        // When: Getting the default value.
        let result = default_value(&native);

        // Then: It should be empty string literal.
        assert_eq!(result, "\"\"");
    }

    /* --------------------- Tests: resolve_preload_path -------------------- */

    #[test]
    fn test_resolve_preload_path_same_package() {
        // Given: A target in the same package.
        let current_pkg = vec!["game".to_string(), "player".to_string()];
        let target_pkg = current_pkg.clone();
        let target_path = vec!["Inventory".to_string()];

        // When: Resolving the preload path.
        let result = resolve_preload_path(&target_pkg, &target_path, &current_pkg);

        // Then: It should be a relative path to sibling file.
        assert_eq!(result, "./Inventory.gd");
    }

    #[test]
    fn test_resolve_preload_path_different_package() {
        // Given: A target in a different package.
        let current_pkg = vec!["game".to_string(), "player".to_string()];
        let target_pkg = vec!["other".to_string(), "pkg".to_string()];
        let target_path = vec!["Inventory".to_string()];

        // When: Resolving the preload path.
        let result = resolve_preload_path(&target_pkg, &target_path, &current_pkg);

        // Then: It should use relative path with parent traversal.
        assert_eq!(result, "../../other/pkg/Inventory.gd");
    }

    /* ------------------------ Tests: escape_keyword ----------------------- */

    #[test]
    fn test_escape_keyword_reserved() {
        // Given: A reserved keyword.
        // When: Escaping it.
        let result = escape_keyword("class");

        // Then: It should have an underscore appended.
        assert_eq!(result, "class_");
    }

    #[test]
    fn test_escape_keyword_not_reserved() {
        // Given: A non-reserved identifier.
        // When: Escaping it.
        let result = escape_keyword("player");

        // Then: It should be unchanged.
        assert_eq!(result, "player");
    }
}
