use baproto::{CodeWriter, CodeWriterBuilder, Generator, GeneratorError, GeneratorOutput, Schema};

use crate::gdscript::collect::{TypeKind, collect_package_types};
use crate::gdscript::types::pkg_to_path;

/* -------------------------------- Mod: Collect ------------------------------ */

pub mod collect;

/* --------------------------------- Mod: AST --------------------------------- */

mod ast;

/* -------------------------------- Mod: Codec -------------------------------- */

mod codec;

/* -------------------------------- Mod: Types -------------------------------- */

mod types;

/* ------------------------------- Mod: Message ------------------------------- */

mod message;

/* ----------------------------- Mod: Enumeration ----------------------------- */

mod enumeration;

/* ------------------------------ Mod: Namespace ------------------------------ */

mod namespace;

/* -------------------------------------------------------------------------- */
/*                              Struct: GDScript                              */
/* -------------------------------------------------------------------------- */

/// `GDScript` is a code generator that produces GDScript bindings from
/// Build-A-Proto schemas.
///
/// It generates one file per type (message or enum), organized into package
/// subdirectories with namespace `mod.gd` files.
#[derive(Clone, Debug)]
pub struct GDScript;

/* ----------------------------- Impl: Default -------------------------------- */

impl GDScript {
    /// `writer` creates a new [`CodeWriter`] suited for GDScript files.
    fn writer() -> CodeWriter {
        CodeWriterBuilder::default()
            .comment_token("##".to_owned())
            .indent_token("\t".to_owned())
            .newline_token("\n".to_owned())
            .build()
            .expect("failed to build CodeWriter")
    }
}

/* ----------------------------- Impl: Generator ------------------------------ */

impl Generator for GDScript {
    fn name(&self) -> &str {
        "gdscript"
    }

    fn generate(&self, schema: &Schema) -> Result<GeneratorOutput, GeneratorError> {
        use std::collections::BTreeSet;

        let mut output = GeneratorOutput::default();

        // Step 1: Generate type files for each package.
        for pkg in &schema.packages {
            let entries = collect_package_types(pkg);
            if entries.is_empty() {
                continue;
            }

            let pkg_path = pkg_to_path(&pkg.name);

            for entry in &entries {
                let path = format!("{}/{}.gd", pkg_path, entry.file_stem.to_lowercase());
                let mut cw = GDScript::writer();

                let content = match &entry.kind {
                    TypeKind::Message(msg) => {
                        message::generate_message(&mut cw, msg, entry, &pkg.name)
                    }
                    TypeKind::Enum(enm) => {
                        enumeration::generate_enum(&mut cw, enm, entry, &pkg.name)
                    }
                }
                .map_err(|e| GeneratorError::Generation(e.to_string()))?;

                output.add(path, content);
            }
        }

        // Step 2: Collect all package path hierarchies (including intermediate paths).
        let mut all_package_paths: BTreeSet<Vec<String>> = BTreeSet::new();
        for pkg in &schema.packages {
            let segments: Vec<String> = pkg.name.iter().map(|s| s.to_string()).collect();

            // Add all prefixes: foo.bar.baz -> [foo], [foo, bar], [foo, bar, baz].
            for i in 1..=segments.len() {
                all_package_paths.insert(segments[..i].to_vec());
            }
        }

        // Step 3: Generate mod.gd for each package (including intermediates).
        for pkg_segments in &all_package_paths {
            let pkg_path = pkg_segments.join("/");
            let pkg_name = pkg_segments.join(".");

            // Find direct children (subpackages).
            let mut subpackages: Vec<String> = all_package_paths
                .iter()
                .filter(|p| {
                    p.len() == pkg_segments.len() + 1 && p[..pkg_segments.len()] == pkg_segments[..]
                })
                .map(|p| p.last().unwrap().clone())
                .collect();
            subpackages.sort();

            // Find types in this exact package.
            let entries = schema
                .packages
                .iter()
                .find(|p| {
                    let segments: Vec<String> = p.name.iter().map(|s| s.to_string()).collect();
                    &segments == pkg_segments
                })
                .map(collect_package_types)
                .unwrap_or_default();

            // Generate mod.gd with both types and subpackages.
            let mut cw = GDScript::writer();
            let content =
                namespace::generate_namespace(&mut cw, &pkg_name, None, &entries, &subpackages)
                    .map_err(|e| GeneratorError::Generation(e.to_string()))?;

            output.add(format!("{}/mod.gd", pkg_path), content);
        }

        // Step 4: Generate root mod.gd.
        if !all_package_paths.is_empty() {
            let mut root_subpackages: Vec<String> = all_package_paths
                .iter()
                .filter(|p| p.len() == 1)
                .map(|p| p[0].clone())
                .collect();
            root_subpackages.sort();

            let mut cw = GDScript::writer();
            let content =
                namespace::generate_namespace(&mut cw, "", Some("BAProto"), &[], &root_subpackages)
                    .map_err(|e| GeneratorError::Generation(e.to_string()))?;

            output.add("mod.gd".to_string(), content);
        }

        Ok(output)
    }
}

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
pub(crate) mod tests {
    use std::path::Path;

    use super::*;
    use baproto::*;

    /* --------------------------- Tests: generate -------------------------- */

    #[test]
    fn test_generate_empty_schema() {
        // Given: An empty schema.
        let schema = Schema { packages: vec![] };

        // When: Generating code.
        let output = GDScript.generate(&schema).unwrap();

        // Then: No files should be generated.
        assert!(output.files.is_empty());
    }

    #[test]
    fn test_generate_empty_package() {
        // Given: A schema with an empty package.
        let schema = Schema {
            packages: vec![Package {
                name: PackageName::try_from(vec!["test"]).unwrap(),
                messages: vec![],
                enums: vec![],
            }],
        };

        // When: Generating code.
        let output = GDScript.generate(&schema).unwrap();

        // Then: Should generate namespace files (test/mod.gd + root mod.gd).
        assert_eq!(output.files.len(), 2);
        assert!(output.files.contains_key(Path::new("test/mod.gd")));
        assert!(output.files.contains_key(Path::new("mod.gd")));
    }

    #[test]
    fn test_generate_single_message() {
        // Given: A schema with a single message.
        let pkg = PackageName::try_from(vec!["game"]).unwrap();
        let schema = Schema {
            packages: vec![Package {
                name: pkg.clone(),
                messages: vec![Message {
                    descriptor: DescriptorBuilder::default()
                        .package(pkg)
                        .path(vec!["Player".to_string()])
                        .build()
                        .unwrap(),
                    doc: None,
                    fields: vec![],
                    messages: vec![],
                    enums: vec![],
                }],
                enums: vec![],
            }],
        };

        // When: Generating code.
        let output = GDScript.generate(&schema).unwrap();

        // Then: Three files should be generated (message + game/mod.gd + root mod.gd).
        assert_eq!(output.files.len(), 3);
        assert!(output.files.contains_key(Path::new("game/player.gd")));
        assert!(output.files.contains_key(Path::new("game/mod.gd")));
        assert!(output.files.contains_key(Path::new("mod.gd")));
    }

    #[test]
    fn test_generate_message_with_fields() {
        // Given: A schema with a message containing fields.
        let pkg = PackageName::try_from(vec!["game"]).unwrap();
        let schema = Schema {
            packages: vec![Package {
                name: pkg.clone(),
                messages: vec![Message {
                    descriptor: DescriptorBuilder::default()
                        .package(pkg)
                        .path(vec!["Player".to_string()])
                        .build()
                        .unwrap(),
                    doc: Some("A player entity.".to_string()),
                    fields: vec![
                        Field {
                            name: "health".to_string(),
                            index: 0,
                            encoding: Encoding {
                                wire: WireFormat::Bits { count: 32 },
                                native: NativeType::Int {
                                    bits: 32,
                                    signed: true,
                                },
                                transforms: vec![],
                                padding_bits: None,
                            },
                            doc: None,
                        },
                        Field {
                            name: "name".to_string(),
                            index: 1,
                            encoding: Encoding {
                                wire: WireFormat::LengthPrefixed { prefix_bits: 16 },
                                native: NativeType::String,
                                transforms: vec![],
                                padding_bits: None,
                            },
                            doc: None,
                        },
                    ],
                    messages: vec![],
                    enums: vec![],
                }],
                enums: vec![],
            }],
        };

        // When: Generating code.
        let output = GDScript.generate(&schema).unwrap();

        // Then: The message file should contain the fields.
        let content = output.files.get(Path::new("game/player.gd")).unwrap();
        assert!(content.contains("var health: int = 0"));
        assert!(content.contains("var name: String = \"\""));
        assert!(content.contains("## A player entity."));
    }

    #[test]
    fn test_generate_single_enum() {
        // Given: A schema with a single enum.
        let pkg = PackageName::try_from(vec!["game"]).unwrap();
        let schema = Schema {
            packages: vec![Package {
                name: pkg.clone(),
                messages: vec![],
                enums: vec![Enum {
                    descriptor: DescriptorBuilder::default()
                        .package(pkg)
                        .path(vec!["State".to_string()])
                        .build()
                        .unwrap(),
                    discriminant: Encoding {
                        wire: WireFormat::Bits { count: 8 },
                        native: NativeType::Int {
                            bits: 8,
                            signed: false,
                        },
                        transforms: vec![],
                        padding_bits: None,
                    },
                    doc: None,
                    variants: vec![
                        Variant::Unit {
                            name: "IDLE".to_string(),
                            index: 0,
                            doc: None,
                        },
                        Variant::Unit {
                            name: "MOVING".to_string(),
                            index: 1,
                            doc: None,
                        },
                    ],
                }],
            }],
        };

        // When: Generating code.
        let output = GDScript.generate(&schema).unwrap();

        // Then: Three files should be generated (enum + game/mod.gd + root mod.gd).
        assert_eq!(output.files.len(), 3);
        assert!(output.files.contains_key(Path::new("game/state.gd")));
        assert!(output.files.contains_key(Path::new("game/mod.gd")));
        assert!(output.files.contains_key(Path::new("mod.gd")));

        let content = output.files.get(Path::new("game/state.gd")).unwrap();
        assert!(content.contains("const IDLE: int = 0"));
        assert!(content.contains("const MOVING: int = 1"));
    }

    #[test]
    fn test_generate_nested_message() {
        // Given: A schema with a message containing a nested message.
        let pkg = PackageName::try_from(vec!["game"]).unwrap();
        let schema = Schema {
            packages: vec![Package {
                name: pkg.clone(),
                messages: vec![Message {
                    descriptor: DescriptorBuilder::default()
                        .package(pkg.clone())
                        .path(vec!["Player".to_string()])
                        .build()
                        .unwrap(),
                    doc: None,
                    fields: vec![],
                    messages: vec![Message {
                        descriptor: DescriptorBuilder::default()
                            .package(pkg)
                            .path(vec!["Player".to_string(), "Stats".to_string()])
                            .build()
                            .unwrap(),
                        doc: None,
                        fields: vec![Field {
                            name: "level".to_string(),
                            index: 0,
                            encoding: Encoding {
                                wire: WireFormat::Bits { count: 8 },
                                native: NativeType::Int {
                                    bits: 8,
                                    signed: false,
                                },
                                transforms: vec![],
                                padding_bits: None,
                            },
                            doc: None,
                        }],
                        messages: vec![],
                        enums: vec![],
                    }],
                    enums: vec![],
                }],
                enums: vec![],
            }],
        };

        // When: Generating code.
        let output = GDScript.generate(&schema).unwrap();

        // Then: Four files should be generated (2 types + game/mod.gd + root mod.gd).
        assert_eq!(output.files.len(), 4);
        assert!(output.files.contains_key(Path::new("game/player.gd")));
        assert!(output.files.contains_key(Path::new("game/player_stats.gd")));
        assert!(output.files.contains_key(Path::new("game/mod.gd")));
        assert!(output.files.contains_key(Path::new("mod.gd")));

        // The parent should reference the nested type.
        let player = output.files.get(Path::new("game/player.gd")).unwrap();
        assert!(player.contains("const Stats := preload(\"./player_stats.gd\")"));

        // The nested type should have the field.
        let stats = output.files.get(Path::new("game/player_stats.gd")).unwrap();
        assert!(stats.contains("var level: int = 0"));

        // The mod.gd should reference both.
        let mod_file = output.files.get(Path::new("game/mod.gd")).unwrap();
        assert!(mod_file.contains("const Player := preload(\"./player.gd\")"));
        assert!(mod_file.contains("const Player_Stats := preload(\"./player_stats.gd\")"));
    }

    #[test]
    fn test_generate_nested_enum() {
        // Given: A schema with a message containing a nested enum.
        let pkg = PackageName::try_from(vec!["game"]).unwrap();
        let schema = Schema {
            packages: vec![Package {
                name: pkg.clone(),
                messages: vec![Message {
                    descriptor: DescriptorBuilder::default()
                        .package(pkg.clone())
                        .path(vec!["Player".to_string()])
                        .build()
                        .unwrap(),
                    doc: None,
                    fields: vec![],
                    messages: vec![],
                    enums: vec![Enum {
                        descriptor: DescriptorBuilder::default()
                            .package(pkg)
                            .path(vec!["Player".to_string(), "State".to_string()])
                            .build()
                            .unwrap(),
                        discriminant: Encoding {
                            wire: WireFormat::Bits { count: 8 },
                            native: NativeType::Int {
                                bits: 8,
                                signed: false,
                            },
                            transforms: vec![],
                            padding_bits: None,
                        },
                        doc: None,
                        variants: vec![Variant::Unit {
                            name: "ACTIVE".to_string(),
                            index: 0,
                            doc: None,
                        }],
                    }],
                }],
                enums: vec![],
            }],
        };

        // When: Generating code.
        let output = GDScript.generate(&schema).unwrap();

        // Then: Four files should be generated (2 types + game/mod.gd + root mod.gd).
        assert_eq!(output.files.len(), 4);
        assert!(output.files.contains_key(Path::new("game/player.gd")));
        assert!(output.files.contains_key(Path::new("game/player_state.gd")));
        assert!(output.files.contains_key(Path::new("game/mod.gd")));
        assert!(output.files.contains_key(Path::new("mod.gd")));

        // The parent should reference the nested enum.
        let player = output.files.get(Path::new("game/player.gd")).unwrap();
        assert!(player.contains("const State := preload(\"./player_state.gd\")"));

        // The nested enum should have the constant.
        let state = output.files.get(Path::new("game/player_state.gd")).unwrap();
        assert!(state.contains("const ACTIVE: int = 0"));
    }

    #[test]
    fn test_generate_multiple_packages() {
        // Given: A schema with multiple packages.
        let pkg1 = PackageName::try_from(vec!["game", "player"]).unwrap();
        let pkg2 = PackageName::try_from(vec!["game", "enemy"]).unwrap();
        let schema = Schema {
            packages: vec![
                Package {
                    name: pkg1.clone(),
                    messages: vec![Message {
                        descriptor: DescriptorBuilder::default()
                            .package(pkg1)
                            .path(vec!["Player".to_string()])
                            .build()
                            .unwrap(),
                        doc: None,
                        fields: vec![],
                        messages: vec![],
                        enums: vec![],
                    }],
                    enums: vec![],
                },
                Package {
                    name: pkg2.clone(),
                    messages: vec![Message {
                        descriptor: DescriptorBuilder::default()
                            .package(pkg2)
                            .path(vec!["Enemy".to_string()])
                            .build()
                            .unwrap(),
                        doc: None,
                        fields: vec![],
                        messages: vec![],
                        enums: vec![],
                    }],
                    enums: vec![],
                },
            ],
        };

        // When: Generating code.
        let output = GDScript.generate(&schema).unwrap();

        // Then: Six files should be generated.
        // (2 messages + 2 package mod.gd + 1 intermediate game/mod.gd + 1 root mod.gd).
        assert_eq!(output.files.len(), 6);
        assert!(
            output
                .files
                .contains_key(Path::new("game/player/player.gd"))
        );
        assert!(output.files.contains_key(Path::new("game/player/mod.gd")));
        assert!(output.files.contains_key(Path::new("game/enemy/enemy.gd")));
        assert!(output.files.contains_key(Path::new("game/enemy/mod.gd")));
        assert!(output.files.contains_key(Path::new("game/mod.gd")));
        assert!(output.files.contains_key(Path::new("mod.gd")));

        // The game/mod.gd should reference both subpackages.
        let game_mod = output.files.get(Path::new("game/mod.gd")).unwrap();
        assert!(game_mod.contains("const player := preload(\"./player/mod.gd\")"));
        assert!(game_mod.contains("const enemy := preload(\"./enemy/mod.gd\")"));

        // The root mod.gd should reference the game package.
        let root_mod = output.files.get(Path::new("mod.gd")).unwrap();
        assert!(root_mod.contains("const game := preload(\"./game/mod.gd\")"));
    }
}
