use crate::error::{DepotError, Result};
use crate::package::VersionMetadata;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    #[serde(default)]
    pub block_unlicensed: bool,
    #[serde(default = "default_max_severity")]
    pub max_vuln_severity: String,
    #[serde(default)]
    pub allowed_licenses: Vec<String>,
    #[serde(default)]
    pub blocked_packages: Vec<String>,
}

fn default_max_severity() -> String {
    "critical".into()
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            block_unlicensed: false,
            max_vuln_severity: default_max_severity(),
            allowed_licenses: Vec::new(),
            blocked_packages: Vec::new(),
        }
    }
}

impl PolicyConfig {
    /// Check a package version against configured policies.
    pub fn check(&self, metadata: &VersionMetadata) -> Result<()> {
        // Block explicitly blocked packages
        if self
            .blocked_packages
            .iter()
            .any(|b| b == metadata.name.as_str())
        {
            return Err(DepotError::PolicyViolation(format!(
                "package {} is blocked",
                metadata.name
            )));
        }

        // Block unlicensed if configured
        if self.block_unlicensed && metadata.license.is_none() {
            return Err(DepotError::PolicyViolation(format!(
                "package {} has no license",
                metadata.name
            )));
        }

        // Check allowed licenses if configured
        if !self.allowed_licenses.is_empty()
            && let Some(license) = &metadata.license
            && !self.allowed_licenses.iter().any(|a| a == license)
        {
            return Err(DepotError::PolicyViolation(format!(
                "package {} has license {license}, which is not in allowed list",
                metadata.name
            )));
        }

        Ok(())
    }
}
