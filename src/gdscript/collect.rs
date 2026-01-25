use baproto::{Enum, Message, Package};

/* -------------------------------------------------------------------------- */
/*                               Enum: TypeKind                               */
/* -------------------------------------------------------------------------- */

/// `TypeKind` represents the kind of type entry (message or enum).
#[derive(Clone, Debug)]
pub enum TypeKind {
    Message(Message),
    Enum(Enum),
}

/* -------------------------------------------------------------------------- */
/*                              Struct: TypeEntry                             */
/* -------------------------------------------------------------------------- */

/// `TypeEntry` represents a single type (message or enum) to be generated as
/// a separate GDScript file.
#[derive(Clone, Debug)]
pub struct TypeEntry {
    /// The kind of type (message or enum).
    pub kind: TypeKind,
    /// The file stem (e.g., "Player_Stats" for nested type Stats in Player).
    pub file_stem: String,
    /// The simple name (e.g., "Stats" for const reference from parent).
    #[allow(dead_code)]
    pub simple_name: String,
    /// File stems of nested types (for generating const preloads).
    pub nested: Vec<String>,
}

/* -------------------------------------------------------------------------- */
/*                          Fn: collect_package_types                         */
/* -------------------------------------------------------------------------- */

/// `collect_package_types` collects all types from a package into a flat list
/// of [`TypeEntry`] values, flattening nested types with underscore prefixes.
pub fn collect_package_types(pkg: &Package) -> Vec<TypeEntry> {
    let mut entries = Vec::new();

    // Collect top-level enums.
    for enm in &pkg.enums {
        let Some(name) = enm.name() else { continue };
        entries.push(TypeEntry {
            kind: TypeKind::Enum(enm.clone()),
            file_stem: name.to_string(),
            simple_name: name.to_string(),
            nested: Vec::new(),
        });
    }

    // Collect top-level messages (recursively collects nested types).
    for msg in &pkg.messages {
        collect_message(&mut entries, msg, &[]);
    }

    entries
}

/* --------------------------- Fn: collect_message -------------------------- */

/// `collect_message` recursively collects a message and its nested types.
fn collect_message(entries: &mut Vec<TypeEntry>, msg: &Message, ancestors: &[&str]) {
    let Some(name) = msg.name() else { return };

    // Build the file stem by joining ancestors with underscores.
    let file_stem = if ancestors.is_empty() {
        name.to_string()
    } else {
        format!("{}_{}", ancestors.join("_"), name)
    };

    // Collect nested enums first.
    let mut nested = Vec::new();
    for enm in &msg.enums {
        let Some(enum_name) = enm.name() else {
            continue;
        };
        let nested_stem = format!("{}_{}", file_stem, enum_name);
        nested.push(nested_stem.clone());

        entries.push(TypeEntry {
            kind: TypeKind::Enum(enm.clone()),
            file_stem: nested_stem,
            simple_name: enum_name.to_string(),
            nested: Vec::new(),
        });
    }

    // Collect nested messages recursively.
    let mut child_ancestors: Vec<&str> = ancestors.to_vec();
    child_ancestors.push(name);

    for nested_msg in &msg.messages {
        let Some(nested_name) = nested_msg.name() else {
            continue;
        };
        let nested_stem = format!("{}_{}", file_stem, nested_name);
        nested.push(nested_stem);

        collect_message(entries, nested_msg, &child_ancestors);
    }

    // Add the message entry itself.
    entries.push(TypeEntry {
        kind: TypeKind::Message(msg.clone()),
        file_stem,
        simple_name: name.to_string(),
        nested,
    });
}

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /* -------------------- Tests: collect_package_types ------------------- */

    #[test]
    fn test_collect_empty_package() {
        // Given: An empty package.
        let pkg = create_test_package(vec![], vec![]);

        // When: Collecting types.
        let entries = collect_package_types(&pkg);

        // Then: No entries should be collected.
        assert!(entries.is_empty());
    }

    #[test]
    fn test_collect_top_level_enum() {
        // Given: A package with a single top-level enum.
        let pkg = create_test_package(vec![], vec![create_test_enum("State")]);

        // When: Collecting types.
        let entries = collect_package_types(&pkg);

        // Then: One entry should be collected with correct file stem.
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_stem, "State");
        assert_eq!(entries[0].simple_name, "State");
    }

    #[test]
    fn test_collect_top_level_message() {
        // Given: A package with a single top-level message.
        let pkg = create_test_package(vec![create_test_message("Player", vec![], vec![])], vec![]);

        // When: Collecting types.
        let entries = collect_package_types(&pkg);

        // Then: One entry should be collected with correct file stem.
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_stem, "Player");
        assert_eq!(entries[0].simple_name, "Player");
    }

    #[test]
    fn test_collect_message_with_nested_enum() {
        // Given: A package with a message containing a nested enum.
        let nested_enum = create_test_enum("State");
        let msg = create_test_message("Player", vec![], vec![nested_enum]);
        let pkg = create_test_package(vec![msg], vec![]);

        // When: Collecting types.
        let entries = collect_package_types(&pkg);

        // Then: Two entries should be collected with correct file stems.
        assert_eq!(entries.len(), 2);

        // Find entries by file stem.
        let enum_entry = entries.iter().find(|e| e.file_stem == "Player_State");
        let msg_entry = entries.iter().find(|e| e.file_stem == "Player");

        assert!(enum_entry.is_some());
        assert!(msg_entry.is_some());

        let msg_entry = msg_entry.unwrap();
        assert!(msg_entry.nested.contains(&"Player_State".to_string()));
    }

    #[test]
    fn test_collect_message_with_nested_message() {
        // Given: A package with a message containing a nested message.
        let nested_msg = create_test_message("Stats", vec![], vec![]);
        let msg = create_test_message("Player", vec![nested_msg], vec![]);
        let pkg = create_test_package(vec![msg], vec![]);

        // When: Collecting types.
        let entries = collect_package_types(&pkg);

        // Then: Two entries should be collected with correct file stems.
        assert_eq!(entries.len(), 2);

        let stats_entry = entries.iter().find(|e| e.file_stem == "Player_Stats");
        let player_entry = entries.iter().find(|e| e.file_stem == "Player");

        assert!(stats_entry.is_some());
        assert!(player_entry.is_some());

        let player_entry = player_entry.unwrap();
        assert!(player_entry.nested.contains(&"Player_Stats".to_string()));
    }

    #[test]
    fn test_collect_deeply_nested_types() {
        // Given: A deeply nested structure: Outer > Middle > Inner.
        let inner = create_test_message("Inner", vec![], vec![]);
        let middle = create_test_message("Middle", vec![inner], vec![]);
        let outer = create_test_message("Outer", vec![middle], vec![]);
        let pkg = create_test_package(vec![outer], vec![]);

        // When: Collecting types.
        let entries = collect_package_types(&pkg);

        // Then: Three entries should be collected with correct file stems.
        assert_eq!(entries.len(), 3);

        assert!(entries.iter().any(|e| e.file_stem == "Outer"));
        assert!(entries.iter().any(|e| e.file_stem == "Outer_Middle"));
        assert!(entries.iter().any(|e| e.file_stem == "Outer_Middle_Inner"));
    }

    /* ----------------------- Fn: create_test_package ---------------------- */

    pub(crate) fn create_test_package(messages: Vec<Message>, enums: Vec<Enum>) -> Package {
        Package {
            name: baproto::PackageName::try_from(vec!["test"]).unwrap(),
            messages,
            enums,
        }
    }

    /* ----------------------- Fn: create_test_message ---------------------- */

    pub(crate) fn create_test_message(
        name: &str,
        nested_messages: Vec<Message>,
        nested_enums: Vec<Enum>,
    ) -> Message {
        Message {
            descriptor: baproto::DescriptorBuilder::default()
                .package(baproto::PackageName::try_from(vec!["test"]).unwrap())
                .path(vec![name.to_string()])
                .build()
                .unwrap(),
            doc: None,
            fields: vec![],
            messages: nested_messages,
            enums: nested_enums,
        }
    }

    /* ------------------------ Fn: create_test_enum ------------------------ */

    pub(crate) fn create_test_enum(name: &str) -> Enum {
        Enum {
            descriptor: baproto::DescriptorBuilder::default()
                .package(baproto::PackageName::try_from(vec!["test"]).unwrap())
                .path(vec![name.to_string()])
                .build()
                .unwrap(),
            discriminant: baproto::Encoding {
                wire: baproto::WireFormat::Bits { count: 8 },
                native: baproto::NativeType::Int {
                    bits: 8,
                    signed: false,
                },
                transforms: vec![],
                padding_bits: None,
            },
            doc: None,
            variants: vec![],
        }
    }
}
