mod file_handler;
mod project;

use clap::{Parser, Subcommand};
use project::{node_js::NodeProject, rust::RustProject, scan_project_deps};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(visible_alias = "a")]
    All,
    #[clap(visible_alias = "node")]
    NodeJS,
    Rust,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::All) => scan_project_deps(NodeProject::new()),
        Some(Commands::NodeJS) => scan_project_deps(NodeProject::new()),
        Some(Commands::Rust) => scan_project_deps(RustProject::new()),
        None => Err("Command doesn't exists".to_string()),
    };

    if let Ok(r) = result {
        r.print_result();
    }
}
