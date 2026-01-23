use baproto::{CodeWriter, Message, StringWriter};

use crate::gdscript::ast::{Emit, FuncDeclBuilder, GDFileBuilder, Item, SectionBuilder, Stmt};
use crate::gdscript::codec;
use crate::gdscript::collect::TypeEntry;
use crate::gdscript::types::{
    collect_field_dependencies, default_value, escape_keyword, type_name,
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

    // Collect dependencies (external message/enum types used in fields).
    let deps = collect_field_dependencies(&msg.fields, pkg, &entry.file_stem);

    // Build sections.
    let mut sections = Vec::new();

    // DEPENDENCIES section - runtime + field dependencies.
    let mut dep_items = Vec::new();

    // Runtime dependencies.
    let runtime_path = "res://addons/baproto/runtime";
    dep_items.push(Item::Stmt(Stmt::Preload {
        name: "_Writer".to_string(),
        path: format!("{}/writer.gd", runtime_path),
    }));
    dep_items.push(Item::Stmt(Stmt::Preload {
        name: "_Reader".to_string(),
        path: format!("{}/reader.gd", runtime_path),
    }));

    // Field dependencies.
    if !deps.is_empty() {
        dep_items.push(Item::Stmt(Stmt::Blank));
        for (const_name, _file_stem, path) in &deps {
            dep_items.push(Item::Stmt(Stmt::Preload {
                name: const_name.clone(),
                path: path.clone(),
            }));
        }
    }

    sections.push(
        SectionBuilder::default()
            .name("DEPENDENCIES")
            .body(dep_items)
            .build()?,
    );

    // NESTED TYPES section (if any).
    if !entry.nested.is_empty() {
        let mut nested_items = Vec::new();
        for nested_stem in &entry.nested {
            // Extract the simple name from the nested file stem.
            let simple_name = nested_stem
                .strip_prefix(&format!("{}_", entry.file_stem))
                .unwrap_or(nested_stem);
            nested_items.push(Item::Stmt(Stmt::Preload {
                name: simple_name.to_string(),
                path: format!("./{}.gd", nested_stem.to_lowercase()),
            }));
        }

        sections.push(
            SectionBuilder::default()
                .name("NESTED TYPES")
                .body(nested_items)
                .build()?,
        );
    }

    // FIELDS section.
    if !msg.fields.is_empty() {
        let mut field_items = Vec::new();
        for field in &msg.fields {
            let name = escape_keyword(&field.name);
            let type_str = type_name(&field.encoding.native);
            let default = default_value(&field.encoding.native);

            field_items.push(Item::Stmt(Stmt::Var {
                name,
                type_hint: Some(type_str),
                value: Some(default),
                doc: field.doc.clone(),
            }));
        }

        sections.push(
            SectionBuilder::default()
                .name("FIELDS")
                .body(field_items)
                .build()?,
        );
    }

    // PUBLIC METHODS section.
    let serialize_func = FuncDeclBuilder::default()
        .name("serialize")
        .params(vec![
            crate::gdscript::ast::FuncParamBuilder::default()
                .name("out")
                .type_hint("PackedByteArray")
                .build()?,
        ])
        .return_type("Error")
        .doc("`serialize` writes this message to a `PackedByteArray`.")
        .body(vec![
            Stmt::Assign {
                target: "_writer".to_string(),
                value: "_Writer.new()".to_string(),
            },
            Stmt::Expr("_encode(_writer)".to_string()),
            Stmt::Expr("out.append_array(_writer.to_bytes())".to_string()),
            Stmt::Return(Some("_writer.get_error()".to_string())),
        ])
        .build()?;

    let deserialize_func = FuncDeclBuilder::default()
        .name("deserialize")
        .params(vec![
            crate::gdscript::ast::FuncParamBuilder::default()
                .name("data")
                .type_hint("PackedByteArray")
                .build()?,
        ])
        .return_type("Error")
        .doc("`deserialize` reads this message from a `PackedByteArray`.")
        .body(vec![
            Stmt::Assign {
                target: "_reader".to_string(),
                value: "_Reader.new(data)".to_string(),
            },
            Stmt::Expr("_decode(_reader)".to_string()),
            Stmt::Return(Some("_reader.get_error()".to_string())),
        ])
        .build()?;

    sections.push(
        SectionBuilder::default()
            .name("PUBLIC METHODS")
            .body(vec![
                Item::Func(serialize_func),
                Item::Func(deserialize_func),
            ])
            .build()?,
    );

    // PRIVATE METHODS section.
    let encode_body = if msg.fields.is_empty() {
        vec![Stmt::Pass]
    } else {
        let mut stmts = Vec::new();
        for field in &msg.fields {
            let field_name = escape_keyword(&field.name);
            stmts.extend(codec::gen_encode_stmts(&field_name, &field.encoding)?);
        }
        stmts
    };

    let encode_func = FuncDeclBuilder::default()
        .name("_encode")
        .params(vec![
            crate::gdscript::ast::FuncParamBuilder::default()
                .name("_writer")
                .type_hint("_Writer")
                .build()?,
        ])
        .return_type("void")
        .doc("`_encode` serializes fields to the writer.")
        .body(encode_body)
        .build()?;

    let decode_body = if msg.fields.is_empty() {
        vec![Stmt::Pass]
    } else {
        let mut stmts = Vec::new();
        for field in &msg.fields {
            let field_name = escape_keyword(&field.name);
            stmts.extend(codec::gen_decode_stmts(&field_name, &field.encoding)?);
        }
        stmts
    };

    let decode_func = FuncDeclBuilder::default()
        .name("_decode")
        .params(vec![
            crate::gdscript::ast::FuncParamBuilder::default()
                .name("_reader")
                .type_hint("_Reader")
                .build()?,
        ])
        .return_type("void")
        .doc("`_decode` deserializes fields from the reader.")
        .body(decode_body)
        .build()?;

    sections.push(
        SectionBuilder::default()
            .name("PRIVATE METHODS")
            .body(vec![Item::Func(encode_func), Item::Func(decode_func)])
            .build()?,
    );

    // Build the GDScript file.
    let mut builder = GDFileBuilder::default();
    builder
        .header_comment("DO NOT EDIT: Generated by baproto-gdscript")
        .extends("RefCounted")
        .sections(sections);

    if let Some(doc) = &msg.doc {
        builder.doc(doc.clone());
    }

    let file = builder.build()?;

    // Emit the file.
    file.emit(cw, &mut w)?;

    Ok(w.into_content())
}

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    // Note: Full integration tests that require constructing Message/Enum with
    // Descriptors are in `mod.rs` where we use the `Generator::generate`
    // method. Unit tests for encode/decode logic are in `codec.rs` and AST
    // emission tests are in `ast/mod.rs`.
}
