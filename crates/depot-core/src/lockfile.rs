use serde::{Deserialize, Serialize};

use crate::package::Ecosystem;

/// The depot lock file — ecosystem-agnostic, blake3-verified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    pub metadata: LockMetadata,
    #[serde(default)]
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockMetadata {
    pub schema_version: u32,
    pub generated_at: String,
    pub depot_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    pub ecosystem: Ecosystem,
    pub name: String,
    pub version: String,
    pub artifacts: Vec<LockedArtifact>,
    pub resolved_from: String,
    #[serde(default)]
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedArtifact {
    pub filename: String,
    pub blake3: String,
    pub size: u64,
}

impl LockFile {
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}
