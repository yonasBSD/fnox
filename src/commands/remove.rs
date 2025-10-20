use crate::commands::Cli;
use crate::config::Config;
use crate::error::Result;
use clap::Args;

#[derive(Debug, Args)]
#[command(visible_aliases = ["rm", "delete"])]
pub struct RemoveCommand {
    /// Secret key to remove
    pub key: String,
}

impl RemoveCommand {
    pub async fn run(&self, cli: &Cli, mut config: Config) -> Result<()> {
        let profile = Config::get_profile(cli.profile.as_deref());
        tracing::debug!("Removing secret '{}' from profile '{}'", self.key, profile);

        // Get the profile secrets
        let profile_secrets = config.get_secrets_mut(&profile);

        if profile_secrets.shift_remove(&self.key).is_some() {
            config.save(&cli.config)?;
            let check = console::style("✓").green();
            let styled_key = console::style(&self.key).cyan();
            let styled_profile = console::style(&profile).magenta();
            if profile == "default" {
                println!("{check} Removed secret {styled_key}");
            } else {
                println!("{check} Removed secret {styled_key} from profile {styled_profile}");
            }
        } else {
            Err(miette::miette!(
                "Secret '{}' not found in profile '{}'",
                self.key,
                profile
            ))?;
        }

        Ok(())
    }
}
