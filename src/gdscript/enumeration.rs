use baproto::{CodeWriter, Enum, StringWriter, Variant};

use crate::gdscript::ast::*;
use crate::gdscript::codec::{gen_enum_decode_stmts, gen_enum_encode_stmts};
use crate::gdscript::collect::TypeEntry;
use crate::gdscript::types::{
    collect_variant_dependencies, default_value, escape_keyword, gen_dependencies_section,
    type_name,
};

/* -------------------------------------------------------------------------- */
/*                              Fn: generate_enum                             */
/* -------------------------------------------------------------------------- */

/// `generate_enum` generates the GDScript code for an enum type.
///
/// Enums are represented as discriminated unions with serialization support.
pub fn generate_enum(
    cw: &mut CodeWriter,
    enm: &Enum,
    entry: &TypeEntry,
    pkg: &[String],
) -> anyhow::Result<String> {
    let mut w = StringWriter::default();

    let mut sections = Vec::new();

    // Dependencies
    sections.push(gen_dependencies(&enm.variants, pkg, &entry.file_stem));

    // Discriminants (GDScript enum)
    if !enm.variants.is_empty() {
        sections.push(gen_enum_decl(&enm.variants)?);
    }

    // Fields
    sections.push(gen_fields());

    // Public methods
    let mut public_methods = Vec::new();
    public_methods.extend(gen_discriminant_methods());
    public_methods.extend(gen_accessor_methods(&enm.variants));
    public_methods.extend(gen_serialization_methods());

    sections.push(
        SectionBuilder::default()
            .header("PUBLIC METHODS")
            .body(
                public_methods
                    .into_iter()
                    .map(Item::FnDef)
                    .collect::<Vec<_>>(),
            )
            .build()
            .unwrap(),
    );

    // Private methods
    let private_methods = gen_private_methods(&enm.variants)?;
    sections.push(
        SectionBuilder::default()
            .header("PRIVATE METHODS")
            .body(
                private_methods
                    .into_iter()
                    .map(Item::FnDef)
                    .collect::<Vec<_>>(),
            )
            .build()
            .unwrap(),
    );

    // Engine methods
    sections.push(gen_engine_methods());

    // Debugging
    let to_string_method = gen_to_string_method(&enm.variants);
    sections.push(
        SectionBuilder::default()
            .header("DEBUGGING")
            .body(vec![Item::FnDef(to_string_method)])
            .build()
            .unwrap(),
    );

    let script = ScriptBuilder::default()
        .header(Comment::do_not_edit())
        .comment(enm.doc.as_ref().map(Comment::from))
        .extends("RefCounted".to_string())
        .sections(sections)
        .build()
        .unwrap();

    script.emit(cw, &mut w)?;

    Ok(w.into_content())
}

/* ------------------------- Fn: gen_enum_decl -------------------------- */

fn gen_enum_decl(variants: &[Variant]) -> anyhow::Result<Section> {
    let mut enum_variants = vec![("NONE".to_string(), -1)];

    for variant in variants {
        match variant {
            Variant::Unit { name, index, .. } | Variant::Field { name, index, .. } => {
                let escaped_name = escape_keyword(name);
                enum_variants.push((escaped_name, *index as i64));
            }
        }
    }

    let enum_decl = EnumDeclBuilder::default().variants(enum_variants).build()?;

    Ok(SectionBuilder::default()
        .header("DISCRIMINANTS")
        .body(vec![enum_decl.into()])
        .build()?)
}

/* ---------------------------- Fn: gen_fields --------------------------- */

fn gen_fields() -> Section {
    let discriminant_field = AssignmentBuilder::default()
        .declaration(DeclarationKind::Var)
        .variable("_discriminant")
        .type_hint(TypeHint::Explicit("int".to_string()))
        .value(ValueKind::Expr(Expr::ident("NONE")))
        .build()
        .unwrap();

    let value_field = AssignmentBuilder::default()
        .declaration(DeclarationKind::Var)
        .variable("_value")
        .type_hint(TypeHint::Explicit("Variant".to_string()))
        .value(ValueKind::Expr(Expr::null()))
        .build()
        .unwrap();

    SectionBuilder::default()
        .header("INITIALIZATION")
        .body(vec![discriminant_field.into(), value_field.into()])
        .build()
        .unwrap()
}

/* --------------------- Fn: gen_discriminant_methods ---------------------- */

fn gen_discriminant_methods() -> Vec<FnDef> {
    let mut methods = Vec::new();

    // which() -> int
    let which_func = FnDefBuilder::default()
        .name("which")
        .comment("`which` returns the current discriminant.")
        .type_hint(TypeHint::Explicit("int".to_string()))
        .body(vec![Item::Return(Expr::ident("_discriminant"))])
        .build()
        .unwrap();
    methods.push(which_func);

    // is_none() -> bool
    let is_none_func = FnDefBuilder::default()
        .name("is_none")
        .comment("`is_none` checks if the enum is unset.")
        .type_hint(TypeHint::Explicit("bool".to_string()))
        .body(vec![Item::Return(Expr::binary_op(
            Expr::ident("_discriminant"),
            Operator::Eq,
            Expr::ident("NONE"),
        ))])
        .build()
        .unwrap();
    methods.push(is_none_func);

    // clear() -> void
    let clear_func = FnDefBuilder::default()
        .name("clear")
        .comment("`clear` sets the enum to NONE.")
        .type_hint(TypeHint::Explicit("void".to_string()))
        .body(vec![
            Assignment::reassign("_discriminant", Expr::ident("NONE")).into(),
            Assignment::reassign("_value", Expr::null()).into(),
        ])
        .build()
        .unwrap();
    methods.push(clear_func);

    methods
}

/* ----------------------- Fn: gen_accessor_methods ------------------------ */

fn gen_accessor_methods(variants: &[Variant]) -> Vec<FnDef> {
    let mut methods = Vec::new();

    for variant in variants {
        match variant {
            Variant::Unit { name, .. } => {
                let snake_name = name.to_lowercase();
                let variant_const = escape_keyword(name);

                // has_xxx() -> bool
                let has_func = FnDefBuilder::default()
                    .name(format!("has_{}", snake_name))
                    .type_hint(TypeHint::Explicit("bool".to_string()))
                    .body(vec![Item::Return(Expr::binary_op(
                        Expr::ident("_discriminant"),
                        Operator::Eq,
                        Expr::ident(&variant_const),
                    ))])
                    .build()
                    .unwrap();
                methods.push(has_func);

                // set_xxx() -> void
                let set_func = FnDefBuilder::default()
                    .name(format!("set_{}", snake_name))
                    .type_hint(TypeHint::Explicit("void".to_string()))
                    .body(vec![
                        Assignment::reassign("_discriminant", Expr::ident(&variant_const)).into(),
                        Assignment::reassign("_value", Expr::null()).into(),
                    ])
                    .build()
                    .unwrap();
                methods.push(set_func);

                // clear_xxx() -> void
                let clear_func = FnDefBuilder::default()
                    .name(format!("clear_{}", snake_name))
                    .type_hint(TypeHint::Explicit("void".to_string()))
                    .body(vec![
                        IfBuilder::default()
                            .condition(Expr::binary_op(
                                Expr::ident("_discriminant"),
                                Operator::Eq,
                                Expr::ident(&variant_const),
                            ))
                            .then_body(Block::from(vec![
                                Assignment::reassign("_discriminant", Expr::ident("NONE")).into(),
                                Assignment::reassign("_value", Expr::null()).into(),
                            ]))
                            .build()
                            .unwrap()
                            .into(),
                    ])
                    .build()
                    .unwrap();
                methods.push(clear_func);
            }
            Variant::Field { name, field, .. } => {
                let snake_name = name.to_lowercase();
                let variant_const = escape_keyword(name);
                let type_str = type_name(&field.encoding.native);
                let default_val = default_value(&field.encoding.native);

                // has_xxx() -> bool
                let has_func = FnDefBuilder::default()
                    .name(format!("has_{}", snake_name))
                    .type_hint(TypeHint::Explicit("bool".to_string()))
                    .body(vec![Item::Return(Expr::binary_op(
                        Expr::ident("_discriminant"),
                        Operator::Eq,
                        Expr::ident(&variant_const),
                    ))])
                    .build()
                    .unwrap();
                methods.push(has_func);

                // get_xxx() -> Type
                let get_func = FnDefBuilder::default()
                    .name(format!("get_{}", snake_name))
                    .type_hint(TypeHint::Explicit(type_str.clone()))
                    .body(vec![
                        IfBuilder::default()
                            .condition(Expr::binary_op(
                                Expr::ident("_discriminant"),
                                Operator::Eq,
                                Expr::ident(&variant_const),
                            ))
                            .then_body(Block::from(vec![Item::Return(Expr::ident("_value"))]))
                            .build()
                            .unwrap()
                            .into(),
                        Item::Return(default_val),
                    ])
                    .build()
                    .unwrap();
                methods.push(get_func);

                // set_xxx(value: Type) -> void
                let set_func = FnDefBuilder::default()
                    .name(format!("set_{}", snake_name))
                    .params(vec![Assignment::param("value", &type_str)])
                    .type_hint(TypeHint::Explicit("void".to_string()))
                    .body(vec![
                        Assignment::reassign("_discriminant", Expr::ident(&variant_const)).into(),
                        Assignment::reassign("_value", Expr::ident("value")).into(),
                    ])
                    .build()
                    .unwrap();
                methods.push(set_func);

                // clear_xxx() -> void
                let clear_func = FnDefBuilder::default()
                    .name(format!("clear_{}", snake_name))
                    .type_hint(TypeHint::Explicit("void".to_string()))
                    .body(vec![
                        IfBuilder::default()
                            .condition(Expr::binary_op(
                                Expr::ident("_discriminant"),
                                Operator::Eq,
                                Expr::ident(&variant_const),
                            ))
                            .then_body(Block::from(vec![
                                Assignment::reassign("_discriminant", Expr::ident("NONE")).into(),
                                Assignment::reassign("_value", Expr::null()).into(),
                            ]))
                            .build()
                            .unwrap()
                            .into(),
                    ])
                    .build()
                    .unwrap();
                methods.push(clear_func);
            }
        }
    }

    methods
}

/* -------------------------- Fn: gen_dependencies -------------------------- */

fn gen_dependencies(variants: &[Variant], pkg: &[String], name: &str) -> Section {
    let deps = collect_variant_dependencies(variants, pkg, name);
    gen_dependencies_section(deps)
}

/* -------------------- Fn: gen_serialization_methods ---------------------- */

fn gen_serialization_methods() -> Vec<FnDef> {
    let mut methods = Vec::new();

    // serialize(out: PackedByteArray) -> Error
    let serialize_func = FnDefBuilder::default()
        .name("serialize")
        .comment("`serialize` writes this enum to a `PackedByteArray`.")
        .params(vec![Assignment::param("out", "PackedByteArray")])
        .type_hint(TypeHint::Explicit("Error".to_string()))
        .body(vec![
            Assignment::var("_writer", FnCall::method(Expr::ident("_Writer"), "new")).into(),
            FnCall::method_args(Expr::ident("self"), "_encode", vec![Expr::ident("_writer")])
                .into(),
            FnCall::method_args(
                Expr::ident("out"),
                "append_array",
                vec![FnCall::method(Expr::ident("_writer"), "to_bytes")],
            )
            .into(),
            Item::Return(FnCall::method(Expr::ident("_writer"), "get_error")),
        ])
        .build()
        .unwrap();
    methods.push(serialize_func);

    // deserialize(data: PackedByteArray) -> Error
    let deserialize_func = FnDefBuilder::default()
        .name("deserialize")
        .comment("`deserialize` reads this enum from a `PackedByteArray`.")
        .params(vec![Assignment::param("data", "PackedByteArray")])
        .type_hint(TypeHint::Explicit("Error".to_string()))
        .body(vec![
            Assignment::var(
                "_reader",
                FnCall::method_args(Expr::ident("_Reader"), "new", vec![Expr::ident("data")]),
            )
            .into(),
            FnCall::method_args(Expr::ident("self"), "_decode", vec![Expr::ident("_reader")])
                .into(),
            Item::Return(FnCall::method(Expr::ident("_reader"), "get_error")),
        ])
        .build()
        .unwrap();
    methods.push(deserialize_func);

    methods
}

/* ----------------------- Fn: gen_private_methods ------------------------- */

fn gen_private_methods(variants: &[Variant]) -> anyhow::Result<Vec<FnDef>> {
    let mut methods = Vec::new();

    // _encode(_writer: _Writer) -> void
    let encode_body = gen_enum_encode_stmts(variants)?;
    let encode_func = FnDefBuilder::default()
        .name("_encode")
        .comment("`_encode` serializes the enum to the writer.")
        .params(vec![Assignment::param("_writer", "_Writer")])
        .type_hint(TypeHint::Explicit("void".to_string()))
        .body(encode_body)
        .build()?;
    methods.push(encode_func);

    // _decode(_reader: _Reader) -> Error
    let decode_body = gen_enum_decode_stmts(variants)?;
    let decode_func = FnDefBuilder::default()
        .name("_decode")
        .comment("`_decode` deserializes the enum from the reader.")
        .params(vec![Assignment::param("_reader", "_Reader")])
        .type_hint(TypeHint::Explicit("Error".to_string()))
        .body(decode_body)
        .build()?;
    methods.push(decode_func);

    Ok(methods)
}

/* ----------------------- Fn: gen_to_string_method ------------------------ */

fn gen_to_string_method(variants: &[Variant]) -> FnDef {
    let mut match_arms = Vec::new();

    // NONE case
    match_arms.push(MatchArm {
        pattern: Expr::ident("NONE"),
        body: Block::from(vec![Item::Return(Expr::from("\"<NONE>\""))]),
    });

    // Variant cases
    for variant in variants {
        match variant {
            Variant::Unit { name, .. } => {
                let variant_const = escape_keyword(name);
                match_arms.push(MatchArm {
                    pattern: Expr::ident(&variant_const),
                    body: Block::from(vec![Item::Return(Expr::from(format!("\"{}\"", name)))]),
                });
            }
            Variant::Field { name, .. } => {
                let variant_const = escape_keyword(name);
                match_arms.push(MatchArm {
                    pattern: Expr::ident(&variant_const),
                    body: Block::from(vec![Item::Return(Expr::binary_op(
                        Expr::from(format!("\"{}(\"", name)),
                        Operator::Add,
                        Expr::binary_op(
                            FnCall::function_args("str", vec![Expr::ident("_value")]),
                            Operator::Add,
                            Expr::from("\")\""),
                        ),
                    ))]),
                });
            }
        }
    }

    let match_stmt = Match {
        scrutinee: Expr::ident("_discriminant"),
        arms: match_arms,
    };

    FnDefBuilder::default()
        .name("_to_string")
        .type_hint(TypeHint::Explicit("String".to_string()))
        .body(vec![match_stmt.into()])
        .build()
        .unwrap()
}

/* ------------------------ Fn: gen_engine_methods ------------------------ */

fn gen_engine_methods() -> Section {
    let init_func = FnDefBuilder::default()
        .name("_init")
        .type_hint(TypeHint::Explicit("void".to_string()))
        .body(vec![
            Assignment::reassign("_discriminant", Expr::ident("NONE")).into(),
            Assignment::reassign("_value", Expr::null()).into(),
        ])
        .build()
        .unwrap();

    SectionBuilder::default()
        .header("ENGINE METHODS (OVERRIDES)")
        .body(vec![Item::FnDef(init_func)])
        .build()
        .unwrap()
}
