use baproto::{CodeWriter, StringWriter};

use crate::gdscript::ast::*;
use crate::gdscript::collect::TypeEntry;

/* -------------------------------------------------------------------------- */
/*                           Fn: generate_namespace                           */
/* -------------------------------------------------------------------------- */

/// `generate_namespace` generates the mod.gd namespace file for a package.
///
/// The namespace file provides preloads for all types in the package and
/// subpackages, allowing users to import the entire package with a single
/// preload.
pub fn generate_namespace(
    cw: &mut CodeWriter,
    pkg_name: &str,
    class_name: Option<&str>,
    entries: &[TypeEntry],
    subpackages: &[String],
) -> anyhow::Result<String> {
    let mut w = StringWriter::default();

    let name = if pkg_name.is_empty() {
        "Root"
    } else {
        pkg_name
    };

    let mut sections = Vec::new();

    if !subpackages.is_empty() {
        sections.push(gen_dependencies(subpackages));
    }

    if !entries.is_empty() {
        sections.push(gen_types(entries));
    }

    sections.push(gen_engine_overrides(name));

    let script = ScriptBuilder::default()
        .header(Comment::do_not_edit())
        .class_name(class_name.map(|s| s.to_owned()))
        .comment(Some(Comment::from("`{}` namespace.")))
        .extends("Object")
        .sections(sections)
        .build()
        .unwrap();

    script.emit(cw, &mut w)?;

    Ok(w.into_content())
}

/* -------------------------- Fn: gen_dependencies -------------------------- */

fn gen_dependencies(deps: &[String]) -> Section {
    let mut items = Vec::new();

    for dep in deps {
        let assignment = Assignment::preload(dep.clone(), format!("./{}/mod.gd", dep));
        items.push(assignment.into());
    }

    SectionBuilder::default()
        .header("DEPENDENCIES")
        .body(items)
        .build()
        .unwrap()
}

/* ------------------------ Fn: gen_engine_overrides ------------------------ */

fn gen_engine_overrides(name: &str) -> Section {
    let init = FnDefBuilder::default()
        .name("_init")
        .body(vec![
            FnCall::assert(
                Literal::from(false),
                format!("{} is non-instantiable", name),
            )
            .into(),
        ])
        .build()
        .unwrap();

    SectionBuilder::default()
        .header("ENGINE METHODS (OVERRIDES)")
        .body(vec![Item::FnDef(init)])
        .build()
        .unwrap()
}

/* ------------------------------ Fn: gen_types ----------------------------- */

fn gen_types(entries: &[TypeEntry]) -> Section {
    let mut items = Vec::new();

    for entry in entries {
        items.push(
            Assignment::preload(
                entry.file_stem.clone(),
                format!("./{}.gd", entry.file_stem.to_lowercase()),
            )
            .into(),
        );
    }

    SectionBuilder::default()
        .header("TYPES")
        .body(items)
        .build()
        .unwrap()
}
