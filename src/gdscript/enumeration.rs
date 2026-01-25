use baproto::{CodeWriter, Enum, StringWriter, Variant};

use crate::gdscript::ast::*;
use crate::gdscript::collect::TypeEntry;
use crate::gdscript::types::{collect_variant_dependencies, escape_keyword};

/* -------------------------------------------------------------------------- */
/*                              Fn: generate_enum                             */
/* -------------------------------------------------------------------------- */

/// `generate_enum` generates the GDScript code for an enum type.
///
/// Enums are represented as classes with integer constants, since GDScript
/// doesn't support standalone enums. The class is non-instantiable.
pub fn generate_enum(
    cw: &mut CodeWriter,
    enm: &Enum,
    entry: &TypeEntry,
    pkg: &[String],
) -> anyhow::Result<String> {
    let mut w = StringWriter::default();

    // Build sections.
    let mut sections = Vec::new();

    sections.push(gen_dependencies(&enm.variants, pkg, &entry.file_stem));

    // CONSTANTS section.
    if !enm.variants.is_empty() {
        let mut items = Vec::new();
        for variant in &enm.variants {
            match variant {
                Variant::Unit { name, index, doc } => {
                    let escaped_name = escape_keyword(name);
                    let mut builder = AssignmentBuilder::default();
                    builder
                        .declaration(DeclarationKind::Const)
                        .variable(escaped_name)
                        .type_hint(TypeHint::Explicit("int".to_string()))
                        .value(ValueKind::Raw(index.to_string()));
                    if let Some(d) = doc {
                        builder.comment(Some(Comment::from(d.as_str())));
                    }
                    items.push(builder.build()?.into());
                }
                Variant::Field {
                    name, index, doc, ..
                } => {
                    // For field variants, we still generate the discriminant constant.
                    // The associated data handling would be done separately.
                    let escaped_name = escape_keyword(name);
                    let mut builder = AssignmentBuilder::default();
                    builder
                        .declaration(DeclarationKind::Const)
                        .variable(escaped_name)
                        .type_hint(TypeHint::Explicit("int".to_string()))
                        .value(ValueKind::Raw(index.to_string()));
                    if let Some(d) = doc {
                        builder.comment(Some(Comment::from(d.as_str())));
                    }
                    items.push(builder.build()?.into());
                }
            }
        }

        sections.push(
            SectionBuilder::default()
                .header("CONSTANTS")
                .body(items)
                .build()?,
        );
    }

    // ENGINE METHODS section (non-instantiable).
    let enum_name = enm.name().unwrap_or("Enum");
    let init_func = FnDefBuilder::default()
        .name("_init")
        .type_hint(TypeHint::Explicit("void".to_string()))
        .body(vec![
            FnCall::assert(
                Literal::from(false),
                format!("{} is non-instantiable", enum_name),
            )
            .into(),
        ])
        .build()?;

    sections.push(
        SectionBuilder::default()
            .header("ENGINE METHODS (OVERRIDES)")
            .body(vec![Item::FnDef(init_func)])
            .build()?,
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

/* -------------------------- Fn: gen_dependencies -------------------------- */

fn gen_dependencies(fields: &[Variant], pkg: &[String], name: &str) -> Section {
    let mut items = Vec::new();

    // Runtime dependencies.
    let path_runtime = "res://addons/baproto/runtime";
    items.push(Assignment::preload("_Reader", format!("{}/reader.gd", path_runtime)).into());
    items.push(Assignment::preload("_Writer", format!("{}/writer.gd", path_runtime)).into());

    // Field dependencies.
    let deps = collect_variant_dependencies(fields, pkg, name);

    for (name, _, path) in &deps {
        items.push(Assignment::preload(name, path).into());
    }

    SectionBuilder::default()
        .header("DEPENDENCIES")
        .body(items)
        .build()
        .unwrap()
}
