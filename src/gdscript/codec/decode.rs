use baproto::{Encoding, NativeType};

use crate::gdscript::ast::{Assignment, Expr, FnCall, ForInBuilder, IfBuilder, Item, Operator};

use super::wire::get_read_method;

/* -------------------------------------------------------------------------- */
/*                            Fn: gen_decode_stmts                            */
/* -------------------------------------------------------------------------- */

/// `gen_decode_stmts` generates decode statements for a field.
pub fn gen_decode_stmts(field_name: &str, encoding: &Encoding) -> anyhow::Result<Vec<Item>> {
    match &encoding.native {
        // Primitives: Bool, Int, Float, String
        NativeType::Bool
        | NativeType::Int { .. }
        | NativeType::Float { .. }
        | NativeType::String => gen_decode_primitive(field_name, encoding),

        // Bytes
        NativeType::Bytes => gen_decode_bytes(field_name),

        // Array
        NativeType::Array { element } => gen_decode_array(field_name, element),

        // Map
        NativeType::Map { key, value } => gen_decode_map(field_name, key, value),

        // Message
        NativeType::Message { descriptor } => gen_decode_message(field_name, descriptor),

        // Enum
        NativeType::Enum { descriptor } => gen_decode_enum(field_name, descriptor),
    }
}

/* ------------------------ Fn: gen_decode_primitive ------------------------ */

/// `gen_decode_primitive` generates decoding for a primitive field.
///
/// # Generated GDScript
/// ```gdscript
/// field_name = _reader.read_xxx()
/// if _reader.get_error() != OK:
///     return _reader.get_error()
/// ```
fn gen_decode_primitive(field_name: &str, encoding: &Encoding) -> anyhow::Result<Vec<Item>> {
    let method = get_read_method(encoding)?;

    let call = FnCall::method_args(Expr::ident("_reader"), &method.method, method.extra_args);

    let assignment = Assignment::reassign(field_name, call);

    Ok(vec![Item::Assignment(assignment), gen_reader_error_check()])
}

/* -------------------------- Fn: gen_decode_bytes -------------------------- */

/// `gen_decode_bytes` generates decoding for a bytes field.
///
/// # Generated GDScript
/// ```gdscript
/// data = _reader.read_bytes(_reader.read_varint_unsigned())
/// if _reader.get_error() != OK:
///     return _reader.get_error()
/// ```
fn gen_decode_bytes(field_name: &str) -> anyhow::Result<Vec<Item>> {
    // Read length
    let length_call = FnCall::method(Expr::ident("_reader"), "read_varint_unsigned");

    // Read bytes with length
    let read_call = FnCall::method_args(Expr::ident("_reader"), "read_bytes", vec![length_call]);

    let assignment = Assignment::reassign(field_name, read_call);

    Ok(vec![Item::Assignment(assignment), gen_reader_error_check()])
}

/* -------------------------- Fn: gen_decode_array -------------------------- */

/// `gen_decode_array` generates decoding for an array field.
fn gen_decode_array(field_name: &str, element: &Encoding) -> anyhow::Result<Vec<Item>> {
    match &element.native {
        // Array of messages or enums requires construction
        NativeType::Message { .. } | NativeType::Enum { .. } => {
            gen_decode_array_message(field_name, element)
        }

        // All other types (primitives, bytes, etc.) can be decoded directly
        _ => gen_decode_array_primitive(field_name, element),
    }
}

/* --------------------- Fn: gen_decode_array_primitive --------------------- */

/// `gen_decode_array_primitive` generates decoding for an array of primitives.
///
/// # Generated GDScript
/// ```gdscript
/// items = []
/// for _i in range(_reader.read_varint_unsigned()):
///     items.append(_reader.read_xxx())
///     if _reader.get_error() != OK:
///         return _reader.get_error()
/// ```
fn gen_decode_array_primitive(field_name: &str, element: &Encoding) -> anyhow::Result<Vec<Item>> {
    // Initialize empty array
    let init = Assignment::reassign(field_name, Expr::empty_array());

    // Read element
    let element_stmts = gen_decode_stmts("_temp", element)?;

    // Append element to array
    let append_call = Item::Expr(FnCall::method_args(
        Expr::ident(field_name),
        "append",
        vec![Expr::ident("_temp")],
    ));

    // Combine element read statements with append
    let mut loop_body = element_stmts;
    loop_body.push(append_call);

    // Read length and create range
    let length_call = FnCall::method(Expr::ident("_reader"), "read_varint_unsigned");
    let range_call = FnCall::function_args("range", vec![length_call]);

    // Create for loop
    let for_loop = Item::ForIn(
        ForInBuilder::default()
            .variable("_i")
            .iterable(range_call)
            .body(loop_body)
            .build()
            .unwrap(),
    );

    Ok(vec![Item::Assignment(init), for_loop])
}

/* ---------------------- Fn: gen_decode_array_message ---------------------- */

/// `gen_decode_array_message` generates decoding for an array of messages.
///
/// # Generated GDScript
/// ```gdscript
/// items = []
/// for _i in range(_reader.read_varint_unsigned()):
///     var _item := Player.new()
///     _item._decode(_reader)
///     if _reader.get_error() != OK:
///         return _reader.get_error()
///     items.append(_item)
/// ```
fn gen_decode_array_message(field_name: &str, element: &Encoding) -> anyhow::Result<Vec<Item>> {
    // Initialize empty array
    let init = Assignment::reassign(field_name, Expr::empty_array());

    // Get type name
    let type_name = match &element.native {
        NativeType::Message { descriptor } | NativeType::Enum { descriptor } => {
            descriptor.path.join("_")
        }
        _ => anyhow::bail!("Expected message or enum type"),
    };

    // Create message instance: var _item := MessageType.new()
    let new_call = FnCall::method(Expr::ident(&type_name), "new");
    let declare_item = Assignment::var("_item", new_call);

    // Call _item._decode(_reader)
    let decode_call = Item::Expr(FnCall::method_args(
        Expr::ident("_item"),
        "_decode",
        vec![Expr::ident("_reader")],
    ));

    // Error check
    let error_check = gen_reader_error_check();

    // Append item to array
    let append_call = Item::Expr(FnCall::method_args(
        Expr::ident(field_name),
        "append",
        vec![Expr::ident("_item")],
    ));

    // Read length and create range
    let length_call = FnCall::method(Expr::ident("_reader"), "read_varint_unsigned");
    let range_call = FnCall::function_args("range", vec![length_call]);

    // Create for loop
    let for_loop = Item::ForIn(
        ForInBuilder::default()
            .variable("_i")
            .iterable(range_call)
            .body(vec![
                Item::Assignment(declare_item),
                decode_call,
                error_check,
                append_call,
            ])
            .build()
            .unwrap(),
    );

    Ok(vec![Item::Assignment(init), for_loop])
}

/* ------------------------- Fn: gen_decode_message ------------------------- */

/// `gen_decode_message` generates decoding for a message field.
///
/// # Generated GDScript
/// ```gdscript
/// player = Player.new()
/// player._decode(_reader)
/// if _reader.get_error() != OK:
///     return _reader.get_error()
/// ```
fn gen_decode_message(
    field_name: &str,
    descriptor: &baproto::Descriptor,
) -> anyhow::Result<Vec<Item>> {
    // Get message type name
    let type_name = descriptor.path.join("_");

    // Create message instance: field = MessageType.new()
    let new_call = FnCall::method(Expr::ident(&type_name), "new");
    let assignment = Assignment::reassign(field_name, new_call);

    // Call field._decode(_reader)
    let decode_call = Item::Expr(FnCall::method_args(
        Expr::ident(field_name),
        "_decode",
        vec![Expr::ident("_reader")],
    ));

    // Error check
    let error_check = gen_reader_error_check();

    Ok(vec![Item::Assignment(assignment), decode_call, error_check])
}

/* -------------------------- Fn: gen_decode_enum --------------------------- */

/// `gen_decode_enum` generates decoding for an enum field.
///
/// # Generated GDScript
/// ```gdscript
/// status = Status.new()
/// status._decode(_reader)
/// if _reader.get_error() != OK:
///     return _reader.get_error()
/// ```
fn gen_decode_enum(
    field_name: &str,
    descriptor: &baproto::Descriptor,
) -> anyhow::Result<Vec<Item>> {
    // Get enum type name
    let type_name = descriptor.path.join("_");

    // Create enum instance: field = EnumType.new()
    let new_call = FnCall::method(Expr::ident(&type_name), "new");
    let assignment = Assignment::reassign(field_name, new_call);

    // Call field._decode(_reader)
    let decode_call = Item::Expr(FnCall::method_args(
        Expr::ident(field_name),
        "_decode",
        vec![Expr::ident("_reader")],
    ));

    // Error check
    let error_check = gen_reader_error_check();

    Ok(vec![Item::Assignment(assignment), decode_call, error_check])
}

/* --------------------------- Fn: gen_decode_map --------------------------- */

/// `gen_decode_map` generates decoding for a map field.
fn gen_decode_map(field_name: &str, key: &Encoding, value: &Encoding) -> anyhow::Result<Vec<Item>> {
    match &value.native {
        // Map of messages or enums requires construction
        NativeType::Message { .. } | NativeType::Enum { .. } => {
            gen_decode_map_message(field_name, key, value)
        }

        // All other types (primitives, bytes, etc.) can be decoded directly
        _ => gen_decode_map_primitive(field_name, key, value),
    }
}

/* ---------------------- Fn: gen_decode_map_primitive ---------------------- */

/// `gen_decode_map_primitive` generates decoding for a map with primitive values.
///
/// # Generated GDScript
/// ```gdscript
/// stats = {}
/// for _i in range(_reader.read_varint_unsigned()):
///     var _key := _reader.read_xxx()
///     if _reader.get_error() != OK:
///         return _reader.get_error()
///     stats[_key] = _reader.read_yyy()
///     if _reader.get_error() != OK:
///         return _reader.get_error()
/// ```
fn gen_decode_map_primitive(
    field_name: &str,
    key: &Encoding,
    value: &Encoding,
) -> anyhow::Result<Vec<Item>> {
    // Initialize empty dict
    let init = Assignment::reassign(field_name, Expr::empty_dict());

    // Read key: var _key := _reader.read_xxx()
    let key_method = get_read_method(key)?;
    let key_read = FnCall::method_args(
        Expr::ident("_reader"),
        &key_method.method,
        key_method.extra_args,
    );
    let declare_key = Assignment::var("_key", key_read);

    // Error check after key read
    let key_error_check = gen_reader_error_check();

    // Read value: stats[_key] = _reader.read_yyy()
    let value_method = get_read_method(value)?;
    let value_read = FnCall::method_args(
        Expr::ident("_reader"),
        &value_method.method,
        value_method.extra_args,
    );
    let map_index = Expr::index(Expr::ident(field_name), Expr::ident("_key"));
    let assign_value = Assignment::reassign(map_index, value_read);

    // Error check after value read
    let value_error_check = gen_reader_error_check();

    // Build loop body
    let loop_body = vec![
        Item::Assignment(declare_key),
        key_error_check,
        Item::Assignment(assign_value),
        value_error_check,
    ];

    // Read length and create range
    let length_call = FnCall::method(Expr::ident("_reader"), "read_varint_unsigned");
    let range_call = FnCall::function_args("range", vec![length_call]);

    // Create for loop
    let for_loop = Item::ForIn(
        ForInBuilder::default()
            .variable("_i")
            .iterable(range_call)
            .body(loop_body)
            .build()
            .unwrap(),
    );

    Ok(vec![Item::Assignment(init), for_loop])
}

/* ----------------------- Fn: gen_decode_map_message ----------------------- */

/// `gen_decode_map_message` generates decoding for a map with message values.
///
/// # Generated GDScript
/// ```gdscript
/// players = {}
/// for _i in range(_reader.read_varint_unsigned()):
///     var _key := _reader.read_string()
///     if _reader.get_error() != OK:
///         return _reader.get_error()
///     var _value := Player.new()
///     _value._decode(_reader)
///     if _reader.get_error() != OK:
///         return _reader.get_error()
///     players[_key] = _value
/// ```
fn gen_decode_map_message(
    field_name: &str,
    key: &Encoding,
    value: &Encoding,
) -> anyhow::Result<Vec<Item>> {
    // Initialize empty dict
    let init = Assignment::reassign(field_name, Expr::empty_dict());

    // Read key: var _key := _reader.read_xxx()
    let key_method = get_read_method(key)?;
    let key_read = FnCall::method_args(
        Expr::ident("_reader"),
        &key_method.method,
        key_method.extra_args,
    );
    let declare_key = Assignment::var("_key", key_read);

    // Error check after key read
    let key_error_check = gen_reader_error_check();

    // Get type name
    let type_name = match &value.native {
        NativeType::Message { descriptor } | NativeType::Enum { descriptor } => {
            descriptor.path.join("_")
        }
        _ => anyhow::bail!("Expected message or enum type"),
    };

    // Create message instance: var _value := MessageType.new()
    let new_call = FnCall::method(Expr::ident(&type_name), "new");
    let declare_value = Assignment::var("_value", new_call);

    // Call _value._decode(_reader)
    let decode_call = Item::Expr(FnCall::method_args(
        Expr::ident("_value"),
        "_decode",
        vec![Expr::ident("_reader")],
    ));

    // Error check after decode
    let decode_error_check = gen_reader_error_check();

    // Assign to map: players[_key] = _value
    let map_index = Expr::index(Expr::ident(field_name), Expr::ident("_key"));
    let assign_to_map = Assignment::reassign(map_index, Expr::ident("_value"));

    // Build loop body
    let loop_body = vec![
        Item::Assignment(declare_key),
        key_error_check,
        Item::Assignment(declare_value),
        decode_call,
        decode_error_check,
        Item::Assignment(assign_to_map),
    ];

    // Read length and create range
    let length_call = FnCall::method(Expr::ident("_reader"), "read_varint_unsigned");
    let range_call = FnCall::function_args("range", vec![length_call]);

    // Create for loop
    let for_loop = Item::ForIn(
        ForInBuilder::default()
            .variable("_i")
            .iterable(range_call)
            .body(loop_body)
            .build()
            .unwrap(),
    );

    Ok(vec![Item::Assignment(init), for_loop])
}

/* -------------------------------------------------------------------------- */
/*                         Fn: gen_reader_error_check                         */
/* -------------------------------------------------------------------------- */

/// `gen_reader_error_check` generates an early return if reader has an error.
///
/// # Generated GDScript
/// ```gdscript
/// if _reader.get_error() != OK:
///     return _reader.get_error()
/// ```
fn gen_reader_error_check() -> Item {
    let condition = Expr::binary_op(
        FnCall::method(Expr::ident("_reader"), "get_error"),
        Operator::NotEq,
        Expr::ident("OK"),
    );

    let return_stmt = Item::Return(Some(FnCall::method(Expr::ident("_reader"), "get_error")));

    Item::If(
        IfBuilder::default()
            .condition(condition)
            .then_body(vec![return_stmt].into())
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

    /* -------------------- Tests: gen_reader_error_check ------------------- */

    #[test]
    fn test_gen_reader_error_check() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // When: Generating a reader error check.
        let item = gen_reader_error_check();

        // When: The item is serialized to source code.
        cw.writeln(&mut s, "# Test context:").unwrap();
        item.emit(&mut cw, &mut s).unwrap();

        // Then: The output matches expected content exactly.
        let actual = s.into_content();

        let expected = r#"# Test context:
if _reader.get_error() != OK:
	return _reader.get_error()"#;

        assert_eq!(actual, expected);
    }

    /* ------------------ Tests: gen_decode_primitive ------------------- */

    #[test]
    fn test_gen_decode_primitive_bool() {
        // Given: A bool encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 1 },
            native: NativeType::Bool,
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("active", &encoding).unwrap();

        // Then: Two statements are generated (assignment + error check).
        assert_eq!(stmts.len(), 2);

        // Then: First statement is assignment.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "active = _reader.read_bool()");

        // Then: Second statement is error check.
        let mut s2 = StringWriter::default();
        stmts[1].emit(&mut cw, &mut s2).unwrap();
        let actual = s2.into_content();

        let expected = r#"if _reader.get_error() != OK:
	return _reader.get_error()"#;

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_gen_decode_primitive_i32() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("score", &encoding).unwrap();

        // Then: Two statements are generated.
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads i32.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "score = _reader.read_i32()");
    }

    #[test]
    fn test_gen_decode_primitive_i8() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("value", &encoding).unwrap();

        // Then: Two statements are generated.
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads i8.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "value = _reader.read_i8()");
    }

    #[test]
    fn test_gen_decode_primitive_i16() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("value", &encoding).unwrap();

        // Then: Two statements are generated.
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads i16.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "value = _reader.read_i16()");
    }

    #[test]
    fn test_gen_decode_primitive_i64() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("value", &encoding).unwrap();

        // Then: Two statements are generated.
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads i64.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "value = _reader.read_i64()");
    }

    #[test]
    fn test_gen_decode_primitive_u8() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("value", &encoding).unwrap();

        // Then: Two statements are generated.
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads u8.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "value = _reader.read_u8()");
    }

    #[test]
    fn test_gen_decode_primitive_u16() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("value", &encoding).unwrap();

        // Then: Two statements are generated.
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads u16.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "value = _reader.read_u16()");
    }

    #[test]
    fn test_gen_decode_primitive_u32() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("value", &encoding).unwrap();

        // Then: Two statements are generated.
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads u32.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "value = _reader.read_u32()");
    }

    #[test]
    fn test_gen_decode_primitive_u64() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("value", &encoding).unwrap();

        // Then: Two statements are generated.
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads u64.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "value = _reader.read_u64()");
    }

    #[test]
    fn test_gen_decode_primitive_f32() {
        // Given: An f32 encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 32 },
            native: NativeType::Float { bits: 32 },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("value", &encoding).unwrap();

        // Then: Two statements are generated.
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads f32.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "value = _reader.read_f32()");
    }

    #[test]
    fn test_gen_decode_primitive_f64() {
        // Given: An f64 encoding.
        let encoding = Encoding {
            wire: WireFormat::Bits { count: 64 },
            native: NativeType::Float { bits: 64 },
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("value", &encoding).unwrap();

        // Then: Two statements are generated.
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads f64.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "value = _reader.read_f64()");
    }

    #[test]
    fn test_gen_decode_primitive_string() {
        // Given: A string encoding.
        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
            native: NativeType::String,
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("name", &encoding).unwrap();

        // Then: Two statements are generated.
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads string.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "name = _reader.read_string()");
    }

    #[test]
    fn test_gen_decode_primitive_varint_signed() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("value", &encoding).unwrap();

        // Then: Two statements are generated.
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads signed varint.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "value = _reader.read_varint_signed()");
    }

    #[test]
    fn test_gen_decode_primitive_varint_unsigned() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("value", &encoding).unwrap();

        // Then: Two statements are generated.
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads unsigned varint.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "value = _reader.read_varint_unsigned()");
    }

    #[test]
    fn test_gen_decode_primitive_zigzag() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("value", &encoding).unwrap();

        // Then: Two statements are generated.
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads zigzag with bit count.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        assert_eq!(s.into_content(), "value = _reader.read_zigzag(32)");
    }

    /* -------------------- Tests: gen_decode_bytes --------------------- */

    #[test]
    fn test_gen_decode_bytes() {
        // Given: A bytes encoding.
        let encoding = Encoding {
            wire: WireFormat::LengthPrefixed { prefix_bits: 64 },
            native: NativeType::Bytes,
            transforms: vec![],
            padding_bits: None,
        };

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("data", &encoding).unwrap();

        // Then: Two statements are generated (assignment + error check).
        assert_eq!(stmts.len(), 2);

        // Then: First statement reads bytes with nested call.
        let mut s = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s).unwrap();
        let actual = s.into_content();
        assert_eq!(
            actual,
            "data = _reader.read_bytes(_reader.read_varint_unsigned())"
        );

        // Then: Second statement is error check.
        let mut s2 = StringWriter::default();
        stmts[1].emit(&mut cw, &mut s2).unwrap();
        let actual2 = s2.into_content();

        let expected2 = r#"if _reader.get_error() != OK:
	return _reader.get_error()"#;

        assert_eq!(actual2, expected2);
    }

    /* -------------------- Tests: gen_decode_array --------------------- */

    #[test]
    fn test_gen_decode_array_primitive() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("items", &encoding).unwrap();

        // Then: Two statements are generated (init + for loop).
        assert_eq!(stmts.len(), 2);

        // Then: First statement initializes empty array.
        let mut s1 = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s1).unwrap();
        assert_eq!(s1.into_content(), "items = []");

        // Then: Second statement is for loop.
        let mut s2 = StringWriter::default();
        stmts[1].emit(&mut cw, &mut s2).unwrap();
        let actual = s2.into_content();

        let expected = r#"for _i in range(_reader.read_varint_unsigned()):
	_temp = _reader.read_i32()
	if _reader.get_error() != OK:
		return _reader.get_error()
	items.append(_temp)"#;

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_gen_decode_array_message() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("players", &encoding).unwrap();

        // Then: Two statements are generated (init + for loop).
        assert_eq!(stmts.len(), 2);

        // Then: First statement initializes empty array.
        let mut s1 = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s1).unwrap();
        assert_eq!(s1.into_content(), "players = []");

        // Then: Second statement is for loop with message construction.
        let mut s2 = StringWriter::default();
        stmts[1].emit(&mut cw, &mut s2).unwrap();
        let actual = s2.into_content();

        let expected = r#"for _i in range(_reader.read_varint_unsigned()):
	var _item := Player.new()
	_item._decode(_reader)
	if _reader.get_error() != OK:
		return _reader.get_error()
	players.append(_item)"#;

        assert_eq!(actual, expected);
    }

    /* --------------------- Tests: gen_decode_map ---------------------- */

    #[test]
    fn test_gen_decode_map_primitive() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("stats", &encoding).unwrap();

        // Then: Two statements are generated (init + for loop).
        assert_eq!(stmts.len(), 2);

        // Then: First statement initializes empty dict.
        let mut s1 = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s1).unwrap();
        assert_eq!(s1.into_content(), "stats = {}");

        // Then: Second statement is for loop.
        let mut s2 = StringWriter::default();
        stmts[1].emit(&mut cw, &mut s2).unwrap();
        let actual = s2.into_content();

        let expected = r#"for _i in range(_reader.read_varint_unsigned()):
	var _key := _reader.read_string()
	if _reader.get_error() != OK:
		return _reader.get_error()
	stats[_key] = _reader.read_i32()
	if _reader.get_error() != OK:
		return _reader.get_error()"#;

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_gen_decode_map_message() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("players", &encoding).unwrap();

        // Then: Two statements are generated (init + for loop).
        assert_eq!(stmts.len(), 2);

        // Then: First statement initializes empty dict.
        let mut s1 = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s1).unwrap();
        assert_eq!(s1.into_content(), "players = {}");

        // Then: Second statement is for loop with message construction.
        let mut s2 = StringWriter::default();
        stmts[1].emit(&mut cw, &mut s2).unwrap();
        let actual = s2.into_content();

        let expected = r#"for _i in range(_reader.read_varint_unsigned()):
	var _key := _reader.read_string()
	if _reader.get_error() != OK:
		return _reader.get_error()
	var _value := Player.new()
	_value._decode(_reader)
	if _reader.get_error() != OK:
		return _reader.get_error()
	players[_key] = _value"#;

        assert_eq!(actual, expected);
    }

    /* ------------------- Tests: gen_decode_message -------------------- */

    #[test]
    fn test_gen_decode_message() {
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

        // When: Generating decode statements.
        let stmts = gen_decode_stmts("player", &encoding).unwrap();

        // Then: Three statements are generated (assignment + decode + error check).
        assert_eq!(stmts.len(), 3);

        // Then: First statement creates instance.
        let mut s1 = StringWriter::default();
        let mut cw = GDScript::writer();
        stmts[0].emit(&mut cw, &mut s1).unwrap();
        assert_eq!(s1.into_content(), "player = Player.new()");

        // Then: Second statement calls decode.
        let mut s2 = StringWriter::default();
        stmts[1].emit(&mut cw, &mut s2).unwrap();
        assert_eq!(s2.into_content(), "player._decode(_reader)");

        // Then: Third statement is error check.
        let mut s3 = StringWriter::default();
        stmts[2].emit(&mut cw, &mut s3).unwrap();
        let actual = s3.into_content();

        let expected = r#"if _reader.get_error() != OK:
	return _reader.get_error()"#;

        assert_eq!(actual, expected);
    }
}
