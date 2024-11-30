mod build;
mod new;

use anyhow::Result;
use clap::{CommandFactory, Parser};

#[derive(clap::Parser)]
#[command(about, version)]
struct Args {
    #[clap(subcommand)]
    task: Option<Task>,

    /// Generate shell completions.
    #[arg(long, value_name = "SHELL", exclusive = true)]
    generate_completion: Option<clap_complete::aot::Shell>,
}

#[derive(clap::Subcommand)]
enum Task {
    /// Create a new book.
    New(new::Args),

    /// Build the current book.
    Build(build::Args),
}

pub fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(task) = args.task {
        return match task {
            Task::New(args) => new::main(args),
            Task::Build(args) => build::main(args),
        };
    }

    let mut cmd = Args::command();

    if let Some(shell) = args.generate_completion {
        clap_complete::generate(
            shell,
            &mut cmd,
            env!("CARGO_BIN_NAME"),
            &mut std::io::stdout(),
        );
        return Ok(());
    }

    cmd.print_help()?;
    Ok(())
}
