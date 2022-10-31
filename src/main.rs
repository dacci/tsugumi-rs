mod task;

use anyhow::{Context as _, Result};

fn main() -> Result<()> {
    use tracing_subscriber::prelude::*;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env()
                .context("failed to initialize tracing")
                .unwrap(),
        )
        .init();

    task::main()
}
