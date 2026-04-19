use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{DepotError, Result};
use crate::policy::PolicyConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub upstream: HashMap<String, UpstreamConfig>,
    #[serde(default)]
    pub policies: PolicyConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub encryption: EncryptionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_bind")]
    pub bind: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: default_bind(),
        }
    }
}

fn default_bind() -> String {
    "0.0.0.0:8080".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_backend")]
    pub backend: String,
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub s3: Option<S3Config>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            path: None,
            s3: None,
        }
    }
}

fn default_backend() -> String {
    "fs".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub url: String,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub tokens: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EncryptionConfig {
    #[serde(default)]
    pub enabled: bool,
    pub key_file: Option<PathBuf>,
}

impl Config {
    /// Load config from a specific path.
    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(DepotError::ConfigNotFound(path.to_path_buf()));
        }
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Load config with default lookup chain:
    /// 1. `DEPOT_CONFIG` env var
    /// 2. `./depot.toml` in current directory
    /// 3. Defaults
    pub fn load() -> Result<Self> {
        if let Ok(path) = std::env::var("DEPOT_CONFIG") {
            let p = PathBuf::from(path);
            if p.exists() {
                return Self::load_from(&p);
            }
        }

        let local = PathBuf::from("depot.toml");
        if local.exists() {
            return Self::load_from(&local);
        }

        Ok(Self::default())
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut upstream = HashMap::new();
        upstream.insert(
            "pypi".into(),
            UpstreamConfig {
                enabled: true,
                url: "https://pypi.org".into(),
            },
        );
        upstream.insert(
            "npm".into(),
            UpstreamConfig {
                enabled: true,
                url: "https://registry.npmjs.org".into(),
            },
        );
        upstream.insert(
            "cargo".into(),
            UpstreamConfig {
                enabled: true,
                url: "https://index.crates.io".into(),
            },
        );
        upstream.insert(
            "hex".into(),
            UpstreamConfig {
                enabled: true,
                url: "https://hex.pm".into(),
            },
        );

        Self {
            server: ServerConfig::default(),
            storage: StorageConfig::default(),
            upstream,
            policies: PolicyConfig::default(),
            auth: AuthConfig::default(),
            encryption: EncryptionConfig::default(),
        }
    }
}
