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
