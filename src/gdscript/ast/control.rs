use baproto::{CodeWriter, Writer};
use derive_builder::Builder;

use super::{Block, Emit, Expr};

/* -------------------------------------------------------------------------- */
/*                               Struct: ForIn                                */
/* -------------------------------------------------------------------------- */

/// `ForIn` represents a for-in loop.
#[derive(Builder, Clone, Debug)]
pub struct ForIn {
    #[builder(setter(into))]
    pub variable: String,
    #[builder(setter(into))]
    pub iterable: Expr,
    #[builder(default, setter(into))]
    pub body: Block,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for ForIn {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        cw.write(w, &format!("for {} in ", self.variable))?;
        self.iterable.emit(cw, w)?;
        cw.write(w, ":")?;
        cw.newline(w)?;

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
    #[builder(default)]
    pub then_body: Block,
    #[builder(default, setter(strip_option))]
    pub else_body: Option<Block>,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for If {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        cw.write(w, "if ")?;
        self.condition.emit(cw, w)?;
        cw.write(w, ":")?;
        cw.newline(w)?;

        self.then_body.emit(cw, w)?;

        if let Some(else_body) = &self.else_body {
            cw.newline(w)?;
            cw.write(w, "else:")?;
            cw.newline(w)?;
            else_body.emit(cw, w)?;
        }

        Ok(())
    }
}

/* -------------------------------------------------------------------------- */
/*                               Struct: Match                                */
/* -------------------------------------------------------------------------- */

/// `Match` represents a match statement for pattern matching.
#[derive(Builder, Clone, Debug)]
pub struct Match {
    #[builder(setter(into))]
    pub scrutinee: Expr,
    #[builder(default)]
    pub arms: Vec<MatchArm>,
}

/// `MatchArm` represents a single arm in a match statement.
#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pattern: Expr,
    pub body: Block,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for Match {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        cw.write(w, "match ")?;
        self.scrutinee.emit(cw, w)?;
        cw.write(w, ":")?;
        cw.newline(w)?;

        cw.indent();
        for arm in &self.arms {
            cw.write(w, &cw.get_indent())?;
            arm.pattern.emit(cw, w)?;
            cw.write(w, ":")?;
            cw.newline(w)?;
            arm.body.emit(cw, w)?;
            cw.newline(w)?;
        }
        cw.outdent();

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

    /* ------------------------------ Tests: If ----------------------------- */

    #[test]
    fn test_if_without_else() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: An if statement without else.
        let if_stmt = IfBuilder::default()
            .condition(Expr::from("x > 0"))
            .build()
            .unwrap();

        // When: The if statement is serialized to source code.
        let result = if_stmt.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "if x > 0:\n\tpass");
    }

    #[test]
    fn test_if_with_else() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: An if statement with else.
        let if_stmt = IfBuilder::default()
            .condition(Expr::from("ready"))
            .else_body(Block::default())
            .build()
            .unwrap();

        // When: The if statement is serialized to source code.
        let result = if_stmt.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "if ready:\n\tpass\nelse:\n\tpass");
    }

    /* --------------------------- Tests: Match ----------------------------- */

    #[test]
    fn test_match_statement() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A match statement with multiple arms.
        let match_stmt = MatchBuilder::default()
            .scrutinee(Expr::from("value"))
            .arms(vec![
                MatchArm {
                    pattern: Expr::from("0"),
                    body: Block::default(),
                },
                MatchArm {
                    pattern: Expr::from("1"),
                    body: Block::default(),
                },
            ])
            .build()
            .unwrap();

        // When: The match statement is serialized to source code.
        let result = match_stmt.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(
            s.into_content(),
            "match value:\n\t0:\n\t\tpass\n\t1:\n\t\tpass\n"
        );
    }

    /* ---------------------------- Tests: ForIn ---------------------------- */

    #[test]
    fn test_for_in_loop() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A for-in loop.
        let for_loop = ForInBuilder::default()
            .variable("item")
            .iterable(Expr::from("items"))
            .build()
            .unwrap();

        // When: The for loop is serialized to source code.
        let result = for_loop.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "for item in items:\n\tpass");
    }
}
