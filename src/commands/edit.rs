use crate::commands::Cli;
use crate::error::{FnoxError, Result};
use clap::Args;
use std::env;
use std::process::Command;

#[derive(Debug, Args)]
pub struct EditCommand {}

impl EditCommand {
    pub async fn run(&self, cli: &Cli, _config: crate::config::Config) -> Result<()> {
        tracing::debug!("Opening config file in editor");

        let editor = env::var("EDITOR")
            .or_else(|_| env::var("VISUAL"))
            .unwrap_or_else(|_| {
                // Default editors based on platform
                if cfg!(target_os = "windows") {
                    "notepad".to_string()
                } else {
                    "vi".to_string()
                }
            });

        tracing::debug!("Using editor: {}", editor);

        let status = Command::new(&editor)
            .arg(&cli.config)
            .status()
            .map_err(|e| FnoxError::EditorLaunchFailed {
                editor: editor.clone(),
                source: e,
            })?;

        if !status.success()
            && let Some(code) = status.code()
        {
            return Err(FnoxError::EditorExitFailed {
                editor: editor.clone(),
                status: code,
            });
        }

        let check = console::style("✓").green();
        let styled_config = console::style(cli.config.display()).cyan();
        println!("{check} Configuration file {styled_config} updated");

        Ok(())
    }
}
