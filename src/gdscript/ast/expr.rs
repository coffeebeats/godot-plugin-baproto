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
    #[builder(setter(into, strip_option))]
    pub receiver: Option<String>,

    /// `name` is the name of the function to call.
    #[builder(setter(into))]
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

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use baproto::StringWriter;

    use super::*;

    /* ----------------------------- Tests: Expr ---------------------------- */

    #[test]
    fn test_expr_raw() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = CodeWriter::default();

        // Given: A raw expression.
        let expr = Expr::Raw("x + y * 2".to_string());

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "x + y * 2");
    }

    /* ---------------------------- Tests: FnCall --------------------------- */

    #[test]
    fn test_fn_call_without_receiver_or_args() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = CodeWriter::default();

        // Given: A function call without receiver or arguments.
        let fn_call = FnCall {
            receiver: None,
            name: "print_debug".to_string(),
            args: vec![],
        };

        // When: The function call is serialized to source code.
        let result = fn_call.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "print_debug()");
    }

    #[test]
    fn test_fn_call_with_receiver() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = CodeWriter::default();

        // Given: A function call with receiver.
        let fn_call = FnCall {
            receiver: Some("self".to_string()),
            name: "get_node".to_string(),
            args: vec![],
        };

        // When: The function call is serialized to source code.
        let result = fn_call.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "self.get_node()");
    }

    #[test]
    fn test_fn_call_with_args() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = CodeWriter::default();

        // Given: A function call with multiple arguments.
        let fn_call = FnCall {
            receiver: None,
            name: "add".to_string(),
            args: vec![
                Expr::Raw("1".to_string()),
                Expr::Raw("2".to_string()),
                Expr::Raw("3".to_string()),
            ],
        };

        // When: The function call is serialized to source code.
        let result = fn_call.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "add(1,2,3,)");
    }
}
