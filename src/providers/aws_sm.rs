use crate::error::{FnoxError, Result};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_secretsmanager::Client;
use std::path::Path;

pub struct AwsSecretsManagerProvider {
    region: String,
    prefix: Option<String>,
}

impl AwsSecretsManagerProvider {
    pub fn new(region: String, prefix: Option<String>) -> Self {
        Self { region, prefix }
    }

    pub fn get_secret_name(&self, key: &str) -> String {
        match &self.prefix {
            Some(prefix) => format!("{}{}", prefix, key),
            None => key.to_string(),
        }
    }

    /// Create an AWS Secrets Manager client
    async fn create_client(&self) -> Result<Client> {
        // Load AWS config with the specified region
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_sdk_secretsmanager::config::Region::new(
                self.region.clone(),
            ))
            .load()
            .await;

        Ok(Client::new(&config))
    }

    /// Get a secret value from AWS Secrets Manager
    async fn get_secret_value(&self, secret_name: &str) -> Result<String> {
        let client = self.create_client().await?;

        let result = client
            .get_secret_value()
            .secret_id(secret_name)
            .send()
            .await
            .map_err(|e| {
                FnoxError::Provider(format!(
                    "Failed to get secret '{}' from AWS Secrets Manager: {}",
                    secret_name, e
                ))
            })?;

        // Get the secret string (not binary)
        result
            .secret_string()
            .ok_or_else(|| {
                FnoxError::Provider(format!(
                    "Secret '{}' has no string value (binary secrets not supported)",
                    secret_name
                ))
            })
            .map(|s| s.to_string())
    }

    /// Create or update a secret in AWS Secrets Manager
    pub async fn put_secret(&self, secret_name: &str, secret_value: &str) -> Result<()> {
        let client = self.create_client().await?;

        // Try to update existing secret first
        match client
            .put_secret_value()
            .secret_id(secret_name)
            .secret_string(secret_value)
            .send()
            .await
        {
            Ok(_) => {
                tracing::debug!("Updated secret '{}' in AWS Secrets Manager", secret_name);
                Ok(())
            }
            Err(e) => {
                // If secret doesn't exist, create it
                if e.to_string().contains("ResourceNotFoundException") {
                    client
                        .create_secret()
                        .name(secret_name)
                        .secret_string(secret_value)
                        .send()
                        .await
                        .map_err(|e| {
                            FnoxError::Provider(format!(
                                "Failed to create secret '{}' in AWS Secrets Manager: {}",
                                secret_name, e
                            ))
                        })?;
                    tracing::debug!("Created secret '{}' in AWS Secrets Manager", secret_name);
                    Ok(())
                } else {
                    Err(FnoxError::Provider(format!(
                        "Failed to update secret '{}' in AWS Secrets Manager: {}",
                        secret_name, e
                    )))
                }
            }
        }
    }
}

#[async_trait]
impl crate::providers::Provider for AwsSecretsManagerProvider {
    fn capabilities(&self) -> Vec<crate::providers::ProviderCapability> {
        vec![crate::providers::ProviderCapability::RemoteStorage]
    }

    async fn get_secret(&self, value: &str, _key_file: Option<&Path>) -> Result<String> {
        let secret_name = self.get_secret_name(value);
        tracing::debug!(
            "Getting secret '{}' from AWS Secrets Manager in region '{}'",
            secret_name,
            self.region
        );

        self.get_secret_value(&secret_name).await
    }

    async fn test_connection(&self) -> Result<()> {
        let client = self.create_client().await?;

        // Try to list secrets to verify connection
        client
            .list_secrets()
            .max_results(1)
            .send()
            .await
            .map_err(|e| {
                FnoxError::Provider(format!(
                    "Failed to connect to AWS Secrets Manager in region '{}': {}",
                    self.region, e
                ))
            })?;

        Ok(())
    }
}
