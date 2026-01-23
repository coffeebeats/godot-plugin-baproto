use baproto::CodeWriter;
use baproto::Writer;
use derive_builder::Builder;

use super::Assignment;
use super::Comment;
use super::Emit;
use super::Item;
use super::TypeHint;

/* -------------------------------------------------------------------------- */
/*                                Struct: FnDef                               */
/* -------------------------------------------------------------------------- */

/// `FnDef` represents a function declaration.
#[derive(Clone, Debug, Builder)]
#[builder(setter(into))]
pub struct FnDef {
    /// `comment` is a doc comment for the function.
    #[builder(default, setter(into, strip_option))]
    pub comment: Option<Comment>,

    /// `name` is the name of the function.
    pub name: String,

    /// `params` is the set of function parameters.
    #[builder(default)]
    pub params: Vec<Assignment>,

    /// `type_hint` is the function's return type hint.
    #[builder(default, setter(into, strip_option))]
    pub type_hint: Option<TypeHint>,

    /// `body` is the function's contents.
    #[builder(default)]
    pub body: Block,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for FnDef {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        if let Some(comment) = self.comment.as_ref() {
            comment.emit(cw, w)?;
        }

        cw.write(w, &format!("func {}(", self.name))?;

        for param in &self.params {
            param.emit(cw, w)?;
        }

        cw.write(w, ")")?;

        match &self.type_hint {
            None | Some(TypeHint::Infer) => cw.writeln(w, ":"),
            Some(TypeHint::Explicit(hint)) => cw.writeln(w, &format!(" -> {}:", hint)),
        }?;

        self.body.emit(cw, w)?;

        Ok(())
    }
}

/* -------------------------------------------------------------------------- */
/*                                Struct: Block                               */
/* -------------------------------------------------------------------------- */

/// `Block` represents a nested block of code (used in function definitions and
/// control statement bodies).
#[derive(Clone, Debug, Default)]
pub struct Block {
    pub body: Vec<Item>,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for Block {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        cw.indent();

        if self.body.is_empty() {
            cw.writeln(w, "pass")?;
        } else {
            for item in &self.body {
                item.emit(cw, w)?;
            }
        }

        cw.outdent();

        Ok(())
    }
}
