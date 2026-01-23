use baproto::{CodeWriter, Writer};

use super::Emit;

/* -------------------------------------------------------------------------- */
/*                               Struct: Comment                              */
/* -------------------------------------------------------------------------- */

/// `Comment` represents a comment in the code.
#[derive(Clone, Debug, Default)]
pub struct Comment {
    pub contents: Vec<String>,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for Comment {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        cw.comment_block(w, &self.contents.join("\n"))
    }
}

/* -------------------------------------------------------------------------- */
/*                            Struct: SectionHeader                           */
/* -------------------------------------------------------------------------- */

/// `SectionHeader` represents a delimeter that demarcates a script section.
#[derive(Clone, Debug, Default)]
pub struct SectionHeader {
    pub title: String,
}

/* --------------------------- Impl: From<String> --------------------------- */

impl<T: AsRef<str>> From<T> for SectionHeader {
    fn from(value: T) -> Self {
        SectionHeader {
            title: value.as_ref().to_owned(),
        }
    }
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for SectionHeader {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        cw.writeln(
            w,
            &format!(
                "# -- {} {} #",
                self.title,
                "-".repeat(88 - self.title.len() - 4 * cw.indent_level() - 7)
            ),
        )
    }
}

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use baproto::StringWriter;

    use super::*;

    #[test]
    fn test_section_header_emits_correct_comment() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = CodeWriter::default();

        // Given: A section with a short name.
        let section = SectionHeader::from("test");

        // When: The header is serialized to source code.
        let header = section.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(header.is_ok());

        // Then: The comment matches expectations.
        assert_eq!(s.into_content(), format!("# -- test {} #", "-".repeat(67)));
    }
}
