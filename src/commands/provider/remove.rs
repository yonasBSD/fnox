use crate::commands::Cli;
use crate::config::Config;
use crate::error::{FnoxError, Result};
use clap::Args;

#[derive(Debug, Args)]
#[command(visible_aliases = ["rm", "delete"])]
pub struct RemoveCommand {
    /// Provider name
    pub provider: String,
}

impl RemoveCommand {
    pub async fn run(&self, cli: &Cli, mut config: Config) -> Result<()> {
        tracing::debug!("Removing provider '{}'", self.provider);

        if config.providers.shift_remove(&self.provider).is_some() {
            config.save(&cli.config)?;
            println!("✓ Removed provider '{}'", self.provider);
        } else {
            return Err(FnoxError::Config(format!(
                "Provider '{}' not found",
                self.provider
            )));
        }

        Ok(())
    }
}
