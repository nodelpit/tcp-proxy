pub mod cli;
use anyhow::Result;

pub async fn run(_config: cli::Config) -> Result<()> {
    Ok(())
}
