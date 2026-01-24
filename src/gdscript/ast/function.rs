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

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use baproto::StringWriter;

    use super::*;

    /* ---------------------------- Tests: Block ---------------------------- */

    #[test]
    fn test_block_empty() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = CodeWriter::default();

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
        let mut cw = CodeWriter::default();

        // Given: A function with no parameters or return type.
        let func = FnDef {
            comment: None,
            name: "_ready".to_string(),
            params: vec![],
            type_hint: None,
            body: Block::default(),
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
        let mut cw = CodeWriter::default();

        // Given: A function with parameters.
        let func = FnDef {
            comment: None,
            name: "add".to_string(),
            params: vec![
                Assignment::builder()
                    .name("a".to_string())
                    .value(super::super::ValueKind::Raw("0".to_string()))
                    .build()
                    .unwrap(),
                Assignment::builder()
                    .name("b".to_string())
                    .value(super::super::ValueKind::Raw("0".to_string()))
                    .build()
                    .unwrap(),
            ],
            type_hint: None,
            body: Block::default(),
        };

        // When: The function is serialized to source code.
        let result = func.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "func add(a = 0b = 0):\n\tpass\n");
    }

    #[test]
    fn test_fn_def_with_return_type() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = CodeWriter::default();

        // Given: A function with explicit return type.
        let func = FnDef {
            comment: None,
            name: "get_value".to_string(),
            params: vec![],
            type_hint: Some(TypeHint::Explicit("int".to_string())),
            body: Block::default(),
        };

        // When: The function is serialized to source code.
        let result = func.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "func get_value() -> int:\n\tpass\n");
    }
}
