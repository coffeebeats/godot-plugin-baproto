use baproto::{CodeWriter, Writer};
use derive_builder::Builder;

use super::Emit;

/* -------------------------------------------------------------------------- */
/*                              Struct: EnumDecl                              */
/* -------------------------------------------------------------------------- */

/// `EnumDecl` represents a GDScript enum declaration.
#[derive(Builder, Clone, Debug)]
pub struct EnumDecl {
    /// Optional enum name (None for anonymous enum)
    #[builder(default, setter(strip_option))]
    pub name: Option<String>,
    /// Enum variant names and values
    pub variants: Vec<(String, i64)>,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for EnumDecl {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        cw.write(w, "enum")?;
        if let Some(name) = &self.name {
            cw.write(w, &format!(" {}", name))?;
        }
        cw.write(w, " {")?;
        cw.newline(w)?;

        cw.indent();
        for (i, (variant_name, value)) in self.variants.iter().enumerate() {
            cw.write(w, &cw.get_indent())?;
            cw.write(w, &format!("{} = {},", variant_name, value))?;
            if i < self.variants.len() - 1 {
                cw.newline(w)?;
            }
        }
        cw.outdent();

        cw.newline(w)?;
        cw.write(w, "}")?;

        Ok(())
    }
}

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use baproto::StringWriter;

    use crate::gdscript::GDScript;

    use super::*;

    /* -------------------------- Tests: EnumDecl --------------------------- */

    #[test]
    fn test_enum_decl_anonymous() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: An anonymous enum declaration.
        let enum_decl = EnumDeclBuilder::default()
            .variants(vec![
                ("NONE".to_string(), -1),
                ("ACTIVE".to_string(), 0),
                ("INACTIVE".to_string(), 1),
            ])
            .build()
            .unwrap();

        // When: The enum is serialized to source code.
        let result = enum_decl.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(
            s.into_content(),
            "enum {\n\tNONE = -1,\n\tACTIVE = 0,\n\tINACTIVE = 1,\n}"
        );
    }

    #[test]
    fn test_enum_decl_named() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A named enum declaration.
        let enum_decl = EnumDeclBuilder::default()
            .name("Status".to_string())
            .variants(vec![
                ("PENDING".to_string(), 0),
                ("COMPLETE".to_string(), 1),
            ])
            .build()
            .unwrap();

        // When: The enum is serialized to source code.
        let result = enum_decl.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(
            s.into_content(),
            "enum Status {\n\tPENDING = 0,\n\tCOMPLETE = 1,\n}"
        );
    }
}
