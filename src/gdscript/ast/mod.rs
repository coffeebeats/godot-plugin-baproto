use baproto::{CodeWriter, Writer};

/* ------------------------------- Mod: Assign ------------------------------ */

mod assign;
pub use assign::*;

/* ------------------------------ Mod: Comment ------------------------------ */

mod comment;
pub use comment::*;

/* ------------------------------ Mod: Control ------------------------------ */

mod control;
pub use control::*;

/* -------------------------------- Mod: Expr ------------------------------- */

mod expr;
pub use expr::*;

/* ------------------------------ Mod: Function ----------------------------- */

mod function;
pub use function::*;

/* -------------------------------- Mod: Script ------------------------------- */

mod script;
pub use script::*;

/* -------------------------------------------------------------------------- */
/*                                Trait: Emit                                 */
/* -------------------------------------------------------------------------- */

/// `Emit` writes a GDScript construct to a `CodeWriter`.
#[allow(dead_code)]
pub trait Emit {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()>;
}

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
}

/* ------------------------- Impl: From<Assignment> ------------------------- */

impl From<Assignment> for Item {
    fn from(value: Assignment) -> Self {
        Self::Assignment(value)
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

impl Emit for Item {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        match self {
            Item::Expr(expr) => expr.emit(cw, w),
            Item::Assignment(assignment) => assignment.emit(cw, w),
            Item::ForIn(f) => f.emit(cw, w),
            Item::If(i) => i.emit(cw, w),
            Item::FnDef(f) => f.emit(cw, w),
        }
    }
}
