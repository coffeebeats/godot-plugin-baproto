use baproto::{Encoding, NativeType, Transform, WireFormat};

use crate::gdscript::ast::Expr;

/* -------------------------------------------------------------------------- */
/*                             Struct: CodecMethod                            */
/* -------------------------------------------------------------------------- */

/// `CodecMethod` contains the method name and extra arguments for codec operations.
#[derive(Debug, PartialEq)]
pub struct CodecMethod {
    /// `method` is the name of the reader/writer method to call.
    pub method: String,
    /// `extra_args` contains additional arguments beyond the value being encoded/decoded.
    pub extra_args: Vec<Expr>,
}

/* -------------------------------------------------------------------------- */
/*                            Fn: get_write_method                            */
/* -------------------------------------------------------------------------- */

/// `get_write_method` returns the writer method name and extra arguments for an encoding.
pub fn get_write_method(encoding: &Encoding) -> anyhow::Result<CodecMethod> {
    let has_zigzag = encoding
        .transforms
        .iter()
        .any(|t| matches!(t, Transform::ZigZag));

    match (&encoding.native, &encoding.wire) {
        // Bool
        (NativeType::Bool, _) => Ok(CodecMethod {
            method: "write_bool".to_string(),
            extra_args: vec![],
        }),

        // Integers with zigzag encoding (must come before fixed-width)
        (NativeType::Int { .. }, WireFormat::Bits { count }) if has_zigzag => Ok(CodecMethod {
            method: "write_zigzag".to_string(),
            extra_args: vec![Expr::Literal((*count as i64).into())],
        }),

        // Integers with fixed-width encoding
        (
            NativeType::Int {
                bits: 8,
                signed: true,
            },
            WireFormat::Bits { count: 8 },
        ) => Ok(CodecMethod {
            method: "write_i8".to_string(),
            extra_args: vec![],
        }),
        (
            NativeType::Int {
                bits: 8,
                signed: false,
            },
            WireFormat::Bits { count: 8 },
        ) => Ok(CodecMethod {
            method: "write_u8".to_string(),
            extra_args: vec![],
        }),
        (
            NativeType::Int {
                bits: 16,
                signed: true,
            },
            WireFormat::Bits { count: 16 },
        ) => Ok(CodecMethod {
            method: "write_i16".to_string(),
            extra_args: vec![],
        }),
        (
            NativeType::Int {
                bits: 16,
                signed: false,
            },
            WireFormat::Bits { count: 16 },
        ) => Ok(CodecMethod {
            method: "write_u16".to_string(),
            extra_args: vec![],
        }),
        (
            NativeType::Int {
                bits: 32,
                signed: true,
            },
            WireFormat::Bits { count: 32 },
        ) => Ok(CodecMethod {
            method: "write_i32".to_string(),
            extra_args: vec![],
        }),
        (
            NativeType::Int {
                bits: 32,
                signed: false,
            },
            WireFormat::Bits { count: 32 },
        ) => Ok(CodecMethod {
            method: "write_u32".to_string(),
            extra_args: vec![],
        }),
        (
            NativeType::Int {
                bits: 64,
                signed: true,
            },
            WireFormat::Bits { count: 64 },
        ) => Ok(CodecMethod {
            method: "write_i64".to_string(),
            extra_args: vec![],
        }),
        (
            NativeType::Int {
                bits: 64,
                signed: false,
            },
            WireFormat::Bits { count: 64 },
        ) => Ok(CodecMethod {
            method: "write_u64".to_string(),
            extra_args: vec![],
        }),

        // Integers with varint encoding
        (NativeType::Int { signed: true, .. }, WireFormat::LengthPrefixed { .. }) => {
            Ok(CodecMethod {
                method: "write_varint_signed".to_string(),
                extra_args: vec![],
            })
        }
        (NativeType::Int { signed: false, .. }, WireFormat::LengthPrefixed { .. }) => {
            Ok(CodecMethod {
                method: "write_varint_unsigned".to_string(),
                extra_args: vec![],
            })
        }

        // Floats
        (NativeType::Float { bits: 32 }, WireFormat::Bits { count: 32 }) => Ok(CodecMethod {
            method: "write_f32".to_string(),
            extra_args: vec![],
        }),
        (NativeType::Float { bits: 64 }, WireFormat::Bits { count: 64 }) => Ok(CodecMethod {
            method: "write_f64".to_string(),
            extra_args: vec![],
        }),

        // String
        (NativeType::String, WireFormat::LengthPrefixed { .. }) => Ok(CodecMethod {
            method: "write_string".to_string(),
            extra_args: vec![],
        }),

        // Unsupported combinations
        _ => anyhow::bail!(
            "Unsupported encoding combination: native={:?}, wire={:?}",
            encoding.native,
            encoding.wire
        ),
    }
}

/* -------------------------------------------------------------------------- */
/*                            Fn: get_read_method                             */
/* -------------------------------------------------------------------------- */

/// `get_read_method` returns the reader method name and extra arguments for an encoding.
pub fn get_read_method(encoding: &Encoding) -> anyhow::Result<CodecMethod> {
    let has_zigzag = encoding
        .transforms
        .iter()
        .any(|t| matches!(t, Transform::ZigZag));

    match (&encoding.native, &encoding.wire) {
        // Bool
        (NativeType::Bool, _) => Ok(CodecMethod {
            method: "read_bool".to_string(),
            extra_args: vec![],
        }),

        // Integers with zigzag encoding (must come before fixed-width)
        (NativeType::Int { .. }, WireFormat::Bits { count }) if has_zigzag => Ok(CodecMethod {
            method: "read_zigzag".to_string(),
            extra_args: vec![Expr::Literal((*count as i64).into())],
        }),

        // Integers with fixed-width encoding
        (
            NativeType::Int {
                bits: 8,
                signed: true,
            },
            WireFormat::Bits { count: 8 },
        ) => Ok(CodecMethod {
            method: "read_i8".to_string(),
            extra_args: vec![],
        }),
        (
            NativeType::Int {
                bits: 8,
                signed: false,
            },
            WireFormat::Bits { count: 8 },
        ) => Ok(CodecMethod {
            method: "read_u8".to_string(),
            extra_args: vec![],
        }),
        (
            NativeType::Int {
                bits: 16,
                signed: true,
            },
            WireFormat::Bits { count: 16 },
        ) => Ok(CodecMethod {
            method: "read_i16".to_string(),
            extra_args: vec![],
        }),
        (
            NativeType::Int {
                bits: 16,
                signed: false,
            },
            WireFormat::Bits { count: 16 },
        ) => Ok(CodecMethod {
            method: "read_u16".to_string(),
            extra_args: vec![],
        }),
        (
            NativeType::Int {
                bits: 32,
                signed: true,
            },
            WireFormat::Bits { count: 32 },
        ) => Ok(CodecMethod {
            method: "read_i32".to_string(),
            extra_args: vec![],
        }),
        (
            NativeType::Int {
                bits: 32,
                signed: false,
            },
            WireFormat::Bits { count: 32 },
        ) => Ok(CodecMethod {
            method: "read_u32".to_string(),
            extra_args: vec![],
        }),
        (
            NativeType::Int {
                bits: 64,
                signed: true,
            },
            WireFormat::Bits { count: 64 },
        ) => Ok(CodecMethod {
            method: "read_i64".to_string(),
            extra_args: vec![],
        }),
        (
            NativeType::Int {
                bits: 64,
                signed: false,
            },
            WireFormat::Bits { count: 64 },
        ) => Ok(CodecMethod {
            method: "read_u64".to_string(),
            extra_args: vec![],
        }),

        // Integers with varint encoding
        (NativeType::Int { signed: true, .. }, WireFormat::LengthPrefixed { .. }) => {
            Ok(CodecMethod {
                method: "read_varint_signed".to_string(),
                extra_args: vec![],
            })
        }
        (NativeType::Int { signed: false, .. }, WireFormat::LengthPrefixed { .. }) => {
            Ok(CodecMethod {
                method: "read_varint_unsigned".to_string(),
                extra_args: vec![],
            })
        }

        // Floats
        (NativeType::Float { bits: 32 }, WireFormat::Bits { count: 32 }) => Ok(CodecMethod {
            method: "read_f32".to_string(),
            extra_args: vec![],
        }),
        (NativeType::Float { bits: 64 }, WireFormat::Bits { count: 64 }) => Ok(CodecMethod {
            method: "read_f64".to_string(),
            extra_args: vec![],
        }),

        // String
        (NativeType::String, WireFormat::LengthPrefixed { .. }) => Ok(CodecMethod {
            method: "read_string".to_string(),
            extra_args: vec![],
        }),

        // Unsupported combinations
        _ => anyhow::bail!(
            "Unsupported encoding combination: native={:?}, wire={:?}",
            encoding.native,
            encoding.wire
        ),
    }
}

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    /* ---------------------- Tests: get_write_method ----------------------- */

    #[test]
    fn test_get_write_method_bool() {
        // Given: A bool encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 1 },
            native: NativeType::Bool,
            transforms: vec![],
            padding_bits: None,
        };

        // When: Getting the write method.
        let result = get_write_method(&encoding).unwrap();

        // Then: The method is write_bool.
        assert_eq!(result.method, "write_bool");
        assert_eq!(result.extra_args.len(), 0);
    }

    #[test]
    fn test_get_write_method_u8() {
        // Given: A u8 encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 8 },
            native: NativeType::Int {
                bits: 8,
                signed: false,
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Getting the write method.
        let result = get_write_method(&encoding).unwrap();

        // Then: The method is write_u8.
        assert_eq!(result.method, "write_u8");
        assert_eq!(result.extra_args.len(), 0);
    }

    #[test]
    fn test_get_write_method_varint_unsigned() {
        // Given: A varint unsigned encoding.
        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
            native: NativeType::Int {
                bits: 64,
                signed: false,
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Getting the write method.
        let result = get_write_method(&encoding).unwrap();

        // Then: The method is write_varint_unsigned.
        assert_eq!(result.method, "write_varint_unsigned");
        assert_eq!(result.extra_args.len(), 0);
    }

    #[test]
    fn test_get_write_method_zigzag() {
        // Given: A zigzag encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 32 },
            native: NativeType::Int {
                bits: 32,
                signed: true,
            },
            transforms: vec![Transform::ZigZag],
            padding_bits: None,
        };

        // When: Getting the write method.
        let result = get_write_method(&encoding).unwrap();

        // Then: The method is write_zigzag with bit count.
        assert_eq!(result.method, "write_zigzag");
        assert_eq!(result.extra_args.len(), 1);
    }

    #[test]
    fn test_get_write_method_string() {
        // Given: A string encoding.
        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
            native: NativeType::String,
            transforms: vec![],
            padding_bits: None,
        };

        // When: Getting the write method.
        let result = get_write_method(&encoding).unwrap();

        // Then: The method is write_string.
        assert_eq!(result.method, "write_string");
        assert_eq!(result.extra_args.len(), 0);
    }

    /* ----------------------- Tests: get_read_method ----------------------- */

    #[test]
    fn test_get_read_method_bool() {
        // Given: A bool encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 1 },
            native: NativeType::Bool,
            transforms: vec![],
            padding_bits: None,
        };

        // When: Getting the read method.
        let result = get_read_method(&encoding).unwrap();

        // Then: The method is read_bool.
        assert_eq!(result.method, "read_bool");
        assert_eq!(result.extra_args.len(), 0);
    }

    #[test]
    fn test_get_read_method_i32() {
        // Given: An i32 encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 32 },
            native: NativeType::Int {
                bits: 32,
                signed: true,
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Getting the read method.
        let result = get_read_method(&encoding).unwrap();

        // Then: The method is read_i32.
        assert_eq!(result.method, "read_i32");
        assert_eq!(result.extra_args.len(), 0);
    }

    #[test]
    fn test_get_read_method_f64() {
        // Given: An f64 encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 64 },
            native: NativeType::Float { bits: 64 },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Getting the read method.
        let result = get_read_method(&encoding).unwrap();

        // Then: The method is read_f64.
        assert_eq!(result.method, "read_f64");
        assert_eq!(result.extra_args.len(), 0);
    }
}
