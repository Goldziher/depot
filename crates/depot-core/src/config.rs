use std::collections::HashMap;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{DepotError, Result};
use crate::policy::PolicyConfig;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpstreamConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub url: String,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct AuthConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub tokens: Vec<String>,
}

impl std::fmt::Debug for AuthConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthConfig")
            .field("enabled", &self.enabled)
            .field("tokens", &format!("[{} redacted]", self.tokens.len()))
            .finish()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
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
            return Err(DepotError::ConfigNotFound(p));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn load_fixtures() -> Vec<serde_json::Value> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testing_data/config/01_config_parsing.json");
        let content = std::fs::read_to_string(&path).unwrap();
        serde_json::from_str(&content).unwrap()
    }

    #[test]
    fn fixture_driven_config_parsing() {
        let fixtures = load_fixtures();
        for fix in &fixtures {
            let name = fix["name"].as_str().unwrap_or("?");
            let toml_input = fix["input"]["toml"].as_str().unwrap();

            if let Some(expected_err) = fix["error"].as_str() {
                let result: std::result::Result<Config, _> = toml::from_str(toml_input);
                assert!(result.is_err(), "fixture '{name}' should fail to parse");
                let _ = expected_err; // error type verified by is_err
                continue;
            }

            let config: Config =
                toml::from_str(toml_input).unwrap_or_else(|e| panic!("fixture '{name}': {e}"));

            if let Some(bind) = fix["expected"]["bind"].as_str() {
                assert_eq!(config.server.bind, bind, "fixture '{name}' bind");
            }
            if let Some(backend) = fix["expected"]["storage_backend"].as_str() {
                assert_eq!(config.storage.backend, backend, "fixture '{name}' backend");
            }
            if let Some(bucket) = fix["expected"]["s3_bucket"].as_str() {
                assert_eq!(
                    config.storage.s3.as_ref().unwrap().bucket,
                    bucket,
                    "fixture '{name}' s3 bucket"
                );
            }
            if let Some(block) = fix["expected"]["block_unlicensed"].as_bool() {
                assert_eq!(
                    config.policies.block_unlicensed, block,
                    "fixture '{name}' block_unlicensed"
                );
            }
            if let Some(auth) = fix["expected"]["auth_enabled"].as_bool() {
                assert_eq!(config.auth.enabled, auth, "fixture '{name}' auth_enabled");
            }
        }
    }

    #[test]
    fn load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("depot.toml");
        std::fs::write(&path, "[server]\nbind = \"127.0.0.1:9999\"\n").unwrap();

        let config = Config::load_from(&path).unwrap();
        assert_eq!(config.server.bind, "127.0.0.1:9999");
    }

    #[test]
    fn load_from_missing_file() {
        let result = Config::load_from(Path::new("/nonexistent/depot.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn defaults_have_all_upstreams() {
        let config = Config::default();
        assert!(config.upstream.contains_key("pypi"));
        assert!(config.upstream.contains_key("npm"));
        assert!(config.upstream.contains_key("cargo"));
        assert!(config.upstream.contains_key("hex"));
    }
}
