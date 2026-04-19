use serde::{Deserialize, Serialize};

/// Supported package ecosystems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ecosystem {
    PyPI,
    Npm,
    Cargo,
    Hex,
}

impl std::fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PyPI => write!(f, "pypi"),
            Self::Npm => write!(f, "npm"),
            Self::Cargo => write!(f, "cargo"),
            Self::Hex => write!(f, "hex"),
        }
    }
}

/// A normalized package name.
///
/// Canonicalizes names across ecosystems (e.g. underscores → hyphens for PyPI).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackageName(String);

impl PackageName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Normalize for a specific ecosystem.
    pub fn normalized(&self, ecosystem: Ecosystem) -> String {
        match ecosystem {
            Ecosystem::PyPI => self.0.to_lowercase().replace('_', "-"),
            _ => self.0.clone(),
        }
    }
}

impl std::fmt::Display for PackageName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Uniquely identifies an artifact in storage.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactId {
    pub ecosystem: Ecosystem,
    pub name: PackageName,
    pub version: String,
    pub filename: String,
}

impl ArtifactId {
    /// Storage key for this artifact: `<ecosystem>/<name>/<version>/<filename>`
    pub fn storage_key(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.ecosystem, self.name, self.version, self.filename
        )
    }
}

/// Metadata for a specific package version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMetadata {
    pub name: PackageName,
    pub version: String,
    pub artifacts: Vec<ArtifactDigest>,
    pub license: Option<String>,
    pub yanked: bool,
}

/// Summary info for a version (used in listings).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub yanked: bool,
}

/// Digest of a single artifact file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDigest {
    pub filename: String,
    pub blake3: String,
    pub size: u64,
}
