use std::path::PathBuf;

use crate::gdscript::GDScript;

/* -------------------------------------------------------------------------- */
/*                                Struct: Args                                */
/* -------------------------------------------------------------------------- */

#[derive(clap::Args, Debug)]
pub struct Args {
    /// A path to a directory in which to generate GDScript files.
    #[arg(short, long, value_name = "OUT_DIR")]
    pub out: Option<PathBuf>,

    /// A root directory to search for imported '.baproto' files. Can be
    /// specified multiple times. Imports are resolved by searching each root in
    /// order. If not specified, defaults to the current working directory.
    #[arg(short = 'I', long = "import_root", value_name = "DIR")]
    pub import_roots: Vec<PathBuf>,

    /// A path to a message definition file to compile.
    #[arg(value_name = "FILES", required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,
}

/* -------------------------------------------------------------------------- */
/*                              Function: handle                              */
/* -------------------------------------------------------------------------- */

/// [`handle`] implements the `generate` command, which compiles a list of
/// `.baproto` schema files into a set of GDScript files rooted at the specified
/// `args.out` directory.
#[allow(unused)]
pub fn handle(args: Args) -> anyhow::Result<()> {
    let generator = GDScript::default();
    baproto::compile(args.files, args.import_roots, args.out, generator)
}
