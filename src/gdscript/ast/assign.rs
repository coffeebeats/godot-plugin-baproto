use std::path::Path;
use std::path::PathBuf;

use baproto::{CodeWriter, Writer};
use derive_builder::Builder;

use super::Comment;
use super::Emit;
use super::Expr;

/* -------------------------------------------------------------------------- */
/*                            Struct: Assignment                              */
/* -------------------------------------------------------------------------- */

/// `Assignment` represents a variable or constant declaration. Note that this
/// element is restricted to [`String`] values for now. In the future, support
/// for GDScript types may be added.
#[derive(Builder, Clone, Debug)]
pub struct Assignment {
    /// `comment` is an optional doc comment associated with the assignment.
    #[builder(default)]
    pub comment: Option<Comment>,

    /// `declaration` is the declaration keyword used.
    #[builder(default = None, setter(into, strip_option))]
    pub declaration: Option<DeclarationKind>,

    /// `name` is the name of the declared variable.
    #[builder(setter(into))]
    pub variable: Expr,

    /// `type_hint` is an optional type hint associated with the declaration.
    #[builder(default = Some(TypeHint::Infer), setter(into))]
    pub type_hint: Option<TypeHint>,

    /// `value` is an optional value assigned to the declared variable.
    #[builder(default, setter(into, strip_option))]
    pub value: Option<ValueKind>,
}

/* ---------------------------- Impl: Assignment ---------------------------- */

impl Assignment {
    /// `reassign` reassigns the specified `variable` to the provided `value`.
    pub fn reassign<T, U>(variable: T, value: U) -> Self
    where
        T: Into<Expr>,
        U: Into<Expr>,
    {
        AssignmentBuilder::default()
            .variable(variable.into())
            .type_hint(None)
            .value(value.into())
            .build()
            .unwrap()
    }

    /// `param` creates a function parameter. To create one with a default
    /// value, see [`Assignment::param_with_default`].
    pub fn param<T, U>(name: T, hint: U) -> Self
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        AssignmentBuilder::default()
            .variable(name.as_ref())
            .type_hint(Some(TypeHint::Explicit(hint.as_ref().to_owned())))
            .build()
            .unwrap()
    }

    /// `param` creates a function parameter. To create one without a default
    /// value, see [`Assignment::param`].
    pub fn param_with_default<T, U, V>(name: T, hint: U, value: V) -> Self
    where
        T: AsRef<str>,
        U: AsRef<str>,
        V: Into<Expr>,
    {
        AssignmentBuilder::default()
            .variable(name.as_ref())
            .type_hint(Some(TypeHint::Explicit(hint.as_ref().to_owned())))
            .value(ValueKind::from(value.into()))
            .build()
            .unwrap()
    }

    /// `preload` creates a script `preload` definition.
    pub fn preload<T, U>(name: T, path: U) -> Self
    where
        T: AsRef<str>,
        U: AsRef<Path>,
    {
        AssignmentBuilder::default()
            .declaration(DeclarationKind::Const)
            .variable(name.as_ref())
            .value(ValueKind::Preload(path.as_ref().to_path_buf()))
            .build()
            .unwrap()
    }

    /// `var` creates a type-inferred new variable definition.
    pub fn var<T, U>(name: T, value: U) -> Self
    where
        T: AsRef<str>,
        U: Into<Expr>,
    {
        AssignmentBuilder::default()
            .declaration(DeclarationKind::Var)
            .type_hint(TypeHint::Infer)
            .variable(name.as_ref())
            .value(value.into())
            .build()
            .unwrap()
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
    /// `Expr` is a structured expression on the right-hand side.
    Expr(Expr),
}

/* ---------------------------- Impl: From<Expr> ---------------------------- */

impl From<Expr> for ValueKind {
    fn from(value: Expr) -> Self {
        Self::Expr(value)
    }
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

/* -------------------------------------------------------------------------- */
/*                              Fn: emit_value                                */
/* -------------------------------------------------------------------------- */

/// `emit_value` emits a value for an assignment.
fn emit_value<W: Writer>(value: &ValueKind, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
    match value {
        ValueKind::Raw(s) => cw.write(w, &format!(" {}", s)),
        ValueKind::Preload(p) => cw.write(w, &format!(" preload(\"{}\")", p.display())),
        ValueKind::Expr(expr) => {
            cw.write(w, " ")?;
            expr.emit(cw, w)?;
            Ok(())
        }
    }
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

        self.variable.emit(cw, w)?;

        match (&self.type_hint, &self.value) {
            // Function parameter without default value: name: Type
            (Some(TypeHint::Explicit(hint)), None) => cw.write(w, &format!(": {}", hint)),
            // No type hint, just assignment: name = value
            (None, Some(value)) => {
                cw.write(w, " =")?;
                emit_value(value, cw, w)
            }
            // Type inference: name := value
            (Some(TypeHint::Infer), Some(value)) => {
                cw.write(w, " :=")?;
                emit_value(value, cw, w)
            }
            // Explicit type with value: name: Type = value
            (Some(TypeHint::Explicit(hint)), Some(value)) => {
                cw.write(w, &format!(": {} =", hint))?;
                emit_value(value, cw, w)
            }
            // Invalid cases
            (None, None) | (Some(TypeHint::Infer), None) => {
                anyhow::bail!("Assignment requires a value when using this type hint")
            }
        }?;

        Ok(())
    }
}

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use baproto::StringWriter;

    use crate::gdscript::GDScript;

    use super::*;

    /* -------------------------- Tests: Assignment ------------------------- */

    #[test]
    fn test_assignment_var_with_raw_value() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A var assignment with a raw value.
        let assignment = AssignmentBuilder::default()
            .variable("my_var".to_string())
            .declaration(DeclarationKind::Var)
            .type_hint(None)
            .value(ValueKind::Raw("42".to_string()))
            .build()
            .unwrap();

        // When: The assignment is serialized to source code.
        let result = assignment.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "var my_var = 42");
    }

    #[test]
    fn test_assignment_const_with_preload() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A const assignment with a preload value.
        let assignment = AssignmentBuilder::default()
            .variable("MyClass".to_string())
            .declaration(DeclarationKind::Const)
            .value(ValueKind::Preload(PathBuf::from("res://script.gd")))
            .build()
            .unwrap();

        // When: The assignment is serialized to source code.
        let result = assignment.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(
            s.into_content(),
            "const MyClass := preload(\"res://script.gd\")"
        );
    }

    #[test]
    fn test_assignment_with_inferred_type_hint() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: An assignment with inferred type hint.
        let assignment = AssignmentBuilder::default()
            .variable("value".to_string())
            .declaration(DeclarationKind::Var)
            .type_hint(TypeHint::Infer)
            .value(ValueKind::Raw("\"hello\"".to_string()))
            .build()
            .unwrap();

        // When: The assignment is serialized to source code.
        let result = assignment.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "var value := \"hello\"");
    }

    #[test]
    fn test_assignment_with_explicit_type_hint() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: An assignment with explicit type hint.
        let assignment = AssignmentBuilder::default()
            .variable("count".to_string())
            .declaration(DeclarationKind::Var)
            .type_hint(TypeHint::Explicit("int".to_string()))
            .value(ValueKind::Raw("0".to_string()))
            .build()
            .unwrap();

        // When: The assignment is serialized to source code.
        let result = assignment.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "var count: int = 0");
    }

    #[test]
    fn test_assignment_with_expr_value() {
        use crate::gdscript::ast::{Expr, Literal};

        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: An assignment with structured expression value.
        let assignment = AssignmentBuilder::default()
            .variable("items".to_string())
            .declaration(DeclarationKind::Var)
            .type_hint(TypeHint::Infer)
            .value(ValueKind::Expr(Expr::Literal(Literal::Array(vec![]))))
            .build()
            .unwrap();

        // When: The assignment is serialized to source code.
        let result = assignment.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "var items := []");
    }
}
