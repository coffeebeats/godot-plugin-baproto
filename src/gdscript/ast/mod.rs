use baproto::{CodeWriter, Writer};
use derive_builder::Builder;

/* ------------------------------- Mod: Config ------------------------------ */

pub mod config;

/* ------------------------------ Mod: Function ----------------------------- */

mod function;
pub use function::*;

/* -------------------------------------------------------------------------- */
/*                                Trait: Emit                                 */
/* -------------------------------------------------------------------------- */

/// `Emit` writes a GDScript construct to a `CodeWriter`.
#[allow(dead_code)]
pub trait Emit {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()>;
}

/* -------------------------------------------------------------------------- */
/*                                 Enum: Stmt                                 */
/* -------------------------------------------------------------------------- */

/// `Stmt` represents a GDScript statement or declaration.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Stmt {
    /// Raw line of code.
    Line(String),

    /// Comment line: `# text`
    Comment(String),

    /// Blank line.
    Blank,

    /// Constant: `const NAME: Type = value` or `const NAME := value`
    Const {
        name: String,
        type_hint: Option<String>,
        value: String,
        doc: Option<String>,
    },

    /// Preload: `const NAME := preload("path")`
    Preload { name: String, path: String },

    /// Variable: `var name: Type = value`
    Var {
        name: String,
        type_hint: Option<String>,
        value: Option<String>,
        doc: Option<String>,
    },

    /// For-in loop with body.
    ForIn {
        var_name: String,
        iterable: String,
        body: Vec<Stmt>,
    },

    /// If statement with optional else.
    If {
        condition: String,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },

    /// Return statement.
    Return(Option<String>),

    /// Pass statement.
    Pass,

    /// Assignment: `target = value`
    Assign { target: String, value: String },

    /// Expression statement (method call, etc.).
    Expr(String),
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for Stmt {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        match self {
            Stmt::Line(line) => cw.writeln(w, line)?,
            Stmt::Comment(text) => cw.comment(w, text)?,
            Stmt::Blank => cw.blank_line(w)?,
            Stmt::Const {
                name,
                type_hint,
                value,
                doc,
            } => {
                if let Some(doc_text) = doc {
                    cw.comment_block(w, doc_text)?;
                }
                if let Some(hint) = type_hint {
                    cw.writeln(w, &format!("const {}: {} = {}", name, hint, value))?;
                } else {
                    cw.writeln(w, &format!("const {} := {}", name, value))?;
                }
            }
            Stmt::Preload { name, path } => {
                cw.writeln(w, &format!("const {} := preload(\"{}\")", name, path))?;
            }
            Stmt::Var {
                name,
                type_hint,
                value,
                doc,
            } => {
                if let Some(doc_text) = doc {
                    cw.comment_block(w, doc_text)?;
                }
                match (type_hint, value) {
                    (Some(hint), Some(val)) => {
                        cw.writeln(w, &format!("var {}: {} = {}", name, hint, val))?;
                    }
                    (Some(hint), None) => {
                        cw.writeln(w, &format!("var {}: {}", name, hint))?;
                    }
                    (None, Some(val)) => {
                        cw.writeln(w, &format!("var {} = {}", name, val))?;
                    }
                    (None, None) => {
                        cw.writeln(w, &format!("var {}", name))?;
                    }
                }
            }
            Stmt::ForIn {
                var_name,
                iterable,
                body,
            } => {
                cw.writeln(w, &format!("for {} in {}:", var_name, iterable))?;
                cw.indent();
                if body.is_empty() {
                    cw.writeln(w, "pass")?;
                } else {
                    for stmt in body {
                        stmt.emit(cw, w)?;
                    }
                }
                cw.outdent();
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                cw.writeln(w, &format!("if {}:", condition))?;
                cw.indent();
                if then_body.is_empty() {
                    cw.writeln(w, "pass")?;
                } else {
                    for stmt in then_body {
                        stmt.emit(cw, w)?;
                    }
                }
                cw.outdent();
                if let Some(else_stmts) = else_body {
                    cw.writeln(w, "else:")?;
                    cw.indent();
                    if else_stmts.is_empty() {
                        cw.writeln(w, "pass")?;
                    } else {
                        for stmt in else_stmts {
                            stmt.emit(cw, w)?;
                        }
                    }
                    cw.outdent();
                }
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    cw.writeln(w, &format!("return {}", e))?;
                } else {
                    cw.writeln(w, "return")?;
                }
            }
            Stmt::Pass => cw.writeln(w, "pass")?,
            Stmt::Assign { target, value } => {
                cw.writeln(w, &format!("{} = {}", target, value))?;
            }
            Stmt::Expr(expr) => cw.writeln(w, expr)?,
        }
        Ok(())
    }
}

/* -------------------------------------------------------------------------- */
/*                                 Enum: Item                                 */
/* -------------------------------------------------------------------------- */

/// `Item` represents a section item (statement or function).
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Item {
    /// A statement.
    Stmt(Stmt),
    /// A function declaration.
    Func(FuncDecl),
}

/* ---------------------------- Impl: From<Stmt> ---------------------------- */

impl From<Stmt> for Item {
    fn from(stmt: Stmt) -> Self {
        Item::Stmt(stmt)
    }
}

/* -------------------------- Impl: From<FuncDecl> -------------------------- */

impl From<FuncDecl> for Item {
    fn from(func: FuncDecl) -> Self {
        Item::Func(func)
    }
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for Item {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        match self {
            Item::Stmt(stmt) => stmt.emit(cw, w),
            Item::Func(func) => func.emit(cw, w),
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                               Struct: Section                              */
/* -------------------------------------------------------------------------- */

/// `Section` represents a labeled code section with header comment.
#[allow(dead_code)]
#[derive(Clone, Debug, Builder)]
#[builder(setter(into))]
pub struct Section {
    pub name: String,
    #[builder(default)]
    pub body: Vec<Item>,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for Section {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        // Emit section header.
        cw.comment(w, &config::format_section_header(&self.name))?;
        cw.blank_line(w)?;

        // Emit body items.
        for item in &self.body {
            item.emit(cw, w)?;
        }

        // Trailing blank line after section.
        cw.blank_line(w)?;

        Ok(())
    }
}

/* -------------------------------------------------------------------------- */
/*                               Struct: GDFile                               */
/* -------------------------------------------------------------------------- */

/// `GDFile` represents a complete GDScript file.
#[allow(dead_code)]
#[derive(Clone, Debug, Builder)]
#[builder(setter(into))]
pub struct GDFile {
    #[builder(default = "config::HEADER_COMMENT.into()")]
    pub header_comment: String,
    #[builder(default, setter(into, strip_option))]
    pub doc: Option<String>,
    pub extends: String,
    #[builder(default, setter(into, strip_option))]
    pub class_name: Option<String>,
    #[builder(default)]
    pub sections: Vec<Section>,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for GDFile {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        // Emit header comment.
        cw.comment(w, &self.header_comment)?;
        cw.blank_line(w)?;

        // Emit doc comment if present.
        if let Some(doc_text) = &self.doc {
            cw.comment_block(w, doc_text)?;
        }

        // Emit extends.
        cw.writeln(w, &format!("extends {}", self.extends))?;

        // Emit class_name if present.
        if let Some(name) = &self.class_name {
            cw.writeln(w, &format!("class_name {}", name))?;
        }

        // Blank line before sections.
        cw.blank_line(w)?;

        // Emit sections.
        for section in &self.sections {
            section.emit(cw, w)?;
        }

        Ok(())
    }
}

/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gdscript::tests::create_code_writer;
    use baproto::StringWriter;

    /* ------------------------- Tests: Stmt::Const ------------------------- */

    #[test]
    fn test_stmt_const_emits_with_type_hint() {
        // Given: A const statement with type hint.
        let stmt = Stmt::Const {
            name: "MAX_HEALTH".into(),
            type_hint: Some("int".into()),
            value: "100".into(),
            doc: None,
        };

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        stmt.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit const with type hint.
        let result = w.into_content();
        assert_eq!(result, "const MAX_HEALTH: int = 100\n");
    }

    #[test]
    fn test_stmt_const_emits_without_type_hint() {
        // Given: A const statement without type hint.
        let stmt = Stmt::Const {
            name: "PI".into(),
            type_hint: None,
            value: "3.14".into(),
            doc: None,
        };

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        stmt.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit const with := operator.
        let result = w.into_content();
        assert_eq!(result, "const PI := 3.14\n");
    }

    #[test]
    fn test_stmt_const_emits_with_doc() {
        // Given: A const statement with doc comment.
        let stmt = Stmt::Const {
            name: "SPEED".into(),
            type_hint: Some("float".into()),
            value: "5.0".into(),
            doc: Some("Maximum speed in m/s.".into()),
        };

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        stmt.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit doc comment before const.
        let result = w.into_content();
        assert!(result.contains("## Maximum speed in m/s."));
        assert!(result.contains("const SPEED: float = 5.0"));
    }

    /* ---------------------------- Tests: Stmt::Var ---------------------------- */

    #[test]
    fn test_stmt_var_emits_with_default() {
        // Given: A var statement with type hint and default.
        let stmt = Stmt::Var {
            name: "health".into(),
            type_hint: Some("int".into()),
            value: Some("100".into()),
            doc: None,
        };

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        stmt.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit var with type and default.
        let result = w.into_content();
        assert_eq!(result, "var health: int = 100\n");
    }

    #[test]
    fn test_stmt_var_emits_without_default() {
        // Given: A var statement with type hint but no default.
        let stmt = Stmt::Var {
            name: "name".into(),
            type_hint: Some("String".into()),
            value: None,
            doc: None,
        };

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        stmt.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit var with type only.
        let result = w.into_content();
        assert_eq!(result, "var name: String\n");
    }

    /* --------------------------- Tests: Stmt::ForIn --------------------------- */

    #[test]
    fn test_stmt_for_in_emits_with_indented_body() {
        // Given: A for-in loop with body.
        let stmt = Stmt::ForIn {
            var_name: "item".into(),
            iterable: "items".into(),
            body: vec![Stmt::Expr("print(item)".into())],
        };

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        stmt.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit for-in with indented body.
        let result = w.into_content();
        assert!(result.contains("for item in items:"));
        assert!(result.contains("\tprint(item)"));
    }

    #[test]
    fn test_stmt_for_in_emits_empty_body_as_pass() {
        // Given: A for-in loop with empty body.
        let stmt = Stmt::ForIn {
            var_name: "item".into(),
            iterable: "items".into(),
            body: vec![],
        };

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        stmt.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit pass statement.
        let result = w.into_content();
        assert!(result.contains("for item in items:"));
        assert!(result.contains("\tpass"));
    }

    /* ---------------------------- Tests: Stmt::If ----------------------------- */

    #[test]
    fn test_stmt_if_emits_without_else() {
        // Given: An if statement without else.
        let stmt = Stmt::If {
            condition: "health > 0".into(),
            then_body: vec![Stmt::Expr("print(\"alive\")".into())],
            else_body: None,
        };

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        stmt.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit if without else.
        let result = w.into_content();
        assert!(result.contains("if health > 0:"));
        assert!(result.contains("\tprint(\"alive\")"));
        assert!(!result.contains("else:"));
    }

    #[test]
    fn test_stmt_if_emits_with_else() {
        // Given: An if statement with else.
        let stmt = Stmt::If {
            condition: "health > 0".into(),
            then_body: vec![Stmt::Expr("print(\"alive\")".into())],
            else_body: Some(vec![Stmt::Expr("print(\"dead\")".into())]),
        };

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        stmt.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit if with else.
        let result = w.into_content();
        assert!(result.contains("if health > 0:"));
        assert!(result.contains("\tprint(\"alive\")"));
        assert!(result.contains("else:"));
        assert!(result.contains("\tprint(\"dead\")"));
    }

    /* ---------------------------- Tests: Item --------------------------------- */

    #[test]
    fn test_item_from_stmt() {
        // Given: A statement.
        let stmt = Stmt::Pass;

        // When: Converting to Item.
        let item: Item = stmt.into();

        // Then: Should be Item::Stmt variant.
        matches!(item, Item::Stmt(Stmt::Pass));
    }

    #[test]
    fn test_item_from_func() {
        // Given: A function declaration.
        let func = FuncDeclBuilder::default().name("test").build().unwrap();

        // When: Converting to Item.
        let item: Item = func.into();

        // Then: Should be Item::Func variant.
        matches!(item, Item::Func(_));
    }

    #[test]
    fn test_item_func_emits_with_params_and_return() {
        // Given: A function with parameters and return type.
        let func = FuncDeclBuilder::default()
            .name("add")
            .params(vec![
                FuncParamBuilder::default()
                    .name("a")
                    .type_hint("int")
                    .build()
                    .unwrap(),
                FuncParamBuilder::default()
                    .name("b")
                    .type_hint("int")
                    .build()
                    .unwrap(),
            ])
            .return_type("int")
            .body(vec![Stmt::Return(Some("a + b".into()))])
            .build()
            .unwrap();

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        func.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit complete function signature and body.
        let result = w.into_content();
        assert!(result.contains("func add(a: int, b: int) -> int:"));
        assert!(result.contains("\treturn a + b"));
    }

    #[test]
    fn test_item_func_emits_empty_body_as_pass() {
        // Given: A function with no body.
        let func = FuncDeclBuilder::default().name("noop").build().unwrap();

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        func.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit pass statement.
        let result = w.into_content();
        assert!(result.contains("func noop():"));
        assert!(result.contains("\tpass"));
    }

    /* ---------------------------- Tests: Section ------------------------------ */

    #[test]
    fn test_section_emits_header_and_body() {
        // Given: A section with statements.
        let section = SectionBuilder::default()
            .name("Constants")
            .body(vec![
                Stmt::Const {
                    name: "MAX".into(),
                    type_hint: None,
                    value: "100".into(),
                    doc: None,
                }
                .into(),
            ])
            .build()
            .unwrap();

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        section.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit header and body.
        let result = w.into_content();
        assert!(result.contains("Constants"));
        assert!(result.contains("const MAX := 100"));
    }

    #[test]
    fn test_section_emits_mixed_items() {
        // Given: A section with statements and functions.
        let section = SectionBuilder::default()
            .name("Mixed")
            .body(vec![
                Stmt::Const {
                    name: "VALUE".into(),
                    type_hint: None,
                    value: "42".into(),
                    doc: None,
                }
                .into(),
                FuncDeclBuilder::default()
                    .name("get_value")
                    .return_type("int")
                    .body(vec![Stmt::Return(Some("VALUE".into()))])
                    .build()
                    .unwrap()
                    .into(),
            ])
            .build()
            .unwrap();

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        section.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit both const and function.
        let result = w.into_content();
        assert!(result.contains("const VALUE := 42"));
        assert!(result.contains("func get_value() -> int:"));
        assert!(result.contains("return VALUE"));
    }

    /* ---------------------------- Tests: GDFile ------------------------------- */

    #[test]
    fn test_gdfile_emits_complete_structure() {
        // Given: A complete GDFile.
        let file = GDFileBuilder::default()
            .extends("RefCounted")
            .doc("A test class.")
            .sections(vec![
                SectionBuilder::default()
                    .name("Fields")
                    .body(vec![
                        Stmt::Var {
                            name: "value".into(),
                            type_hint: Some("int".into()),
                            value: Some("0".into()),
                            doc: None,
                        }
                        .into(),
                    ])
                    .build()
                    .unwrap(),
            ])
            .build()
            .unwrap();

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        file.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit complete file structure.
        let result = w.into_content();
        assert!(result.contains("# DO NOT EDIT: Generated by baproto-gdscript"));
        assert!(result.contains("## A test class."));
        assert!(result.contains("extends RefCounted"));
        assert!(result.contains("Fields"));
        assert!(result.contains("var value: int = 0"));
    }

    #[test]
    fn test_gdfile_emits_with_class_name() {
        // Given: A GDFile with class_name.
        let file = GDFileBuilder::default()
            .extends("Node")
            .class_name("MyNode")
            .build()
            .unwrap();

        // When: Emitting to code writer.
        let mut cw = create_code_writer();
        let mut w = StringWriter::default();
        file.emit(&mut cw, &mut w).unwrap();

        // Then: Should emit class_name line.
        let result = w.into_content();
        assert!(result.contains("extends Node"));
        assert!(result.contains("class_name MyNode"));
    }
}
