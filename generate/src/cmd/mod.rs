pub mod generate;

/* -------------------------------------------------------------------------- */
/*                               Enum: Commands                               */
/* -------------------------------------------------------------------------- */

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /* -------------------------- Category: Generate ------------------------ */
    /// Generate GDScript bindings from '.baproto' schema files.
    Generate(generate::Args),
}
