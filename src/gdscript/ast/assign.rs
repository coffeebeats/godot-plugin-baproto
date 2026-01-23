use std::path::PathBuf;

use baproto::{CodeWriter, Writer};
use derive_builder::Builder;

use super::Comment;
use super::Emit;

/* -------------------------------------------------------------------------- */
/*                            Struct: Assignment                              */
/* -------------------------------------------------------------------------- */

/// `Assignment` represents a variable or constant declaration. Note that this
/// element is restricted to [`String`] values for now. In the future, support
/// for GDScript types may be added.
#[derive(Builder, Clone, Debug, Default)]
pub struct Assignment {
    /// `comment` is an optional doc comment associated with the assignment.
    pub comment: Option<Comment>,

    /// `declaration` is the declaration keyword used.
    #[builder(default, setter(strip_option))]
    pub declaration: Option<DeclarationKind>,

    /// `name` is the name of the declared variable.
    pub name: String,

    /// `type_hint` is an optional type hint associated with the declaration.
    #[builder(default, setter(strip_option))]
    pub type_hint: Option<TypeHint>,

    /// `value` is an optional value assigned to the declared variable.
    #[builder(setter(strip_option))]
    pub value: Option<ValueKind>,
}

/* ---------------------------- Impl: Assignment ---------------------------- */

impl Assignment {
    /// `builder` returns a default [`AssignmentBuilder`].
    pub fn builder() -> AssignmentBuilder {
        AssignmentBuilder::default()
    }
}

/* -------------------------- Enum: DeclarationKind ------------------------- */

/// `DeclarationKind` specifies the type of declaration.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum DeclarationKind {
    /// `Const` represents a `var` declaration.
    Const,
    /// `Var` represents a `var` declaration.
    #[default]
    Var,
}

/* ----------------------------- Enum: ValueKind ---------------------------- */

/// `ValueKind` specifies the type of assignment.
#[derive(Clone, Debug, PartialEq)]
pub enum ValueKind {
    /// `Raw` is a raw [`String`] on the right-hand side of the statement.
    Raw(String),
    /// `Preload` is a preload statement for the specified file.
    Preload(PathBuf),
}

/* ----------------------------- Enum: TypeHint ----------------------------- */

#[derive(Clone, Debug, Default)]
pub enum TypeHint {
    /// `Infer` defines a hint that's inferred from context.
    #[default]
    Infer,
    /// `Explicit` is a type hint that's explicitly specified.
    Explicit(String),
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for Assignment {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        if let Some(comment) = self.comment.as_ref() {
            comment.emit(cw, w)?;
        }

        match &self.declaration {
            None => Ok(()),
            Some(DeclarationKind::Const) => cw.write(w, "const "),
            Some(DeclarationKind::Var) => cw.write(w, "var "),
        }?;

        cw.write(w, &self.name)?;

        match &self.type_hint {
            None => cw.write(w, " ="),
            Some(TypeHint::Infer) => cw.write(w, " :="),
            Some(TypeHint::Explicit(hint)) => cw.write(w, &format!(": {} =", hint)),
        }?;

        match &self.value {
            None => todo!(),
            Some(ValueKind::Raw(s)) => cw.write(w, &format!(" {}", s)),
            Some(ValueKind::Preload(p)) => cw.write(w, &format!(" preload(\"{}\")", p.display())),
        }?;

        Ok(())
    }
}
