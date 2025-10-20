use crate::error::{FnoxError, Result};
use async_trait::async_trait;
use google_cloud_secretmanager_v1::client::SecretManagerService;
use std::path::Path;

pub struct GoogleSecretManagerProvider {
    project: String,
    prefix: Option<String>,
}

impl GoogleSecretManagerProvider {
    pub fn new(project: String, prefix: Option<String>) -> Self {
        Self { project, prefix }
    }

    /// Build the full secret name with optional prefix
    fn build_secret_name(&self, value: &str) -> String {
        let secret_name = if let Some(prefix) = &self.prefix {
            format!("{}{}", prefix, value)
        } else {
            value.to_string()
        };

        format!(
            "projects/{}/secrets/{}/versions/latest",
            self.project, secret_name
        )
    }

    /// Create a Secret Manager client
    async fn create_client(&self) -> Result<SecretManagerService> {
        SecretManagerService::builder().build().await.map_err(|e| {
            FnoxError::Provider(format!("Failed to create GCP Secret Manager client: {}", e))
        })
    }
}

#[async_trait]
impl crate::providers::Provider for GoogleSecretManagerProvider {
    fn capabilities(&self) -> Vec<crate::providers::ProviderCapability> {
        vec![crate::providers::ProviderCapability::RemoteStorage]
    }

    async fn get_secret(&self, value: &str, _key_file: Option<&Path>) -> Result<String> {
        let client = self.create_client().await?;
        let secret_name = self.build_secret_name(value);

        let response = client
            .access_secret_version()
            .set_name(secret_name)
            .send()
            .await
            .map_err(|e| {
                FnoxError::Provider(format!(
                    "Failed to access secret '{}' from GCP Secret Manager: {}",
                    value, e
                ))
            })?;

        // Extract the payload data
        let payload = response
            .payload
            .ok_or_else(|| FnoxError::Provider("Secret has no payload".to_string()))?;

        // Convert bytes to string
        String::from_utf8(payload.data.to_vec())
            .map_err(|e| FnoxError::Provider(format!("Secret value is not valid UTF-8: {}", e)))
    }

    async fn test_connection(&self) -> Result<()> {
        let client = self.create_client().await?;

        // Try to list secrets to verify access
        client
            .list_secrets()
            .set_parent(format!("projects/{}", self.project))
            .send()
            .await
            .map_err(|e| {
                FnoxError::Provider(format!(
                    "Failed to connect to GCP Secret Manager or access project '{}': {}",
                    self.project, e
                ))
            })?;

        Ok(())
    }
}
