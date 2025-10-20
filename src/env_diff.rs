use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a change operation for an environment variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvDiffOperation {
    /// Set or update an environment variable
    Set(String, String),
    /// Remove an environment variable
    Remove(String),
}

/// Tracks changes between old and new environment states
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EnvDiff {
    /// Old environment state (from previous session)
    pub old: HashMap<String, String>,
    /// New environment state (secrets to load)
    pub new: HashMap<String, String>,
}

impl EnvDiff {
    /// Create a new EnvDiff from old and new secret maps
    pub fn new(old: HashMap<String, String>, new: HashMap<String, String>) -> Self {
        Self { old, new }
    }

    /// Calculate the operations needed to transform old -> new
    pub fn operations(&self) -> Vec<EnvDiffOperation> {
        let mut ops = Vec::new();

        // Find additions and changes
        for (key, new_value) in &self.new {
            match self.old.get(key) {
                Some(old_value) if old_value == new_value => {
                    // No change, skip
                }
                _ => {
                    // New or changed value
                    ops.push(EnvDiffOperation::Set(key.clone(), new_value.clone()));
                }
            }
        }

        // Find removals
        for key in self.old.keys() {
            if !self.new.contains_key(key) {
                ops.push(EnvDiffOperation::Remove(key.clone()));
            }
        }

        ops
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.operations().is_empty()
    }

    /// Serialize to base64-encoded msgpack for storage in __FNOX_DIFF
    pub fn encode(&self) -> Result<String> {
        let bytes = rmp_serde::to_vec(self)?;
        let compressed = miniz_oxide::deflate::compress_to_vec(&bytes, 6);
        Ok(data_encoding::BASE64.encode(&compressed))
    }

    /// Deserialize from base64-encoded msgpack
    #[allow(dead_code)]
    pub fn decode(encoded: &str) -> Result<Self> {
        let compressed = data_encoding::BASE64.decode(encoded.as_bytes())?;
        let bytes = miniz_oxide::inflate::decompress_to_vec(&compressed)
            .map_err(|e| anyhow::anyhow!("failed to decompress env diff: {:?}", e))?;
        let diff = rmp_serde::from_slice(&bytes)?;
        Ok(diff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_diff_no_changes() {
        let old = HashMap::from([("KEY1".to_string(), "value1".to_string())]);
        let new = HashMap::from([("KEY1".to_string(), "value1".to_string())]);
        let diff = EnvDiff::new(old, new);

        assert!(!diff.has_changes());
        assert_eq!(diff.operations().len(), 0);
    }

    #[test]
    fn test_env_diff_additions() {
        let old = HashMap::new();
        let new = HashMap::from([
            ("KEY1".to_string(), "value1".to_string()),
            ("KEY2".to_string(), "value2".to_string()),
        ]);
        let diff = EnvDiff::new(old, new);

        assert!(diff.has_changes());
        let ops = diff.operations();
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn test_env_diff_removals() {
        let old = HashMap::from([
            ("KEY1".to_string(), "value1".to_string()),
            ("KEY2".to_string(), "value2".to_string()),
        ]);
        let new = HashMap::new();
        let diff = EnvDiff::new(old, new);

        assert!(diff.has_changes());
        let ops = diff.operations();
        assert_eq!(ops.len(), 2);
        assert!(matches!(ops[0], EnvDiffOperation::Remove(_)));
    }

    #[test]
    fn test_env_diff_changes() {
        let old = HashMap::from([("KEY1".to_string(), "old_value".to_string())]);
        let new = HashMap::from([("KEY1".to_string(), "new_value".to_string())]);
        let diff = EnvDiff::new(old, new);

        assert!(diff.has_changes());
        let ops = diff.operations();
        assert_eq!(ops.len(), 1);
        assert!(matches!(
            &ops[0],
            EnvDiffOperation::Set(k, v) if k == "KEY1" && v == "new_value"
        ));
    }

    #[test]
    fn test_env_diff_encode_decode() {
        let old = HashMap::from([("KEY1".to_string(), "value1".to_string())]);
        let new = HashMap::from([
            ("KEY1".to_string(), "value1".to_string()),
            ("KEY2".to_string(), "value2".to_string()),
        ]);
        let diff = EnvDiff::new(old, new);

        let encoded = diff.encode().unwrap();
        let decoded = EnvDiff::decode(&encoded).unwrap();

        assert_eq!(diff.old, decoded.old);
        assert_eq!(diff.new, decoded.new);
    }
}
