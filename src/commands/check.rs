use crate::config::Config;
use crate::error::Result;
use clap::Args;
use tracing::{error, info, warn};

use crate::commands::Cli;

#[derive(Debug, Args)]
#[command(visible_alias = "c")]
pub struct CheckCommand {}

impl CheckCommand {
    pub async fn run(&self, cli: &Cli, config: Config) -> Result<()> {
        config.validate()?;
        let profile = Config::get_profile(cli.profile.as_deref());

        // Load config
        info!("Checking configuration for profile: {}", profile);

        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check secrets
        if let Ok(secrets) = config.get_secrets(&profile) {
            if secrets.is_empty() {
                warnings.push("No secrets defined in profile".to_string());
            } else {
                info!("Found {} secret(s) in profile", secrets.len());

                for (name, secret_config) in secrets {
                    // Check if secret has a value source
                    if !secret_config.has_value() {
                        match secret_config.if_missing {
                            Some(crate::config::IfMissing::Error) => {
                                issues.push(format!(
                                    "Secret '{}' is required but has no value source",
                                    name
                                ));
                            }
                            Some(crate::config::IfMissing::Warn) => {
                                warnings.push(format!("Secret '{}' has no value source", name));
                            }
                            _ => {
                                // Ignore is fine
                            }
                        }
                    }

                    // Check provider configuration
                    if let Some(provider) = &secret_config.provider {
                        let providers = config.get_providers(&profile);
                        if !providers.contains_key(provider) {
                            warnings.push(format!(
                                "Secret '{}' references unknown provider '{}'",
                                name, provider
                            ));
                        }
                    }
                }
            }
        } else {
            issues.push(format!("Profile '{}' not found", profile));
        }

        // Check providers
        let providers = config.get_providers(&profile);
        if providers.is_empty() {
            warnings.push("No providers configured".to_string());
        } else {
            info!("Found {} provider(s) in profile", providers.len());
        }

        // Report results
        if !issues.is_empty() {
            error!("Found {} error(s):", issues.len());
            for issue in &issues {
                error!("  {}", issue);
            }
        }

        if !warnings.is_empty() {
            warn!("Found {} warning(s):", warnings.len());
            for warning in &warnings {
                warn!("  {}", warning);
            }
        }

        if issues.is_empty() && warnings.is_empty() {
            info!("✓ Configuration is healthy");
        } else if issues.is_empty() {
            info!("✓ Configuration is OK (with warnings)");
        }

        if !issues.is_empty() {
            std::process::exit(1);
        }

        Ok(())
    }
}
