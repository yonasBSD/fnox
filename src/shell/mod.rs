use std::fmt;

mod bash;
mod fish;
mod zsh;

pub use bash::Bash;
pub use fish::Fish;
pub use zsh::Zsh;

/// Options for shell activation
#[derive(Debug, Clone)]
pub struct ActivateOptions {
    /// Path to the fnox executable
    pub exe: std::path::PathBuf,
    /// Additional shell-specific options
    pub no_hook_env: bool,
}

/// Trait for shell-specific implementations
pub trait Shell: fmt::Display + Send + Sync {
    /// Generate activation script for this shell
    fn activate(&self, opts: ActivateOptions) -> String;

    /// Generate deactivation script for this shell
    fn deactivate(&self) -> String;

    /// Generate code to set an environment variable
    fn set_env(&self, key: &str, value: &str) -> String;

    /// Generate code to unset an environment variable
    fn unset_env(&self, key: &str) -> String;
}

/// Parse shell name into Shell implementation
/// If name is None, detect shell from environment
pub fn get_shell(name: Option<&str>) -> anyhow::Result<Box<dyn Shell>> {
    let shell_name = match name {
        Some(n) => n.to_string(),
        None => detect_shell().ok_or_else(|| anyhow::anyhow!("Could not detect shell"))?,
    };

    match shell_name.as_str() {
        "bash" => Ok(Box::new(Bash)),
        "zsh" => Ok(Box::new(Zsh)),
        "fish" => Ok(Box::new(Fish)),
        _ => anyhow::bail!("unsupported shell: {}", shell_name),
    }
}

/// Detect current shell from environment
pub fn detect_shell() -> Option<String> {
    if let Ok(shell) = std::env::var("FNOX_SHELL") {
        return Some(shell);
    }

    std::env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(String::from))
}
