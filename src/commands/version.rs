use crate::commands::Cli;
use crate::error::Result;
use clap::Parser;

#[derive(Parser)]
#[command(visible_aliases = ["v"])]
pub struct VersionCommand;

impl VersionCommand {
    pub async fn run(&self, _cli: &Cli) -> Result<()> {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        Ok(())
    }
}
