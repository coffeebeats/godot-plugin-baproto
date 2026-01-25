use baproto::CodeWriter;
use baproto::Writer;
use derive_builder::Builder;

use super::Emit;

/* -------------------------------------------------------------------------- */
/*                                 Enum: Expr                                 */
/* -------------------------------------------------------------------------- */

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// `BinaryOp` is a binary operator expression.
    BinaryOp(BinaryOp),
    /// `FnCall` is a function call expression.
    FnCall(FnCall),
    /// `FieldAccess` is a property access expression.
    FieldAccess(FieldAccess),
    /// `Identifier` is a simple variable reference.
    Identifier(String),
    /// `IndexAccess` is an array/map subscript access.
    IndexAccess(IndexAccess),
    /// `Literal` is a type-safe literal value.
    Literal(Literal),
}

/* ------------------------------- Impl: Expr ------------------------------- */

impl Expr {
    /// `ident` creates an identifer expression.
    pub fn ident<T: AsRef<str>>(name: T) -> Expr {
        Expr::Identifier(name.as_ref().into())
    }

    /// `field` creates a field access expression.
    #[allow(dead_code)]
    pub fn field<T: Into<Expr>, U: AsRef<str>>(receiver: T, field: U) -> Expr {
        Expr::FieldAccess(FieldAccess {
            receiver: Box::new(receiver.into()),
            field: field.as_ref().to_string(),
        })
    }

    /// `index` creates an index access expression.
    pub fn index<T: Into<Expr>, U: Into<Expr>>(receiver: T, index: U) -> Expr {
        Expr::IndexAccess(IndexAccess {
            receiver: Box::new(receiver.into()),
            index: Box::new(index.into()),
        })
    }

    /// `empty_array` creates an empty array literal.
    pub fn empty_array() -> Expr {
        Expr::Literal(Literal::Array(vec![]))
    }

    /// `empty_dict` creates an empty dict literal.
    pub fn empty_dict() -> Expr {
        Expr::Literal(Literal::Dict(vec![]))
    }

    /// `null` creates an expression for GDScript's `null`.
    pub fn null() -> Expr {
        Expr::Identifier("null".to_string())
    }

    /// `binary_op` creates a binary operation expression.
    pub fn binary_op<T: Into<Expr>, U: Into<Expr>>(left: T, op: Operator, right: U) -> Expr {
        Expr::BinaryOp(BinaryOp {
            left: Box::new(left.into()),
            op,
            right: Box::new(right.into()),
        })
    }
}

/* ------------------------- Impl: From<AsRef<str>> ------------------------- */

impl<T: AsRef<str>> From<T> for Expr {
    fn from(value: T) -> Self {
        Self::Identifier(value.as_ref().to_owned())
    }
}

/* ------------------------- Impl: From<BinaryOp> --------------------------- */

impl From<BinaryOp> for Expr {
    fn from(value: BinaryOp) -> Self {
        Self::BinaryOp(value)
    }
}

/* --------------------------- Impl: From<FnCall> --------------------------- */

impl From<FnCall> for Expr {
    fn from(value: FnCall) -> Self {
        Self::FnCall(value)
    }
}

/* ------------------------ Impl: From<FieldAccess> ------------------------- */

impl From<FieldAccess> for Expr {
    fn from(value: FieldAccess) -> Self {
        Self::FieldAccess(value)
    }
}

/* ------------------------ Impl: From<IndexAccess> ------------------------- */

impl From<IndexAccess> for Expr {
    fn from(value: IndexAccess) -> Self {
        Self::IndexAccess(value)
    }
}

/* -------------------------- Impl: From<Literal> --------------------------- */

impl From<Literal> for Expr {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for Expr {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        match self {
            Self::BinaryOp(b) => b.emit(cw, w),
            Self::FnCall(f) => f.emit(cw, w),
            Self::FieldAccess(f) => f.emit(cw, w),
            Self::Identifier(name) => cw.write(w, name),
            Self::IndexAccess(i) => i.emit(cw, w),
            Self::Literal(l) => l.emit(cw, w),
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                             Struct: BinaryOp                               */
/* -------------------------------------------------------------------------- */

/// `BinaryOp` is a binary operator expression.
#[derive(Clone, Debug, PartialEq)]
pub struct BinaryOp {
    /// `left` is the left-hand side expression.
    pub left: Box<Expr>,
    /// `op` is the binary operator.
    pub op: Operator,
    /// `right` is the right-hand side expression.
    pub right: Box<Expr>,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for BinaryOp {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        self.left.emit(cw, w)?;
        cw.write(w, " ")?;
        self.op.emit(cw, w)?;
        cw.write(w, " ")?;
        self.right.emit(cw, w)?;
        Ok(())
    }
}

/* -------------------------------------------------------------------------- */
/*                              Enum: Operator                                */
/* -------------------------------------------------------------------------- */

/// `Operator` represents a binary operator.
#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    /// `Eq` is the equality operator.
    Eq,
    /// `NotEq` is the inequality operator.
    NotEq,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for Operator {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        let s = match self {
            Self::Eq => "==",
            Self::NotEq => "!=",
        };
        cw.write(w, s)
    }
}

/* -------------------------------------------------------------------------- */
/*                             Struct: FieldAccess                            */
/* -------------------------------------------------------------------------- */

/// `FieldAccess` is a property access expression.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldAccess {
    /// `receiver` is the expression on which the field is accessed.
    pub receiver: Box<Expr>,
    /// `field` is the name of the field being accessed.
    pub field: String,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for FieldAccess {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        self.receiver.emit(cw, w)?;
        cw.write(w, &format!(".{}", self.field))?;
        Ok(())
    }
}

/* -------------------------------------------------------------------------- */
/*                             Struct: IndexAccess                            */
/* -------------------------------------------------------------------------- */

/// `IndexAccess` is an array/map subscript access expression.
#[derive(Clone, Debug, PartialEq)]
pub struct IndexAccess {
    /// `receiver` is the expression being indexed.
    pub receiver: Box<Expr>,
    /// `index` is the index expression.
    pub index: Box<Expr>,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for IndexAccess {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        self.receiver.emit(cw, w)?;
        cw.write(w, "[")?;
        self.index.emit(cw, w)?;
        cw.write(w, "]")?;
        Ok(())
    }
}

/* -------------------------------------------------------------------------- */
/*                                Enum: Literal                               */
/* -------------------------------------------------------------------------- */

/// `Literal` represents a type-safe GDScript literal value.
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    /// `Bool` is a boolean literal.
    Bool(bool),
    /// `Int` is an integer literal.
    Int(i64),
    /// `Float` is a floating-point literal.
    Float(f32),
    /// `String` is a string literal.
    String(String),
    /// `Array` is an array literal.
    Array(Vec<Expr>),
    /// `Dict` is a dictionary literal.
    Dict(Vec<(Expr, Expr)>),
}

/* ---------------------------- Impl: From<bool> ---------------------------- */

impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

/* ----------------------------- Impl: From<i64> ---------------------------- */

impl From<i64> for Literal {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

/* ----------------------------- Impl: From<f32> ---------------------------- */

impl From<f32> for Literal {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

/* --------------------------- Impl: From<String> --------------------------- */

impl From<String> for Literal {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

/* ---------------------------- Impl: From<&str> ---------------------------- */

impl From<&str> for Literal {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

/* -------------------------- Impl: From<Vec<Expr>> ------------------------- */

impl From<Vec<Expr>> for Literal {
    fn from(value: Vec<Expr>) -> Self {
        Self::Array(value)
    }
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for Literal {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        match self {
            Self::Bool(b) => cw.write(w, if *b { "true" } else { "false" }),
            Self::Int(i) => cw.write(w, &i.to_string()),
            Self::Float(f) => {
                let s = if f.fract().abs() < f32::EPSILON {
                    &format!("{:.1}", f)
                } else {
                    &f.to_string()
                };

                cw.write(w, s)
            }
            Self::String(s) => cw.write(w, &format!("\"{}\"", s)),
            Self::Array(elements) => {
                cw.write(w, "[")?;
                for (idx, elem) in elements.iter().enumerate() {
                    if idx > 0 {
                        cw.write(w, ", ")?;
                    }
                    elem.emit(cw, w)?;
                }
                cw.write(w, "]")?;
                Ok(())
            }
            Self::Dict(pairs) => {
                cw.write(w, "{")?;
                for (idx, (key, value)) in pairs.iter().enumerate() {
                    if idx > 0 {
                        cw.write(w, ", ")?;
                    }
                    key.emit(cw, w)?;
                    cw.write(w, ": ")?;
                    value.emit(cw, w)?;
                }
                cw.write(w, "}")?;
                Ok(())
            }
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                               Struct: FnCall                               */
/* -------------------------------------------------------------------------- */

/// `FnCall` is a function call expression.
#[derive(Builder, Clone, Debug, PartialEq)]
#[builder(setter(into))]
pub struct FnCall {
    /// `receiver` is an object on which the target function is defined.
    #[builder(default, setter(strip_option))]
    pub receiver: Option<Box<Expr>>,

    /// `name` is the name of the function to call.
    #[builder(setter(into))]
    pub name: String,

    /// `args` is the set of function arguments.
    #[builder(default)]
    pub args: Vec<Expr>,
}

/* ------------------------------ Impl: FnCall ------------------------------ */

impl FnCall {
    /// `assert` creates a function call [`Expr`] representing a GDScript
    /// assertion.
    pub fn assert<T, U>(condition: T, message: U) -> Expr
    where
        T: Into<Expr>,
        U: AsRef<str>,
    {
        FnCall {
            args: vec![condition.into(), message.into()],
            name: "assert".to_owned(),
            receiver: None,
        }
        .into()
    }

    /// `function` creates a standalone function call [`Expr`]. To call a
    /// function with arguments, see [`FnCall::function_args`].
    pub fn function<T>(name: T) -> Expr
    where
        T: AsRef<str>,
    {
        FnCall {
            args: Vec::default(),
            name: name.as_ref().to_owned(),
            receiver: None,
        }
        .into()
    }

    /// `function` creates a standalone function call [`Expr`]. To call a
    /// function without arguments, see [`FnCall::function`].
    pub fn function_args<T, U>(name: T, args: Vec<U>) -> Expr
    where
        T: AsRef<str>,
        U: Into<Expr>,
    {
        FnCall {
            args: args.into_iter().map(|i| i.into()).collect(),
            name: name.as_ref().to_owned(),
            receiver: None,
        }
        .into()
    }

    /// `method` creates a function call [`Expr`] on the provided receiver. To
    /// call a method with arguments, see [`FnCall::method_args`].
    pub fn method<T, U>(receiver: T, name: U) -> Expr
    where
        T: Into<Expr>,
        U: AsRef<str>,
    {
        FnCall {
            args: Vec::default(),
            name: name.as_ref().to_owned(),
            receiver: Some(Box::new(receiver.into())),
        }
        .into()
    }

    /// `method` creates a function call [`Expr`] on the provided receiver. To
    /// call a method without arguments, see [`FnCall::method`].
    pub fn method_args<T, U, V>(receiver: T, name: U, args: Vec<V>) -> Expr
    where
        T: Into<Expr>,
        U: AsRef<str>,
        V: Into<Expr>,
    {
        FnCall {
            args: args.into_iter().map(|i| i.into()).collect(),
            name: name.as_ref().to_owned(),
            receiver: Some(Box::new(receiver.into())),
        }
        .into()
    }
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for FnCall {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        if let Some(receiver) = self.receiver.as_ref() {
            receiver.emit(cw, w)?;
            cw.write(w, ".")?;
        }

        cw.write(w, &format!("{}(", self.name))?;

        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                cw.write(w, ", ")?;
            }
            arg.emit(cw, w)?;
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

    use crate::gdscript::GDScript;

    use super::*;

    /* ---------------------------- Tests: FnCall --------------------------- */

    #[test]
    fn test_fn_call_without_receiver_or_args() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

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
        let mut cw = GDScript::writer();

        // Given: A function call with receiver.
        let fn_call = FnCall {
            receiver: Some(Box::new(Expr::Identifier("self".to_string()))),
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
        let mut cw = GDScript::writer();

        // Given: A function call with multiple arguments.
        let fn_call = FnCall {
            receiver: None,
            name: "add".to_string(),
            args: vec![
                Expr::Literal(1.into()),
                Expr::Literal(2.into()),
                Expr::Literal(3.into()),
            ],
        };

        // When: The function call is serialized to source code.
        let result = fn_call.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "add(1, 2, 3)");
    }

    #[test]
    fn test_fn_call_with_expr_receiver() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A function call with expression receiver.
        let fn_call = FnCall {
            receiver: Some(Box::new(Expr::FieldAccess(FieldAccess {
                receiver: Box::new(Expr::Identifier("player".to_string())),
                field: "stats".to_string(),
            }))),
            name: "get_health".to_string(),
            args: vec![],
        };

        // When: The function call is serialized to source code.
        let result = fn_call.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "player.stats.get_health()");
    }

    /* -------------------------- Tests: Identifier ------------------------- */

    #[test]
    fn test_identifier() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: An identifier expression.
        let expr = Expr::Identifier("my_variable".to_string());

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "my_variable");
    }

    /* ------------------------- Tests: FieldAccess ------------------------- */

    #[test]
    fn test_field_access_simple() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A field access expression.
        let expr = Expr::FieldAccess(FieldAccess {
            receiver: Box::new(Expr::Identifier("player".to_string())),
            field: "health".to_string(),
        });

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "player.health");
    }

    #[test]
    fn test_field_access_chained() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A chained field access expression.
        let expr = Expr::FieldAccess(FieldAccess {
            receiver: Box::new(Expr::FieldAccess(FieldAccess {
                receiver: Box::new(Expr::Identifier("player".to_string())),
                field: "stats".to_string(),
            })),
            field: "health".to_string(),
        });

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "player.stats.health");
    }

    /* ------------------------- Tests: IndexAccess ------------------------- */

    #[test]
    fn test_index_access_with_identifier() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: An index access expression.
        let expr = Expr::IndexAccess(IndexAccess {
            receiver: Box::new(Expr::Identifier("items".to_string())),
            index: Box::new(Expr::Identifier("_key".to_string())),
        });

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "items[_key]");
    }

    #[test]
    fn test_index_access_with_literal() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: An index access expression with literal index.
        let expr = Expr::IndexAccess(IndexAccess {
            receiver: Box::new(Expr::Identifier("array".to_string())),
            index: Box::new(Expr::Literal(Literal::Int(5))),
        });

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "array[5]");
    }

    /* --------------------------- Tests: Literal --------------------------- */

    #[test]
    fn test_literal_bool_true() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A boolean literal.
        let expr = Expr::Literal(Literal::Bool(true));

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "true");
    }

    #[test]
    fn test_literal_bool_false() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A boolean literal.
        let expr = Expr::Literal(Literal::Bool(false));

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "false");
    }

    #[test]
    fn test_literal_int() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: An integer literal.
        let expr = Expr::Literal(Literal::Int(42));

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "42");
    }

    #[test]
    fn test_literal_float() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A float literal.
        let expr = Expr::Literal(Literal::Float(3.5));

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "3.5");
    }

    #[test]
    fn test_literal_string() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A string literal.
        let expr = Expr::Literal(Literal::String("hello".to_string()));

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "\"hello\"");
    }

    #[test]
    fn test_literal_empty_array() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: An empty array literal.
        let expr = Expr::Literal(Literal::Array(vec![]));

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "[]");
    }

    #[test]
    fn test_literal_array_with_elements() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: An array literal with elements.
        let expr = Expr::Literal(Literal::Array(vec![
            Expr::Literal(Literal::Int(1)),
            Expr::Literal(Literal::Int(2)),
            Expr::Literal(Literal::Int(3)),
        ]));

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "[1, 2, 3]");
    }

    #[test]
    fn test_literal_empty_dict() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: An empty dict literal.
        let expr = Expr::Literal(Literal::Dict(vec![]));

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "{}");
    }

    #[test]
    fn test_literal_dict_with_entries() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A dict literal with entries.
        let expr = Expr::Literal(Literal::Dict(vec![
            (
                Expr::Literal(Literal::String("name".to_string())),
                Expr::Literal(Literal::String("John".to_string())),
            ),
            (
                Expr::Literal(Literal::String("age".to_string())),
                Expr::Literal(Literal::Int(30)),
            ),
        ]));

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "{\"name\": \"John\", \"age\": 30}");
    }

    /* -------------------------- Tests: BinaryOp --------------------------- */

    #[test]
    fn test_binary_op_equality() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A binary operation with equality operator.
        let expr = Expr::binary_op(Expr::ident("x"), Operator::Eq, Expr::Literal(5.into()));

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "x == 5");
    }

    #[test]
    fn test_binary_op_inequality() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A binary operation with inequality operator.
        let expr = Expr::binary_op(
            Expr::field(Expr::ident("_reader"), "get_error"),
            Operator::NotEq,
            Expr::ident("OK"),
        );

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "_reader.get_error != OK");
    }

    #[test]
    fn test_binary_op_with_method_call() {
        // Given: A string to write to.
        let mut s = StringWriter::default();

        // Given: A code writer to write with.
        let mut cw = GDScript::writer();

        // Given: A binary operation with method call.
        let expr = Expr::binary_op(
            FnCall::method(Expr::ident("_reader"), "get_error"),
            Operator::NotEq,
            Expr::ident("OK"),
        );

        // When: The expression is serialized to source code.
        let result = expr.emit(&mut cw, &mut s);

        // Then: There was no error.
        assert!(result.is_ok());

        // Then: The output matches expectations.
        assert_eq!(s.into_content(), "_reader.get_error() != OK");
    }
}
