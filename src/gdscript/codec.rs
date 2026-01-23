use baproto::{Encoding, NativeType, WireFormat};

use super::ast::Stmt;
use super::types::type_name;

/* -------------------------------------------------------------------------- */
/*                            Struct: PrimitiveCodec                          */
/* -------------------------------------------------------------------------- */

/// `PrimitiveCodec` maps a primitive encoding to read/write methods.
#[allow(dead_code)]
#[derive(Debug)]
struct PrimitiveCodec {
    write_method: &'static str,
    read_method: &'static str,
    /// `format_args` generates any additional arguments needed beyond the value/target.
    format_args: Option<fn(&Encoding) -> String>,
}

/* -------------------------------------------------------------------------- */
/*                          Fn: resolve_primitive_codec                       */
/* -------------------------------------------------------------------------- */

/// `resolve_primitive_codec` returns the codec for a primitive encoding.
#[allow(dead_code)]
fn resolve_primitive_codec(encoding: &Encoding) -> Option<PrimitiveCodec> {
    match (&encoding.wire, &encoding.native) {
        // Bool type.
        (WireFormat::Bits { count: 1 }, NativeType::Bool) => Some(PrimitiveCodec {
            write_method: "write_bool",
            read_method: "read_bool",
            format_args: None,
        }),

        // String type.
        (WireFormat::LengthPrefixed { .. }, NativeType::String) => Some(PrimitiveCodec {
            write_method: "write_string",
            read_method: "read_string",
            format_args: None,
        }),

        // Bytes type (write needs size prefix, read needs size arg).
        (WireFormat::LengthPrefixed { .. }, NativeType::Bytes) => None, // Handled specially

        // Int types with specific bit widths.
        (WireFormat::Bits { count }, NativeType::Int { bits, signed }) => {
            // Check for zigzag transform.
            let has_zigzag = encoding
                .transforms
                .iter()
                .any(|t| matches!(t, baproto::Transform::ZigZag));

            if has_zigzag {
                Some(PrimitiveCodec {
                    write_method: "write_zigzag",
                    read_method: "read_zigzag",
                    format_args: Some(|enc: &Encoding| {
                        if let WireFormat::Bits { count } = enc.wire {
                            format!(", {}", count)
                        } else {
                            String::new()
                        }
                    }),
                })
            } else {
                match (bits, signed, count) {
                    (8, false, 8) => Some(PrimitiveCodec {
                        write_method: "write_u8",
                        read_method: "read_u8",
                        format_args: None,
                    }),
                    (8, true, 8) => Some(PrimitiveCodec {
                        write_method: "write_i8",
                        read_method: "read_i8",
                        format_args: None,
                    }),
                    (16, false, 16) => Some(PrimitiveCodec {
                        write_method: "write_u16",
                        read_method: "read_u16",
                        format_args: None,
                    }),
                    (16, true, 16) => Some(PrimitiveCodec {
                        write_method: "write_i16",
                        read_method: "read_i16",
                        format_args: None,
                    }),
                    (32, false, 32) => Some(PrimitiveCodec {
                        write_method: "write_u32",
                        read_method: "read_u32",
                        format_args: None,
                    }),
                    (32, true, 32) => Some(PrimitiveCodec {
                        write_method: "write_i32",
                        read_method: "read_i32",
                        format_args: None,
                    }),
                    (64, _, 64) => Some(PrimitiveCodec {
                        write_method: "write_i64",
                        read_method: "read_i64",
                        format_args: None,
                    }),
                    _ => Some(PrimitiveCodec {
                        write_method: "write_bits",
                        read_method: "read_bits",
                        format_args: Some(|enc: &Encoding| {
                            if let WireFormat::Bits { count } = enc.wire {
                                format!(", {}", count)
                            } else {
                                String::new()
                            }
                        }),
                    }),
                }
            }
        }

        // Varint unsigned.
        (WireFormat::LengthPrefixed { .. }, NativeType::Int { signed: false, .. }) => {
            Some(PrimitiveCodec {
                write_method: "write_varint_unsigned",
                read_method: "read_varint_unsigned",
                format_args: None,
            })
        }

        // Varint signed.
        (WireFormat::LengthPrefixed { .. }, NativeType::Int { signed: true, .. }) => {
            Some(PrimitiveCodec {
                write_method: "write_varint_signed",
                read_method: "read_varint_signed",
                format_args: None,
            })
        }

        // Float types.
        (WireFormat::Bits { count: 32 }, NativeType::Float { bits: 32 }) => Some(PrimitiveCodec {
            write_method: "write_f32",
            read_method: "read_f32",
            format_args: None,
        }),
        (WireFormat::Bits { count: 64 }, NativeType::Float { bits: 64 }) => Some(PrimitiveCodec {
            write_method: "write_f64",
            read_method: "read_f64",
            format_args: None,
        }),

        _ => None,
    }
}

/* -------------------------------------------------------------------------- */
/*                            Fn: gen_encode_stmts                            */
/* -------------------------------------------------------------------------- */

/// `gen_encode_stmts` generates encode statements for a field.
#[allow(dead_code)]
pub fn gen_encode_stmts(field_name: &str, encoding: &Encoding) -> anyhow::Result<Vec<Stmt>> {
    let mut stmts = Vec::new();

    match &encoding.native {
        // Message types - direct method call.
        NativeType::Message { .. } => {
            stmts.push(Stmt::Expr(format!("{}._encode(_writer)", field_name)));
        }

        // Array type.
        NativeType::Array { element } => {
            stmts.push(Stmt::Expr(format!(
                "_writer.write_varint_unsigned({}.size())",
                field_name
            )));
            let inner_stmts = gen_encode_stmts("_item", element)?;
            stmts.push(Stmt::ForIn {
                var_name: "_item".into(),
                iterable: field_name.to_string(),
                body: inner_stmts,
            });
        }

        // Map type.
        NativeType::Map { key, value } => {
            stmts.push(Stmt::Expr(format!(
                "_writer.write_varint_unsigned({}.size())",
                field_name
            )));

            let mut loop_body = Vec::new();
            loop_body.extend(gen_encode_stmts("_key", key)?);
            loop_body.extend(gen_encode_stmts(&format!("{}[_key]", field_name), value)?);

            stmts.push(Stmt::ForIn {
                var_name: "_key".into(),
                iterable: field_name.to_string(),
                body: loop_body,
            });
        }

        // Enum types (represented as int).
        NativeType::Enum { .. } => {
            // Treat enum as its underlying int encoding.
            let int_encoding = Encoding {
                wire: encoding.wire.clone(),
                native: NativeType::Int {
                    bits: 32,
                    signed: true,
                },
                transforms: encoding.transforms.clone(),
                padding_bits: encoding.padding_bits,
            };
            return gen_encode_stmts(field_name, &int_encoding);
        }

        // Bytes type (special handling for size prefix).
        NativeType::Bytes if matches!(encoding.wire, WireFormat::LengthPrefixed { .. }) => {
            stmts.push(Stmt::Expr(format!(
                "_writer.write_varint_unsigned({}.size())",
                field_name
            )));
            stmts.push(Stmt::Expr(format!("_writer.write_bytes({})", field_name)));
        }

        // Primitives.
        _ => {
            if let Some(codec) = resolve_primitive_codec(encoding) {
                let args = if let Some(format_fn) = codec.format_args {
                    format_fn(encoding)
                } else {
                    String::new()
                };
                stmts.push(Stmt::Expr(format!(
                    "_writer.{}({}{}))",
                    codec.write_method, field_name, args
                )));
            } else {
                anyhow::bail!(
                    "Unsupported encoding combination: wire={:?}, native={:?}",
                    encoding.wire,
                    encoding.native
                );
            }
        }
    }

    Ok(stmts)
}

/* -------------------------------------------------------------------------- */
/*                            Fn: gen_decode_stmts                            */
/* -------------------------------------------------------------------------- */

/// `gen_decode_stmts` generates decode statements for a field.
#[allow(dead_code)]
pub fn gen_decode_stmts(field_name: &str, encoding: &Encoding) -> anyhow::Result<Vec<Stmt>> {
    let mut stmts = Vec::new();

    match &encoding.native {
        // Message types.
        NativeType::Message { .. } => {
            let type_str = type_name(&encoding.native);
            stmts.push(Stmt::Assign {
                target: field_name.to_string(),
                value: format!("{}.new()", type_str),
            });
            stmts.push(Stmt::Expr(format!("{}._decode(_reader)", field_name)));
        }

        // Array type.
        NativeType::Array { element } => {
            stmts.push(Stmt::Assign {
                target: field_name.to_string(),
                value: "[]".to_string(),
            });

            let mut loop_body = Vec::new();
            if matches!(element.native, NativeType::Message { .. }) {
                let type_str = type_name(&element.native);
                loop_body.push(Stmt::Assign {
                    target: "_item".to_string(),
                    value: format!("{}.new()", type_str),
                });
                loop_body.push(Stmt::Expr("_item._decode(_reader)".to_string()));
                loop_body.push(Stmt::Expr(format!("{}.append(_item)", field_name)));
            } else {
                let item_expr = gen_decode_expr(element)?;
                loop_body.push(Stmt::Expr(format!("{}.append({})", field_name, item_expr)));
            }

            stmts.push(Stmt::ForIn {
                var_name: "_i".into(),
                iterable: "range(_reader.read_varint_unsigned())".to_string(),
                body: loop_body,
            });
        }

        // Map type.
        NativeType::Map { key, value } => {
            stmts.push(Stmt::Assign {
                target: field_name.to_string(),
                value: "{}".to_string(),
            });

            let mut loop_body = Vec::new();
            let key_expr = gen_decode_expr(key)?;
            loop_body.push(Stmt::Assign {
                target: "_key".to_string(),
                value: key_expr,
            });

            if matches!(value.native, NativeType::Message { .. }) {
                let type_str = type_name(&value.native);
                loop_body.push(Stmt::Assign {
                    target: "_val".to_string(),
                    value: format!("{}.new()", type_str),
                });
                loop_body.push(Stmt::Expr("_val._decode(_reader)".to_string()));
                loop_body.push(Stmt::Assign {
                    target: format!("{}[_key]", field_name),
                    value: "_val".to_string(),
                });
            } else {
                let val_expr = gen_decode_expr(value)?;
                loop_body.push(Stmt::Assign {
                    target: format!("{}[_key]", field_name),
                    value: val_expr,
                });
            }

            stmts.push(Stmt::ForIn {
                var_name: "_i".into(),
                iterable: "range(_reader.read_varint_unsigned())".to_string(),
                body: loop_body,
            });
        }

        // Enum types (represented as int).
        NativeType::Enum { .. } => {
            // Treat enum as its underlying int encoding.
            let int_encoding = Encoding {
                wire: encoding.wire.clone(),
                native: NativeType::Int {
                    bits: 32,
                    signed: true,
                },
                transforms: encoding.transforms.clone(),
                padding_bits: encoding.padding_bits,
            };
            return gen_decode_stmts(field_name, &int_encoding);
        }

        // Primitives.
        _ => {
            let decode_expr = gen_decode_expr(encoding)?;
            stmts.push(Stmt::Assign {
                target: field_name.to_string(),
                value: decode_expr,
            });
        }
    }

    Ok(stmts)
}

/* -------------------------------------------------------------------------- */
/*                            Fn: gen_decode_expr                             */
/* -------------------------------------------------------------------------- */

/// `gen_decode_expr` generates a decode expression for a value.
#[allow(dead_code)]
fn gen_decode_expr(encoding: &Encoding) -> anyhow::Result<String> {
    let expr = match &encoding.native {
        // Bytes type (special handling for size argument).
        NativeType::Bytes if matches!(encoding.wire, WireFormat::LengthPrefixed { .. }) => {
            "_reader.read_bytes(_reader.read_varint_unsigned())".to_string()
        }

        // Enum types.
        NativeType::Enum { .. } => {
            let int_encoding = Encoding {
                wire: encoding.wire.clone(),
                native: NativeType::Int {
                    bits: 32,
                    signed: true,
                },
                transforms: encoding.transforms.clone(),
                padding_bits: encoding.padding_bits,
            };
            gen_decode_expr(&int_encoding)?
        }

        // Primitives.
        _ => {
            if let Some(codec) = resolve_primitive_codec(encoding) {
                let args = if let Some(format_fn) = codec.format_args {
                    format_fn(encoding)
                } else {
                    String::new()
                };
                format!(
                    "_reader.{}({})",
                    codec.read_method,
                    args.trim_start_matches(", ")
                )
            } else {
                anyhow::bail!(
                    "Unsupported encoding combination: wire={:?}, native={:?}",
                    encoding.wire,
                    encoding.native
                );
            }
        }
    };

    Ok(expr)
}

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use baproto::{Descriptor, DescriptorBuilder, PackageName};

    /* ------------------- Tests: resolve_primitive_codec ------------------- */

    #[test]
    fn test_resolve_codec_bool() {
        // Given: A bool encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 1 },
            native: NativeType::Bool,
            transforms: vec![],
            padding_bits: None,
        };

        // When: Resolving codec.
        let codec = resolve_primitive_codec(&encoding).unwrap();

        // Then: Should return bool codec.
        assert_eq!(codec.write_method, "write_bool");
        assert_eq!(codec.read_method, "read_bool");
    }

    #[test]
    fn test_resolve_codec_int_variants() {
        // Given: Various int encodings.
        let test_cases = vec![
            (8, false, 8, "write_u8", "read_u8"),
            (16, true, 16, "write_i16", "read_i16"),
            (32, false, 32, "write_u32", "read_u32"),
            (64, true, 64, "write_i64", "read_i64"),
        ];

        for (bits, signed, count, expected_write, expected_read) in test_cases {
            // When: Resolving codec.
            let encoding = Encoding {
                wire: WireFormat::Bits { count },
                native: NativeType::Int { bits, signed },
                transforms: vec![],
                padding_bits: None,
            };
            let codec = resolve_primitive_codec(&encoding).unwrap();

            // Then: Should return correct methods.
            assert_eq!(codec.write_method, expected_write);
            assert_eq!(codec.read_method, expected_read);
        }
    }

    #[test]
    fn test_resolve_codec_zigzag() {
        // Given: An int encoding with zigzag transform.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 32 },
            native: NativeType::Int {
                bits: 32,
                signed: true,
            },
            transforms: vec![baproto::Transform::ZigZag],
            padding_bits: None,
        };

        // When: Resolving codec.
        let codec = resolve_primitive_codec(&encoding).unwrap();

        // Then: Should return zigzag codec.
        assert_eq!(codec.write_method, "write_zigzag");
        assert_eq!(codec.read_method, "read_zigzag");
    }

    /* ----------------------- Tests: gen_encode_stmts ---------------------- */

    #[test]
    fn test_encode_stmts_primitive() {
        // Given: A bool encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 1 },
            native: NativeType::Bool,
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("active", &encoding).unwrap();

        // Then: Should generate write_bool call.
        assert_eq!(stmts.len(), 1);
        matches!(stmts[0], Stmt::Expr(_));
        if let Stmt::Expr(expr) = &stmts[0] {
            assert!(expr.contains("write_bool"));
            assert!(expr.contains("active"));
        }
    }

    #[test]
    fn test_encode_stmts_array() {
        // Given: An array of ints.
        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 16 },
            native: NativeType::Array {
                element: Box::new(Encoding {
                    wire: WireFormat::Bits { count: 32 },
                    native: NativeType::Int {
                        bits: 32,
                        signed: true,
                    },
                    transforms: vec![],
                    padding_bits: None,
                }),
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("items", &encoding).unwrap();

        // Then: Should generate size write + for loop.
        assert!(stmts.len() >= 2);
        matches!(stmts[0], Stmt::Expr(_));
        matches!(stmts[1], Stmt::ForIn { .. });
    }

    #[test]
    fn test_encode_stmts_message() {
        // Given: A message encoding.
        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 16 },
            native: NativeType::Message {
                descriptor: Descriptor {
                    package: PackageName::try_from(vec!["test"]).unwrap(),
                    path: vec!["Player".to_string()],
                },
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("player", &encoding).unwrap();

        // Then: Should generate _encode call.
        assert_eq!(stmts.len(), 1);
        matches!(stmts[0], Stmt::Expr(_));
        if let Stmt::Expr(expr) = &stmts[0] {
            assert!(expr.contains("player._encode(_writer)"));
        }
    }

    /* ----------------------- Tests: gen_decode_stmts ---------------------- */

    #[test]
    fn test_decode_stmts_primitive() {
        // Given: A bool encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 1 },
            native: NativeType::Bool,
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("active", &encoding).unwrap();

        // Then: Should generate read_bool assignment.
        assert_eq!(stmts.len(), 1);
        matches!(stmts[0], Stmt::Assign { .. });
        if let Stmt::Assign { target, value } = &stmts[0] {
            assert_eq!(target, "active");
            assert!(value.contains("read_bool"));
        }
    }

    #[test]
    fn test_decode_stmts_array() {
        // Given: An array of ints.
        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 16 },
            native: NativeType::Array {
                element: Box::new(Encoding {
                    wire: WireFormat::Bits { count: 32 },
                    native: NativeType::Int {
                        bits: 32,
                        signed: true,
                    },
                    transforms: vec![],
                    padding_bits: None,
                }),
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("items", &encoding).unwrap();

        // Then: Should generate empty array assignment + for loop.
        assert!(stmts.len() >= 2);
        matches!(stmts[0], Stmt::Assign { .. });
        matches!(stmts[1], Stmt::ForIn { .. });
    }

    #[test]
    fn test_decode_stmts_message() {
        // Given: A message encoding.
        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 16 },
            native: NativeType::Message {
                descriptor: DescriptorBuilder::default()
                    .package(PackageName::try_from(vec!["test"]).unwrap())
                    .path(vec!["Player".to_string()])
                    .build()
                    .unwrap(),
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("player", &encoding).unwrap();

        // Then: Should generate new + _decode call.
        assert_eq!(stmts.len(), 2);
        matches!(stmts[0], Stmt::Assign { .. });
        matches!(stmts[1], Stmt::Expr(_));
    }
}
