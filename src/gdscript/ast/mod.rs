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

/* -------------------------------- Mod: Item ------------------------------- */

mod item;
pub use item::*;

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
