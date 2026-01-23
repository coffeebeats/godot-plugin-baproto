use baproto::CodeWriter;
use baproto::Writer;
use derive_builder::Builder;

use super::Emit;
use super::Stmt;

/* -------------------------------------------------------------------------- */
/*                              Struct: FuncDecl                              */
/* -------------------------------------------------------------------------- */

/// `FuncDecl` represents a function declaration.
///
/// NOTE: `body` is `Vec<Stmt>` (not `Vec<Item>`) since function bodies contain
/// only statements, not nested function declarations. This also breaks the
/// [`Stmt`] to [`FuncDecl`] cycle.
#[allow(dead_code)]
#[derive(Clone, Debug, Builder)]
#[builder(setter(into))]
pub struct FuncDecl {
    pub name: String,
    #[builder(default)]
    pub params: Vec<FuncParam>,
    #[builder(default, setter(into, strip_option))]
    pub return_type: Option<String>,
    #[builder(default, setter(into, strip_option))]
    pub doc: Option<String>,
    #[builder(default)]
    pub body: Vec<Stmt>,
}

/* ---------------------------- Struct: FuncParam --------------------------- */

/// `FuncParam` represents a function parameter.
#[allow(dead_code)]
#[derive(Clone, Debug, Builder)]
#[builder(setter(into))]
pub struct FuncParam {
    pub name: String,
    #[builder(default, setter(into, strip_option))]
    pub type_hint: Option<String>,
    #[builder(default, setter(into, strip_option))]
    pub default_value: Option<String>,
}

/* ------------------------------- Impl: Emit ------------------------------- */

impl Emit for FuncDecl {
    fn emit<W: Writer>(&self, cw: &mut CodeWriter, w: &mut W) -> anyhow::Result<()> {
        // Emit doc comment if present.
        if let Some(doc_text) = &self.doc {
            cw.comment_block(w, doc_text)?;
        }

        // Build function signature: `func name(params) -> ReturnType:`
        let mut sig = format!("func {}", self.name);

        // Add parameters.
        sig.push('(');
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                sig.push_str(", ");
            }
            let mut param_str = param.name.clone();
            if let Some(hint) = &param.type_hint {
                param_str.push_str(&format!(": {}", hint));
            }
            if let Some(default_val) = &param.default_value {
                param_str.push_str(&format!(" = {}", default_val));
            }
            sig.push_str(&param_str);
        }
        sig.push(')');

        // Add return type.
        if let Some(ret) = &self.return_type {
            sig.push_str(&format!(" -> {}", ret));
        }
        sig.push(':');

        cw.writeln(w, &sig)?;

        // Emit body.
        cw.indent();
        if self.body.is_empty() {
            cw.writeln(w, "pass")?;
        } else {
            for stmt in &self.body {
                stmt.emit(cw, w)?;
            }
        }
        cw.outdent();

        Ok(())
    }
}
