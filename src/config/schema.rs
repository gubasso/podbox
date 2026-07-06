use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) meta: MetaConfig,
    #[serde(default)]
    pub(crate) defaults: DefaultsConfig,
    #[serde(default)]
    pub(crate) runtime: RuntimeConfig,
    #[serde(default)]
    pub(crate) images: ImagesConfig,
    #[serde(default)]
    pub(crate) network: NetworkConfig,
    #[serde(default)]
    pub(crate) projects: Vec<ProjectConfig>,
    #[serde(flatten)]
    pub(crate) unknown: BTreeMap<String, toml::Value>,
}

impl Config {
    pub(crate) fn validate(&self) -> Result<(), AppError> {
        validate_one(
            "defaults.reconcile",
            &self.defaults.reconcile,
            &["auto", "always", "never"],
        )?;
        validate_one(
            "defaults.color",
            &self.defaults.color,
            &["auto", "always", "never"],
        )?;
        validate_one("defaults.output", &self.defaults.output, &["human", "json"])?;
        validate_one(
            "images.pull_policy",
            &self.images.pull_policy,
            &["missing", "newer", "always", "never"],
        )?;
        validate_one(
            "network.egress",
            &self.network.egress,
            &["deny", "allowlist"],
        )?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct MetaConfig {
    #[serde(default = "schema_one")]
    pub(crate) schema_version: u32,
}

impl Default for MetaConfig {
    fn default() -> Self {
        Self {
            schema_version: schema_one(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct DefaultsConfig {
    #[serde(default = "default_manifest")]
    pub(crate) manifest: String,
    #[serde(default = "default_reconcile")]
    pub(crate) reconcile: String,
    #[serde(default = "default_color")]
    pub(crate) color: String,
    #[serde(default = "default_output")]
    pub(crate) output: String,
    #[serde(flatten)]
    pub(crate) unknown: BTreeMap<String, toml::Value>,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            manifest: default_manifest(),
            reconcile: default_reconcile(),
            color: default_color(),
            output: default_output(),
            unknown: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct RuntimeConfig {
    #[serde(default = "default_profile")]
    pub(crate) profile: String,
    #[serde(default = "default_memory")]
    pub(crate) memory_mib: u32,
    #[serde(default = "default_cpus")]
    pub(crate) cpus: u32,
    #[serde(flatten)]
    pub(crate) unknown: BTreeMap<String, toml::Value>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            profile: default_profile(),
            memory_mib: default_memory(),
            cpus: default_cpus(),
            unknown: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ImagesConfig {
    #[serde(default = "default_pull_policy")]
    pub(crate) pull_policy: String,
}

impl Default for ImagesConfig {
    fn default() -> Self {
        Self {
            pull_policy: default_pull_policy(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct NetworkConfig {
    #[serde(default = "default_egress")]
    pub(crate) egress: String,
    #[serde(default)]
    pub(crate) allow: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            egress: default_egress(),
            allow: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ProjectConfig {
    pub(crate) name: Option<String>,
    pub(crate) path: String,
    pub(crate) manifest: Option<String>,
    pub(crate) composed_digest: Option<String>,
    pub(crate) image_digest: Option<String>,
}

fn schema_one() -> u32 {
    1
}
fn default_manifest() -> String {
    "default".to_string()
}
fn default_reconcile() -> String {
    "auto".to_string()
}
fn default_color() -> String {
    "auto".to_string()
}
fn default_output() -> String {
    "human".to_string()
}
fn default_profile() -> String {
    "microvm".to_string()
}
fn default_memory() -> u32 {
    2048
}
fn default_cpus() -> u32 {
    2
}
fn default_pull_policy() -> String {
    "missing".to_string()
}
fn default_egress() -> String {
    "deny".to_string()
}

fn validate_one(key: &str, value: &str, allowed: &[&str]) -> Result<(), AppError> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(AppError::validation(format!(
            "{key} has invalid value `{value}`"
        )))
    }
}
