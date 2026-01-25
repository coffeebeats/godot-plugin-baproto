use baproto::CodeWriter;
use baproto::Writer;
use derive_builder::Builder;

use super::Assignment;
use super::Comment;
use super::Emit;
use super::Expr;
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
    #[builder(
        default = Some(TypeHint::Explicit("void".to_owned())),
        setter(into, strip_option),
    )]
    pub type_hint: Option<TypeHint>,

    /// `body` is the function's contents.
    #[builder(default, setter(into))]
    pub body: Block,

    /// `return_value` is an optional return expression at the end of the
    /// function. Migrate this to [`Item`] if multiple return support is needed.
    #[builder(default, setter(into, strip_option))]
    pub return_value: Option<Expr>,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for FnDef {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        if let Some(comment) = self.comment.as_ref() {
            comment.emit(cw, w)?;
        }

        cw.write(w, &format!("func {}(", self.name))?;

        for (i, param) in self.params.iter().enumerate() {
            param.emit(cw, w)?;
            if i < self.params.len() - 1 {
                cw.write(w, ", ")?;
            }
        }

        cw.write(w, ")")?;

        match &self.type_hint {
            None | Some(TypeHint::Infer) => cw.writeln(w, ":"),
            Some(TypeHint::Explicit(hint)) => cw.writeln(w, &format!(" -> {}:", hint)),
        }?;

        self.body.emit(cw, w)?;

        if let Some(return_expr) = &self.return_value {
            cw.indent();

            cw.write(w, &format!("{}return ", cw.get_indent()))?;
            return_expr.emit(cw, w)?;

            cw.outdent();
            cw.newline(w)?;
        }

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

/* -------------------------- Impl: From<Vec<Item>> ------------------------- */

impl From<Vec<Item>> for Block {
    fn from(items: Vec<Item>) -> Self {
        Block { body: items }
    }
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

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use baproto::StringWriter;

    use crate::gdscript::{GDScript, ast::Literal};

    use super::*;

    /* ---------------------------- Tests: Block ---------------------------- */

    #[test]
    fn test_block_empty() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: An empty block.
        let block = Block::default();

        // When: The block is serialized to source code.
        let result = block.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "\tpass\n");
    }

    /* ---------------------------- Tests: FnDef ---------------------------- */

    #[test]
    fn test_fn_def_no_params_no_return_type() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A function with no parameters or return type.
        let func = FnDef {
            comment: None,
            name: "_ready".to_string(),
            params: vec![],
            type_hint: None,
            body: Block::default(),
            return_value: None,
        };

        // When: The function is serialized to source code.
        let result = func.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "func _ready():\n\tpass\n");
    }

    #[test]
    fn test_fn_def_with_params() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A function with parameters.
        let func = FnDef {
            comment: None,
            name: "add".to_string(),
            params: vec![
                Assignment::param_with_default("a", "int", Literal::Int(0)),
                Assignment::param_with_default("b", "int", Literal::Int(0)),
            ],
            type_hint: None,
            body: Block::default(),
            return_value: None,
        };

        // When: The function is serialized to source code.
        let result = func.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(
            s.into_content(),
            "func add(a: int = 0, b: int = 0):\n\tpass\n"
        );
    }

    #[test]
    fn test_fn_def_with_return_type() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A function with explicit return type.
        let func = FnDef {
            comment: None,
            name: "get_value".to_string(),
            params: vec![],
            type_hint: Some(TypeHint::Explicit("int".to_string())),
            body: Block::default(),
            return_value: None,
        };

        // When: The function is serialized to source code.
        let result = func.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "func get_value() -> int:\n\tpass\n");
    }

    #[test]
    fn test_fn_def_with_return_value() {
        use crate::gdscript::ast::{Expr, Literal};

        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A function with a return value.
        let func = FnDef {
            comment: None,
            name: "get_five".to_string(),
            params: vec![],
            type_hint: Some(TypeHint::Explicit("int".to_string())),
            body: Block::default(),
            return_value: Some(Expr::Literal(Literal::Int(5))),
        };

        // When: The function is serialized to source code.
        let result = func.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(
            s.into_content(),
            "func get_five() -> int:\n\tpass\n\treturn 5\n"
        );
    }
}
