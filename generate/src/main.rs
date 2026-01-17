mod cmd;
mod gdscript;

use anyhow::Result;
use clap::Parser;

use cmd::Commands;

#[derive(Parser)]
#[command(name = "baproto-gdscript", author, version, about)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// `quiet` silences all non-essential logging.
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    quiet: bool,

    /// `verbose` enables additional detailed logging.
    #[arg(short, long, global = true)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        /* ------------------------ Category: Generate ----------------------- */
        Commands::Generate(args) => cmd::generate::handle(args),
    }
}
