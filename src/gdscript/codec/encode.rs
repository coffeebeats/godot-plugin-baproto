use baproto::{Encoding, NativeType};

use crate::gdscript::ast::{Assignment, Expr, FnCall, ForInBuilder, IfBuilder, Item, Operator};

use super::wire::get_write_method;

/* -------------------------------------------------------------------------- */
/*                            Fn: gen_encode_stmts                            */
/* -------------------------------------------------------------------------- */

/// `gen_encode_stmts` generates encode statements for a field.
pub fn gen_encode_stmts(field_name: &str, encoding: &Encoding) -> anyhow::Result<Vec<Item>> {
    match &encoding.native {
        // Primitives: Bool, Int, Float, String
        NativeType::Bool
        | NativeType::Int { .. }
        | NativeType::Float { .. }
        | NativeType::String => gen_encode_primitive(field_name, encoding),

        // Bytes
        NativeType::Bytes => gen_encode_bytes(field_name),

        // Array
        NativeType::Array { element } => gen_encode_array(field_name, element),

        // Map
        NativeType::Map { key, value } => gen_encode_map(field_name, key, value),

        // Message
        NativeType::Message { .. } => gen_encode_message(field_name),

        // Enum
        NativeType::Enum { .. } => gen_encode_enum(field_name),
    }
}

/* ------------------------ Fn: gen_encode_primitive ------------------------ */

/// `gen_encode_primitive` generates encoding for a primitive field.
///
/// # Generated GDScript
/// ```gdscript
/// _writer.write_xxx(field_name)
/// ```
fn gen_encode_primitive(field_name: &str, encoding: &Encoding) -> anyhow::Result<Vec<Item>> {
    let method = get_write_method(encoding)?;

    let mut args = vec![Expr::ident(field_name)];
    args.extend(method.extra_args);

    let call = FnCall::method_args(Expr::ident("_writer"), &method.method, args);

    Ok(vec![Item::Expr(call)])
}

/* -------------------------- Fn: gen_encode_bytes -------------------------- */

/// `gen_encode_bytes` generates encoding for a bytes field.
///
/// # Generated GDScript
/// ```gdscript
/// _writer.write_varint_unsigned(data.size())
/// _writer.write_bytes(data)
/// ```
fn gen_encode_bytes(field_name: &str) -> anyhow::Result<Vec<Item>> {
    // Write length prefix
    let size_call = FnCall::method(Expr::ident(field_name), "size");
    let write_length = FnCall::method_args(
        Expr::ident("_writer"),
        "write_varint_unsigned",
        vec![size_call],
    );

    // Write bytes
    let write_bytes = FnCall::method_args(
        Expr::ident("_writer"),
        "write_bytes",
        vec![Expr::ident(field_name)],
    );

    Ok(vec![Item::Expr(write_length), Item::Expr(write_bytes)])
}

/* -------------------------- Fn: gen_encode_array -------------------------- */

/// `gen_encode_array` generates encoding for an array field.
fn gen_encode_array(field_name: &str, element: &Encoding) -> anyhow::Result<Vec<Item>> {
    match &element.native {
        // Array of messages or enums requires null checks
        NativeType::Message { .. } | NativeType::Enum { .. } => {
            gen_encode_array_message(field_name, element)
        }

        // All other types (primitives, bytes, etc.) can be encoded directly
        _ => gen_encode_array_primitive(field_name, element),
    }
}

/* --------------------- Fn: gen_encode_array_primitive --------------------- */

/// `gen_encode_array_primitive` generates encoding for an array of primitives.
///
/// # Generated GDScript
/// ```gdscript
/// _writer.write_varint_unsigned(items.size())
/// for _item in items:
///     _writer.write_xxx(_item)
/// ```
fn gen_encode_array_primitive(field_name: &str, element: &Encoding) -> anyhow::Result<Vec<Item>> {
    // Write array length
    let size_call = FnCall::method(Expr::ident(field_name), "size");
    let write_length = FnCall::method_args(
        Expr::ident("_writer"),
        "write_varint_unsigned",
        vec![size_call],
    );

    // Generate encoding statements for element
    let element_stmts = gen_encode_stmts("_item", element)?;

    // Create for loop
    let for_loop = Item::ForIn(
        ForInBuilder::default()
            .variable("_item")
            .iterable(Expr::ident(field_name))
            .body(element_stmts)
            .build()
            .unwrap(),
    );

    Ok(vec![Item::Expr(write_length), for_loop])
}

/* ---------------------- Fn: gen_encode_array_message ---------------------- */

/// `gen_encode_array_message` generates encoding for an array of messages.
///
/// # Generated GDScript
/// ```gdscript
/// _writer.write_varint_unsigned(items.size())
/// for _item in items:
///     if _item == null:
///         _writer.set_error(ERR_INVALID_DATA)
///         return
///     _item._encode(_writer)
/// ```
fn gen_encode_array_message(field_name: &str, _element: &Encoding) -> anyhow::Result<Vec<Item>> {
    // Write array length
    let size_call = FnCall::method(Expr::ident(field_name), "size");
    let write_length = FnCall::method_args(
        Expr::ident("_writer"),
        "write_varint_unsigned",
        vec![size_call],
    );

    // Null check for message
    let null_check = gen_null_check("_item");

    // Call _item._encode(_writer)
    let encode_call = Item::Expr(FnCall::method_args(
        Expr::ident("_item"),
        "_encode",
        vec![Expr::ident("_writer")],
    ));

    // Create for loop
    let for_loop = Item::ForIn(
        ForInBuilder::default()
            .variable("_item")
            .iterable(Expr::ident(field_name))
            .body(vec![null_check, encode_call])
            .build()
            .unwrap(),
    );

    Ok(vec![Item::Expr(write_length), for_loop])
}

/* ------------------------- Fn: gen_encode_message ------------------------- */

/// `gen_encode_message` generates encoding for a message field.
///
/// # Generated GDScript
/// ```gdscript
/// if player == null:
///     _writer.set_error(ERR_INVALID_DATA)
///     return
/// player._encode(_writer)
/// ```
fn gen_encode_message(field_name: &str) -> anyhow::Result<Vec<Item>> {
    // Null check
    let null_check = gen_null_check(field_name);

    // Call field._encode(_writer)
    let encode_call = Item::Expr(FnCall::method_args(
        Expr::ident(field_name),
        "_encode",
        vec![Expr::ident("_writer")],
    ));

    Ok(vec![null_check, encode_call])
}

/* -------------------------- Fn: gen_encode_enum --------------------------- */

/// `gen_encode_enum` generates encoding for an enum field.
///
/// # Generated GDScript
/// ```gdscript
/// if status == null:
///     _writer.set_error(ERR_INVALID_DATA)
///     return
/// status._encode(_writer)
/// ```
fn gen_encode_enum(field_name: &str) -> anyhow::Result<Vec<Item>> {
    // Null check
    let null_check = gen_null_check(field_name);

    // Call field._encode(_writer)
    let encode_call = Item::Expr(FnCall::method_args(
        Expr::ident(field_name),
        "_encode",
        vec![Expr::ident("_writer")],
    ));

    Ok(vec![null_check, encode_call])
}

/* --------------------------- Fn: gen_encode_map --------------------------- */

/// `gen_encode_map` generates encoding for a map field.
fn gen_encode_map(field_name: &str, key: &Encoding, value: &Encoding) -> anyhow::Result<Vec<Item>> {
    match &value.native {
        // Map of messages or enums requires null checks
        NativeType::Message { .. } | NativeType::Enum { .. } => {
            gen_encode_map_message(field_name, key, value)
        }

        // All other types (primitives, bytes, etc.) can be encoded directly
        _ => gen_encode_map_primitive(field_name, key, value),
    }
}

/* ---------------------- Fn: gen_encode_map_primitive ---------------------- */

/// `gen_encode_map_primitive` generates encoding for a map with primitive values.
///
/// # Generated GDScript
/// ```gdscript
/// _writer.write_varint_unsigned(stats.size())
/// for _key in stats:
///     _writer.write_xxx(_key)
///     _writer.write_yyy(stats[_key])
/// ```
fn gen_encode_map_primitive(
    field_name: &str,
    key: &Encoding,
    value: &Encoding,
) -> anyhow::Result<Vec<Item>> {
    // Write map size
    let size_call = FnCall::method(Expr::ident(field_name), "size");
    let write_length = FnCall::method_args(
        Expr::ident("_writer"),
        "write_varint_unsigned",
        vec![size_call],
    );

    // Generate encoding statements for key
    let key_stmts = gen_encode_stmts("_key", key)?;

    // Generate encoding for value: _writer.write_xxx(field_name[_key])
    let value_method = get_write_method(value)?;
    let value_access = Expr::index(Expr::ident(field_name), Expr::ident("_key"));

    let mut value_args = vec![value_access];
    value_args.extend(value_method.extra_args);

    let value_write = Item::Expr(FnCall::method_args(
        Expr::ident("_writer"),
        &value_method.method,
        value_args,
    ));

    // Build loop body
    let mut loop_body = key_stmts;
    loop_body.push(value_write);

    // Create for loop
    let for_loop = Item::ForIn(
        ForInBuilder::default()
            .variable("_key")
            .iterable(Expr::ident(field_name))
            .body(loop_body)
            .build()
            .unwrap(),
    );

    Ok(vec![Item::Expr(write_length), for_loop])
}

/* ----------------------- Fn: gen_encode_map_message ----------------------- */

/// `gen_encode_map_message` generates encoding for a map with message values.
///
/// # Generated GDScript
/// ```gdscript
/// _writer.write_varint_unsigned(players.size())
/// for _key in players:
///     _writer.write_xxx(_key)
///     var _value := players[_key]
///     if _value == null:
///         _writer.set_error(ERR_INVALID_DATA)
///         return
///     _value._encode(_writer)
/// ```
fn gen_encode_map_message(
    field_name: &str,
    key: &Encoding,
    _value: &Encoding,
) -> anyhow::Result<Vec<Item>> {
    // Write map size
    let size_call = FnCall::method(Expr::ident(field_name), "size");
    let write_length = FnCall::method_args(
        Expr::ident("_writer"),
        "write_varint_unsigned",
        vec![size_call],
    );

    // Generate encoding statements for key
    let key_stmts = gen_encode_stmts("_key", key)?;

    // Declare value variable: var _value := field_name[_key]
    let value_access = Expr::index(Expr::ident(field_name), Expr::ident("_key"));
    let declare_value = Assignment::var("_value", value_access);

    // Null check for message value
    let null_check = gen_null_check("_value");

    // Call _value._encode(_writer)
    let encode_call = Item::Expr(FnCall::method_args(
        Expr::ident("_value"),
        "_encode",
        vec![Expr::ident("_writer")],
    ));

    // Build loop body
    let mut loop_body = key_stmts;
    loop_body.push(Item::Assignment(declare_value));
    loop_body.push(null_check);
    loop_body.push(encode_call);

    // Create for loop
    let for_loop = Item::ForIn(
        ForInBuilder::default()
            .variable("_key")
            .iterable(Expr::ident(field_name))
            .body(loop_body)
            .build()
            .unwrap(),
    );

    Ok(vec![Item::Expr(write_length), for_loop])
}

/* -------------------------------------------------------------------------- */
/*                           Fn: gen_null_check                               */
/* -------------------------------------------------------------------------- */

/// `gen_null_check` generates an early return if field is null.
///
/// # Generated GDScript
/// ```gdscript
/// if field == null:
///     _writer.set_error(ERR_INVALID_DATA)
///     return
/// ```
fn gen_null_check(field_name: &str) -> Item {
    let condition = Expr::binary_op(Expr::ident(field_name), Operator::Eq, Expr::null());

    let set_error = Item::Expr(FnCall::method_args(
        Expr::ident("_writer"),
        "set_error",
        vec![Expr::ident("ERR_INVALID_DATA")],
    ));

    let return_stmt = Item::Return(Expr::null());

    Item::If(
        IfBuilder::default()
            .condition(condition)
            .then_body(vec![set_error, return_stmt].into())
            .build()
            .unwrap(),
    )
}

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use baproto::{Encoding, NativeType, StringWriter, Transform, WireFormat};

    use crate::gdscript::GDScript;
    use crate::gdscript::ast::Emit;

    use super::*;

    /* ---------------------- Tests: gen_null_check --------------------- */

    #[test]
    fn test_gen_null_check() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // When: Generating a null check for a field.
        let item = gen_null_check("player");

        // When: The item is serialized to source code.
        let result = item.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expected content exactly.
        let actual = s.into_content();

        let expected = r#"if player == null:
	_writer.set_error(ERR_INVALID_DATA)
	return null"#;

        assert_eq!(actual, expected);
    }

    /* ------------------ Tests: gen_encode_primitive ------------------- */

    #[test]
    fn test_gen_encode_primitive_bool() {
        // Given: A bool encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 1 },
            native: NativeType::Bool,
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("active", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_bool.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_bool(active)");
    }

    #[test]
    fn test_gen_encode_primitive_u8() {
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

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("count", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_u8.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_u8(count)");
    }

    #[test]
    fn test_gen_encode_primitive_string() {
        // Given: A string encoding.
        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
            native: NativeType::String,
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("name", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_string.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_string(name)");
    }

    #[test]
    fn test_gen_encode_primitive_i8() {
        // Given: An i8 encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 8 },
            native: NativeType::Int {
                bits: 8,
                signed: true,
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("value", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_i8.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_i8(value)");
    }

    #[test]
    fn test_gen_encode_primitive_i16() {
        // Given: An i16 encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 16 },
            native: NativeType::Int {
                bits: 16,
                signed: true,
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("value", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_i16.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_i16(value)");
    }

    #[test]
    fn test_gen_encode_primitive_i32() {
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

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("value", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_i32.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_i32(value)");
    }

    #[test]
    fn test_gen_encode_primitive_i64() {
        // Given: An i64 encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 64 },
            native: NativeType::Int {
                bits: 64,
                signed: true,
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("value", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_i64.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_i64(value)");
    }

    #[test]
    fn test_gen_encode_primitive_u16() {
        // Given: A u16 encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 16 },
            native: NativeType::Int {
                bits: 16,
                signed: false,
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("value", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_u16.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_u16(value)");
    }

    #[test]
    fn test_gen_encode_primitive_u32() {
        // Given: A u32 encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 32 },
            native: NativeType::Int {
                bits: 32,
                signed: false,
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("value", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_u32.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_u32(value)");
    }

    #[test]
    fn test_gen_encode_primitive_u64() {
        // Given: A u64 encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 64 },
            native: NativeType::Int {
                bits: 64,
                signed: false,
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("value", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_u64.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_u64(value)");
    }

    #[test]
    fn test_gen_encode_primitive_f32() {
        // Given: An f32 encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 32 },
            native: NativeType::Float { bits: 32 },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("value", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_f32.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_f32(value)");
    }

    #[test]
    fn test_gen_encode_primitive_f64() {
        // Given: An f64 encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 64 },
            native: NativeType::Float { bits: 64 },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("value", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_f64.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_f64(value)");
    }

    #[test]
    fn test_gen_encode_primitive_varint_signed() {
        // Given: A signed varint encoding.
        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
            native: NativeType::Int {
                bits: 64,
                signed: true,
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("value", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_varint_signed.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_varint_signed(value)");
    }

    #[test]
    fn test_gen_encode_primitive_varint_unsigned() {
        // Given: An unsigned varint encoding.
        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
            native: NativeType::Int {
                bits: 64,
                signed: false,
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("value", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_varint_unsigned.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_varint_unsigned(value)");
    }

    #[test]
    fn test_gen_encode_primitive_zigzag() {
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

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("value", &encoding).unwrap();

        // Then: A single method call is generated.
        assert_eq!(stmts.len(), 1);

        // Then: It calls write_zigzag with bit count.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "_writer.write_zigzag(value, 32)");
    }

    /* -------------------- Tests: gen_encode_bytes --------------------- */

    #[test]
    fn test_gen_encode_bytes() {
        // Given: A bytes encoding.
        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
            native: NativeType::Bytes,
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("data", &encoding).unwrap();

        // Then: Two statements are generated (length + bytes).
        assert_eq!(stmts.len(), 2);

        // Then: First statement writes the length.
        let mut s1 = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s1).unwrap();
        let actual1 = s1.into_content();
        assert_eq!(actual1, "_writer.write_varint_unsigned(data.size())");

        // Then: Second statement writes the bytes.
        let mut s2 = StringWriter::default();
        stmts[1].emit(&mut cw, &mut s2).unwrap();
        assert_eq!(s2.into_content(), "_writer.write_bytes(data)");
    }

    /* -------------------- Tests: gen_encode_array --------------------- */

    #[test]
    fn test_gen_encode_array_primitive() {
        // Given: An array of i32 encoding.
        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
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

        // Then: Two statements are generated (length + for loop).
        assert_eq!(stmts.len(), 2);

        // Then: First statement writes array length.
        let mut s1 = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s1).unwrap();
        let actual1 = s1.into_content();
        assert_eq!(actual1, "_writer.write_varint_unsigned(items.size())");

        // Then: Second statement is for loop.
        let mut s2 = StringWriter::default();
        stmts[1].emit(&mut cw, &mut s2).unwrap();
        let actual2 = s2.into_content();

        let expected2 = r#"for _item in items:
	_writer.write_i32(_item)"#;

        assert_eq!(actual2, expected2);
    }

    #[test]
    fn test_gen_encode_array_message() {
        use baproto::DescriptorBuilder;

        // Given: An array of messages encoding.
        let descriptor = DescriptorBuilder::default()
            .package(baproto::PackageName::try_from(vec!["test"]).unwrap())
            .path(vec!["Player".to_string()])
            .build()
            .unwrap();

        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
            native: NativeType::Array {
                element: Box::new(Encoding {
                    wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
                    native: NativeType::Message { descriptor },
                    transforms: vec![],
                    padding_bits: None,
                }),
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("players", &encoding).unwrap();

        // Then: Two statements are generated (length + for loop).
        assert_eq!(stmts.len(), 2);

        // Then: Second statement is for loop with null check and encode.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[1].emit(&mut cw, &mut s).unwrap();
        let actual = s.into_content();

        let expected = r#"for _item in players:
	if _item == null:
		_writer.set_error(ERR_INVALID_DATA)
		return null
	_item._encode(_writer)"#;

        assert_eq!(actual, expected);
    }

    /* --------------------- Tests: gen_encode_map ---------------------- */

    #[test]
    fn test_gen_encode_map_primitive() {
        // Given: A map with String keys and i32 values.
        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
            native: NativeType::Map {
                key: Box::new(Encoding {
                    wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
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
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("stats", &encoding).unwrap();

        // Then: Two statements are generated (length + for loop).
        assert_eq!(stmts.len(), 2);

        // Then: First statement writes map size.
        let mut s1 = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s1).unwrap();
        let actual1 = s1.into_content();
        assert_eq!(actual1, "_writer.write_varint_unsigned(stats.size())");

        // Then: Second statement is for loop.
        let mut s2 = StringWriter::default();
        stmts[1].emit(&mut cw, &mut s2).unwrap();
        let actual2 = s2.into_content();

        let expected2 = r#"for _key in stats:
	_writer.write_string(_key)
	_writer.write_i32(stats[_key])"#;

        assert_eq!(actual2, expected2);
    }

    #[test]
    fn test_gen_encode_map_message() {
        use baproto::DescriptorBuilder;

        // Given: A map with String keys and message values.
        let descriptor = DescriptorBuilder::default()
            .package(baproto::PackageName::try_from(vec!["test"]).unwrap())
            .path(vec!["Player".to_string()])
            .build()
            .unwrap();

        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
            native: NativeType::Map {
                key: Box::new(Encoding {
                    wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
                    native: NativeType::String,
                    transforms: vec![],
                    padding_bits: None,
                }),
                value: Box::new(Encoding {
                    wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
                    native: NativeType::Message { descriptor },
                    transforms: vec![],
                    padding_bits: None,
                }),
            },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("players", &encoding).unwrap();

        // Then: Two statements are generated (length + for loop).
        assert_eq!(stmts.len(), 2);

        // Then: Second statement is for loop with null check and encode.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[1].emit(&mut cw, &mut s).unwrap();
        let actual = s.into_content();

        let expected = r#"for _key in players:
	_writer.write_string(_key)
	var _value := players[_key]
	if _value == null:
		_writer.set_error(ERR_INVALID_DATA)
		return null
	_value._encode(_writer)"#;

        assert_eq!(actual, expected);
    }

    /* -------------------- Tests: gen_encode_message ------------------- */

    #[test]
    fn test_gen_encode_message() {
        use baproto::DescriptorBuilder;

        // Given: A message encoding.
        let descriptor = DescriptorBuilder::default()
            .package(baproto::PackageName::try_from(vec!["test"]).unwrap())
            .path(vec!["Player".to_string()])
            .build()
            .unwrap();

        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
            native: NativeType::Message { descriptor },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating encode statements.
        let stmts = gen_encode_stmts("player", &encoding).unwrap();

        // Then: Two statements are generated (null check + encode call).
        assert_eq!(stmts.len(), 2);

        // Then: First statement is null check.
        let mut s1 = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s1).unwrap();
        let actual1 = s1.into_content();

        let expected1 = r#"if player == null:
	_writer.set_error(ERR_INVALID_DATA)
	return null"#;

        assert_eq!(actual1, expected1);

        // Then: Second statement is encode call.
        let mut s2 = StringWriter::default();
        stmts[1].emit(&mut cw, &mut s2).unwrap();
        assert_eq!(s2.into_content(), "player._encode(_writer)");
    }
}
