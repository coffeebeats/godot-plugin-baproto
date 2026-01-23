use baproto::CodeWriter;
use baproto::Writer;
use derive_builder::Builder;

use super::Emit;

/* -------------------------------------------------------------------------- */
/*                                 Enum: Expr                                 */
/* -------------------------------------------------------------------------- */

#[derive(Clone, Debug)]
pub enum Expr {
    /// `Raw` is an arbitrary expression.
    Raw(String),
    /// `FnCall` is a function call expression.
    FnCall(FnCall),
}

/* --------------------------- Impl: From<String> --------------------------- */

impl From<String> for Expr {
    fn from(value: String) -> Self {
        Self::Raw(value)
    }
}

/* --------------------------- Impl: From<FnCall> --------------------------- */

impl From<FnCall> for Expr {
    fn from(value: FnCall) -> Self {
        Self::FnCall(value)
    }
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for Expr {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        match self {
            Self::Raw(s) => cw.write(w, s),
            Self::FnCall(f) => f.emit(cw, w),
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                               Struct: FnCall                               */
/* -------------------------------------------------------------------------- */

/// `FnCall` is a function call expression.
#[derive(Clone, Debug, Builder)]
#[builder(setter(into))]
pub struct FnCall {
    /// `receiver` is an object on which the target function is defined.
    #[builder(setter(strip_option))]
    pub receiver: Option<String>,

    /// `name` is the name of the function to call.
    pub name: String,

    /// `args` is the set of function arguments.
    #[builder(default)]
    pub args: Vec<Expr>,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for FnCall {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        if let Some(receiver) = self.receiver.as_ref() {
            cw.write(w, &format!("{}.", receiver))?;
        }

        cw.write(w, &format!("{}(", self.name))?;

        for arg in &self.args {
            arg.emit(cw, w)?;
            cw.write(w, ",")?;
        }

        cw.write(w, ")")?;

        Ok(())
    }
}
