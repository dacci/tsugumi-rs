mod build;
mod new;

use anyhow::Result;
use clap::Parser;

#[derive(clap::Parser)]
#[command(about, version)]
enum Args {
    /// Create a new book.
    New(new::Args),

    /// Build the current book.
    Build(build::Args),
}

pub fn main() -> Result<()> {
    match Args::parse() {
        Args::New(args) => new::main(args),
        Args::Build(args) => build::main(args),
    }
}
