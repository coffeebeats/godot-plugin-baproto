use baproto::{CodeWriter, Writer};
use derive_builder::Builder;

use super::{Block, Emit, Expr};

/* -------------------------------------------------------------------------- */
/*                               Struct: ForIn                                */
/* -------------------------------------------------------------------------- */

/// `ForIn` represents a for-in loop.
#[derive(Clone, Debug)]
pub struct ForIn {
    pub variable: String,
    pub iterable: Expr,
    pub body: Block,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for ForIn {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        cw.write(w, &format!("for {} in ", self.variable))?;
        self.iterable.emit(cw, w)?;
        cw.writeln(w, ":")?;

        self.body.emit(cw, w)
    }
}

/* -------------------------------------------------------------------------- */
/*                                Struct: If                                  */
/* -------------------------------------------------------------------------- */

/// `If` represents an if-else conditional.
#[derive(Builder, Clone, Debug)]
pub struct If {
    #[builder(setter(into))]
    pub condition: Expr,
    pub then_body: Block,
    pub else_body: Option<Block>,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for If {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        cw.write(w, "if ")?;
        self.condition.emit(cw, w)?;
        cw.write(w, ":")?;

        self.then_body.emit(cw, w)?;

        if let Some(else_body) = &self.else_body {
            cw.writeln(w, "else:")?;
            else_body.emit(cw, w)?;
        }

        Ok(())
    }
}

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use baproto::StringWriter;

    use super::*;

    /* ------------------------------ Tests: If ----------------------------- */

    #[test]
    fn test_if_without_else() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = CodeWriter::default();

        // Given: An if statement without else.
        let if_stmt = If {
            condition: Expr::Raw("x > 0".to_string()),
            then_body: Block::default(),
            else_body: None,
        };

        // When: The if statement is serialized to source code.
        let result = if_stmt.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "if x > 0:\n\tpass\n");
    }

    #[test]
    fn test_if_with_else() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = CodeWriter::default();

        // Given: An if statement with else.
        let if_stmt = If {
            condition: Expr::Raw("ready".to_string()),
            then_body: Block::default(),
            else_body: Some(Block::default()),
        };

        // When: The if statement is serialized to source code.
        let result = if_stmt.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "if ready:\n\tpass\nelse:\n\tpass\n");
    }

    /* ---------------------------- Tests: ForIn ---------------------------- */

    #[test]
    fn test_for_in_loop() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = CodeWriter::default();

        // Given: A for-in loop.
        let for_loop = ForIn {
            variable: "item".to_string(),
            iterable: Expr::Raw("items".to_string()),
            body: Block::default(),
        };

        // When: The for loop is serialized to source code.
        let result = for_loop.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "for item in items:\n\tpass\n");
    }
}
