use baproto::{CodeWriter, Writer};

use super::Assignment;
use super::Expr;
use super::FnDef;
use super::ForIn;
use super::If;

/* -------------------------------------------------------------------------- */
/*                                 Enum: Item                                 */
/* -------------------------------------------------------------------------- */

/// `Item` represents a top-level construct in GDScript code.
#[derive(Clone, Debug)]
pub enum Item {
    /// `Assignment` is a variable declaration or assignment.
    Assignment(Assignment),

    /// `Expr` is an expression (method call, etc.).
    Expr(Expr),

    /// `FnDef` is a function definition.
    FnDef(FnDef),

    /// For-in loop.
    ForIn(ForIn),

    /// If-else conditional.
    If(If),

    /// Early return statement.
    Return(Expr),
}

/* ------------------------- Impl: From<Assignment> ------------------------- */

impl From<Assignment> for Item {
    fn from(value: Assignment) -> Self {
        Self::Assignment(value)
    }
}

/* ---------------------------- Impl: From<Expr> ---------------------------- */

impl From<Expr> for Item {
    fn from(value: Expr) -> Self {
        Self::Expr(value)
    }
}

/* ---------------------------- Impl: From<ForIn> --------------------------- */

impl From<ForIn> for Item {
    fn from(value: ForIn) -> Self {
        Self::ForIn(value)
    }
}

/* ----------------------------- Impl: From<If> ----------------------------- */

impl From<If> for Item {
    fn from(value: If) -> Self {
        Self::If(value)
    }
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl super::Emit for Item {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        match self {
            Item::Expr(expr) => expr.emit(cw, w),
            Item::Assignment(assignment) => assignment.emit(cw, w),
            Item::ForIn(f) => f.emit(cw, w),
            Item::If(i) => i.emit(cw, w),
            Item::FnDef(f) => f.emit(cw, w),
            Item::Return(expr) => {
                cw.write(w, "return ")?;
                expr.emit(cw, w)
            }
        }
    }
}
