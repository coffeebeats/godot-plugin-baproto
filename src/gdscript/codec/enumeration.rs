use baproto::Variant;

use crate::gdscript::ast::{
    Assignment, Block, Expr, FnCall, IfBuilder, Item, Match, MatchArm, Operator,
};

use super::wire::get_write_method;

/* -------------------------------------------------------------------------- */
/*                          Fn: gen_enum_encode_stmts                         */
/* -------------------------------------------------------------------------- */

/// `gen_enum_encode_stmts` generates encoding statements for an enum.
///
/// # Generated GDScript
/// ```gdscript
/// if _discriminant == NONE:
///     _writer.set_error(ERR_INVALID_DATA)
///     return null
/// _writer.write_varint_signed(_discriminant)
/// match _discriminant:
///     UNIT_VARIANT:
///         pass
///     FIELD_VARIANT:
///         _writer.write_xxx(_value)
/// ```
pub fn gen_enum_encode_stmts(variants: &[Variant]) -> anyhow::Result<Vec<Item>> {
    let mut stmts = Vec::new();

    // Validate discriminant is not NONE
    let none_check = IfBuilder::default()
        .condition(Expr::binary_op(
            Expr::ident("_discriminant"),
            Operator::Eq,
            Expr::ident("NONE"),
        ))
        .then_body(Block::from(vec![
            FnCall::method_args(
                Expr::ident("_writer"),
                "set_error",
                vec![Expr::ident("ERR_INVALID_DATA")],
            )
            .into(),
            Item::Return(Expr::null()),
        ]))
        .build()?;

    stmts.push(none_check.into());

    // Write discriminant
    let write_discriminant = FnCall::method_args(
        Expr::ident("_writer"),
        "write_varint_signed",
        vec![Expr::ident("_discriminant")],
    );
    stmts.push(write_discriminant.into());

    // Match on discriminant to write value for field variants
    if !variants.is_empty() {
        let mut match_arms = Vec::new();

        for variant in variants {
            match variant {
                Variant::Unit { name, .. } => {
                    // Unit variants: just pass
                    match_arms.push(MatchArm {
                        pattern: Expr::ident(name),
                        body: Block::default(),
                    });
                }
                Variant::Field { name, field, .. } => {
                    // Field variants: write the value
                    let method = get_write_method(&field.encoding)?;
                    let mut args = vec![Expr::ident("_value")];
                    args.extend(method.extra_args);

                    let write_value =
                        FnCall::method_args(Expr::ident("_writer"), &method.method, args);

                    match_arms.push(MatchArm {
                        pattern: Expr::ident(name),
                        body: Block::from(vec![write_value.into()]),
                    });
                }
            }
        }

        let match_stmt = Match {
            scrutinee: Expr::ident("_discriminant"),
            arms: match_arms,
        };

        stmts.push(match_stmt.into());
    }

    Ok(stmts)
}

/* -------------------------------------------------------------------------- */
/*                          Fn: gen_enum_decode_stmts                         */
/* -------------------------------------------------------------------------- */

/// `gen_enum_decode_stmts` generates decoding statements for an enum.
///
/// # Generated GDScript
/// ```gdscript
/// _discriminant = _reader.read_varint_signed()
/// if _reader.get_error() != OK:
///     return _reader.get_error()
/// match _discriminant:
///     NONE:
///         _reader.set_error(ERR_INVALID_DATA)
///         return _reader.get_error()
///     UNIT_VARIANT:
///         _value = null
///     FIELD_VARIANT:
///         _value = _reader.read_xxx()
///         if _reader.get_error() != OK:
///             return _reader.get_error()
/// return _reader.get_error()
/// ```
pub fn gen_enum_decode_stmts(variants: &[Variant]) -> anyhow::Result<Vec<Item>> {
    let mut stmts = Vec::new();

    // Read discriminant
    let read_discriminant = Assignment::reassign(
        "_discriminant",
        FnCall::method(Expr::ident("_reader"), "read_varint_signed"),
    );
    stmts.push(read_discriminant.into());

    // Check for read error
    let error_check = IfBuilder::default()
        .condition(Expr::binary_op(
            FnCall::method(Expr::ident("_reader"), "get_error"),
            Operator::NotEq,
            Expr::ident("OK"),
        ))
        .then_body(Block::from(vec![Item::Return(FnCall::method(
            Expr::ident("_reader"),
            "get_error",
        ))]))
        .build()?;

    stmts.push(error_check.into());

    // Match on discriminant
    let mut match_arms = Vec::new();

    // NONE case - reject with error
    match_arms.push(MatchArm {
        pattern: Expr::ident("NONE"),
        body: Block::from(vec![
            FnCall::method_args(
                Expr::ident("_reader"),
                "set_error",
                vec![Expr::ident("ERR_INVALID_DATA")],
            )
            .into(),
            Item::Return(FnCall::method(Expr::ident("_reader"), "get_error")),
        ]),
    });

    // Variant cases
    for variant in variants {
        match variant {
            Variant::Unit { name, .. } => {
                // Unit variants: set _value to null
                match_arms.push(MatchArm {
                    pattern: Expr::ident(name),
                    body: Block::from(vec![Assignment::reassign("_value", Expr::null()).into()]),
                });
            }
            Variant::Field { name, field, .. } => {
                // Field variants: read the value
                let method = get_write_method(&field.encoding)?;
                let read_method = method.method.replace("write", "read");

                let read_args = method.extra_args.clone();
                let read_call = if read_args.is_empty() {
                    FnCall::method(Expr::ident("_reader"), read_method.clone())
                } else {
                    FnCall::method_args(Expr::ident("_reader"), read_method.clone(), read_args)
                };

                let assign_value = Assignment::reassign("_value", read_call);

                let error_check = IfBuilder::default()
                    .condition(Expr::binary_op(
                        FnCall::method(Expr::ident("_reader"), "get_error"),
                        Operator::NotEq,
                        Expr::ident("OK"),
                    ))
                    .then_body(Block::from(vec![Item::Return(FnCall::method(
                        Expr::ident("_reader"),
                        "get_error",
                    ))]))
                    .build()?;

                match_arms.push(MatchArm {
                    pattern: Expr::ident(name),
                    body: Block::from(vec![assign_value.into(), error_check.into()]),
                });
            }
        }
    }

    let match_stmt = Match {
        scrutinee: Expr::ident("_discriminant"),
        arms: match_arms,
    };

    stmts.push(match_stmt.into());

    // Final return
    stmts.push(Item::Return(FnCall::method(
        Expr::ident("_reader"),
        "get_error",
    )));

    Ok(stmts)
}
