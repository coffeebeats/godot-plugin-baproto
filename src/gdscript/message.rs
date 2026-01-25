use baproto::Field;
use baproto::{CodeWriter, Message, StringWriter};

use crate::gdscript::ast::*;
use crate::gdscript::codec;
use crate::gdscript::collect::TypeEntry;
use crate::gdscript::types::{
    collect_field_dependencies, default_value, escape_keyword, gen_dependencies_section, type_name,
};

/* -------------------------------------------------------------------------- */
/*                            Fn: generate_message                            */
/* -------------------------------------------------------------------------- */

/// `generate_message` generates the GDScript code for a message type.
pub fn generate_message(
    cw: &mut CodeWriter,
    msg: &Message,
    entry: &TypeEntry,
    pkg: &[String],
) -> anyhow::Result<String> {
    let mut w = StringWriter::default();

    let mut sections = Vec::new();

    sections.push(gen_dependencies(&msg.fields, pkg, &entry.file_stem));
    sections.push(gen_types(entry));

    if !msg.fields.is_empty() {
        sections.push(gen_fields(&msg.fields));
    }

    sections.push(gen_public_methods());
    sections.push(gen_private_methods(&msg.fields)?);

    let script = ScriptBuilder::default()
        .header(Comment::do_not_edit())
        .comment(msg.doc.as_ref().map(Comment::from))
        .extends("RefCounted".to_string())
        .sections(sections)
        .build()
        .unwrap();

    script.emit(cw, &mut w)?;

    Ok(w.into_content())
}

/* -------------------------- Fn: gen_dependencies -------------------------- */

fn gen_dependencies(fields: &[Field], pkg: &[String], name: &str) -> Section {
    let deps = collect_field_dependencies(fields, pkg, name);
    gen_dependencies_section(deps)
}

/* ----------------------------- Fn: gen_fields ----------------------------- */

fn gen_fields(fields: &[Field]) -> Section {
    let mut items = Vec::new();

    for field in fields {
        let type_str = type_name(&field.encoding.native);
        let default_value = default_value(&field.encoding.native);

        items.push(
            AssignmentBuilder::default()
                .comment(field.doc.as_ref().map(Comment::from))
                .declaration(DeclarationKind::Var)
                .variable(escape_keyword(&field.name))
                .type_hint(TypeHint::Explicit(type_str))
                .value(ValueKind::Expr(default_value))
                .build()
                .unwrap()
                .into(),
        );
    }

    SectionBuilder::default()
        .header("INITIALIZATION")
        .body(items)
        .build()
        .unwrap()
}

/* ------------------------- Fn: gen_public_methods ------------------------- */

fn gen_public_methods() -> Section {
    let serialize = FnDefBuilder::default()
        .comment("`serialize` writes this message to a `PackedByteArray`.")
        .name("serialize")
        .params(vec![Assignment::param("out", "PackedByteArray")])
        .type_hint(TypeHint::Explicit("Error".to_string()))
        .body(vec![
            Assignment::var("_writer", FnCall::method("_Writer", "new")).into(),
            FnCall::function_args("_encode", vec!["_writer"]).into(),
            FnCall::method_args(
                "out",
                "append_array",
                vec![FnCall::method("_writer", "to_bytes")],
            )
            .into(),
        ])
        .return_value(FnCall::method("_writer", "get_error"))
        .build()
        .unwrap();

    let deserialize = FnDefBuilder::default()
        .comment("`deserialize` reads this message from a `PackedByteArray`.")
        .name("deserialize")
        .params(vec![Assignment::param("data", "PackedByteArray")])
        .type_hint(TypeHint::Explicit("Error".to_string()))
        .body(vec![
            Assignment::var(
                "_reader",
                FnCall::method_args("_Reader", "new", vec!["data"]),
            )
            .into(),
            FnCall::function_args("_decode", vec!["_reader"]).into(),
        ])
        .return_value(FnCall::method("_reader", "get_error"))
        .build()
        .unwrap();

    SectionBuilder::default()
        .header("PUBLIC METHODS")
        .body(vec![Item::FnDef(serialize), Item::FnDef(deserialize)])
        .build()
        .unwrap()
}

/* ------------------------- Fn: gen_private_methods ------------------------ */

fn gen_private_methods(fields: &[Field]) -> anyhow::Result<Section> {
    let encode = FnDefBuilder::default()
        .comment("`_encode` serializes fields to the writer.")
        .name("_encode")
        .params(vec![Assignment::param("_writer", "_Writer")])
        .body(
            fields
                .iter()
                .try_fold(Vec::new(), |mut out, f| -> anyhow::Result<Vec<Item>> {
                    let field_name = escape_keyword(&f.name);
                    let stmts = codec::gen_encode_stmts(&field_name, &f.encoding)?;
                    out.extend(stmts);
                    Ok(out)
                })?,
        )
        .build()
        .unwrap();

    let decode = FnDefBuilder::default()
        .comment("`_decode` deserializes fields from the reader.")
        .name("_decode")
        .params(vec![Assignment::param("_reader", "_Reader")])
        .type_hint(TypeHint::Explicit("Error".to_string()))
        .body(
            fields
                .iter()
                .try_fold(Vec::new(), |mut out, f| -> anyhow::Result<Vec<Item>> {
                    let field_name = escape_keyword(&f.name);
                    let stmts = codec::gen_decode_stmts(&field_name, &f.encoding)?;
                    out.extend(stmts);
                    Ok(out)
                })?,
        )
        .return_value(FnCall::method("_reader", "get_error"))
        .build()
        .unwrap();

    Ok(SectionBuilder::default()
        .header("PRIVATE METHODS")
        .body(vec![Item::FnDef(encode), Item::FnDef(decode)])
        .build()
        .unwrap())
}

/* ------------------------------ Fn: gen_types ----------------------------- */

fn gen_types(entry: &TypeEntry) -> Section {
    let mut items = Vec::new();

    for name in &entry.nested {
        let simple_name = name
            .strip_prefix(&format!("{}_", entry.file_stem))
            .unwrap_or(name);

        items
            .push(Assignment::preload(simple_name, format!("./{}.gd", name.to_lowercase())).into());
    }

    SectionBuilder::default()
        .header("TYPES")
        .body(items)
        .build()
        .unwrap()
}
