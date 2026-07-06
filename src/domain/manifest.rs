use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{domain::network::Allowlist, error::AppError};

pub(crate) const SCHEMA_VERSION: u32 = 1;
pub(crate) const COMPOSITION_RULES_VERSION: &str = "podbox-compose-v1";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub(crate) struct ManifestName(pub(crate) String);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub(crate) struct LayerName(pub(crate) String);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub(crate) struct LayerRef(pub(crate) String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct RuntimeProfile(pub(crate) String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct RuntimeResources {
    pub(crate) memory_mib: u32,
    pub(crate) cpus: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ManifestDocument {
    pub(crate) layers: Vec<LayerRef>,
    pub(crate) runtime: Option<RuntimeProfile>,
    pub(crate) resources: Option<RuntimeResources>,
    pub(crate) network: Option<ManifestNetwork>,
}

impl ManifestDocument {
    pub(crate) fn validate(&self, name: &str) -> Result<(), AppError> {
        if self.layers.is_empty() {
            return Err(AppError::validation(format!(
                "manifest `{name}` must declare at least one layer"
            )));
        }
        Ok(())
    }

    pub(crate) fn leaf(&self) -> Option<&LayerRef> {
        self.layers.last()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ManifestNetwork {
    #[serde(default)]
    pub(crate) allow: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct DevcontainerFragment {
    #[serde(default)]
    pub(crate) name: Option<String>,
    #[serde(default)]
    pub(crate) image: Option<String>,
    #[serde(default)]
    pub(crate) mounts: Vec<String>,
    // Lifecycle commands accept the string, array, and object devcontainer forms
    // (spec `04`: "Merge by key (object form); string/array forms preserved").
    #[serde(default, rename = "postCreateCommand")]
    pub(crate) post_create_command: Option<serde_json::Value>,
    #[serde(default, rename = "postStartCommand")]
    pub(crate) post_start_command: Option<serde_json::Value>,
    #[serde(default, rename = "containerEnv")]
    pub(crate) container_env: BTreeMap<String, String>,
    #[serde(default, rename = "remoteEnv")]
    pub(crate) remote_env: BTreeMap<String, String>,
    #[serde(default, rename = "runArgs")]
    pub(crate) run_args: Vec<String>,
    #[serde(default, rename = "workspaceMount")]
    pub(crate) workspace_mount: Option<String>,
    #[serde(default, rename = "workspaceFolder")]
    pub(crate) workspace_folder: Option<String>,
    #[serde(default)]
    pub(crate) network: Option<ManifestNetwork>,
    #[serde(flatten)]
    pub(crate) extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ComposedDevcontainer {
    pub(crate) schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) image: Option<String>,
    pub(crate) mounts: Vec<String>,
    #[serde(rename = "postCreateCommand", skip_serializing_if = "Option::is_none")]
    pub(crate) post_create_command: Option<serde_json::Value>,
    #[serde(rename = "postStartCommand", skip_serializing_if = "Option::is_none")]
    pub(crate) post_start_command: Option<serde_json::Value>,
    #[serde(rename = "containerEnv")]
    pub(crate) container_env: BTreeMap<String, String>,
    #[serde(rename = "remoteEnv")]
    pub(crate) remote_env: BTreeMap<String, String>,
    #[serde(rename = "runArgs")]
    pub(crate) run_args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "workspaceMount")]
    pub(crate) workspace_mount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "workspaceFolder")]
    pub(crate) workspace_folder: Option<String>,
    // Documented public shape (spec `04`): the composed egress allowlist lives under
    // `network.allow`, part of the supported devcontainer subset — not a top-level key.
    pub(crate) network: ManifestNetwork,
}

impl Default for ComposedDevcontainer {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            name: None,
            image: None,
            mounts: Vec::new(),
            post_create_command: None,
            post_start_command: None,
            container_env: BTreeMap::new(),
            remote_env: BTreeMap::new(),
            run_args: Vec::new(),
            workspace_mount: None,
            workspace_folder: None,
            network: ManifestNetwork { allow: Vec::new() },
        }
    }
}

pub(crate) fn merge_fragments(
    fragments: &[DevcontainerFragment],
    manifest_allow: &[String],
) -> ComposedDevcontainer {
    let mut out = ComposedDevcontainer::default();
    let mut allow = Allowlist::default().union(&Allowlist(
        manifest_allow
            .iter()
            .cloned()
            .map(crate::domain::network::AllowEntry)
            .collect(),
    ));
    for fragment in fragments {
        if fragment.name.is_some() {
            out.name = fragment.name.clone();
        }
        if fragment.image.is_some() {
            out.image = fragment.image.clone();
        }
        out.mounts.extend(fragment.mounts.clone());
        merge_lifecycle(&mut out.post_create_command, &fragment.post_create_command);
        merge_lifecycle(&mut out.post_start_command, &fragment.post_start_command);
        out.container_env.extend(fragment.container_env.clone());
        out.remote_env.extend(fragment.remote_env.clone());
        out.run_args.extend(fragment.run_args.clone());
        if fragment.workspace_mount.is_some() {
            out.workspace_mount = fragment.workspace_mount.clone();
        }
        if fragment.workspace_folder.is_some() {
            out.workspace_folder = fragment.workspace_folder.clone();
        }
        if let Some(network) = &fragment.network {
            let next = Allowlist(
                network
                    .allow
                    .iter()
                    .cloned()
                    .map(crate::domain::network::AllowEntry)
                    .collect(),
            );
            allow = allow.union(&next);
        }
    }
    out.network.allow = allow.0.into_iter().map(|entry| entry.0).collect();
    out
}

/// Merge a lifecycle-command field (`postCreateCommand`/`postStartCommand`) across layers.
/// Object forms merge by key (later layer wins per key); string and array forms are preserved
/// and replace the accumulator whole (last-wins), matching spec `04`'s merge table.
fn merge_lifecycle(acc: &mut Option<serde_json::Value>, next: &Option<serde_json::Value>) {
    let Some(next) = next else { return };
    match (acc.as_mut(), next) {
        (Some(serde_json::Value::Object(dst)), serde_json::Value::Object(src)) => {
            for (key, value) in src {
                dst.insert(key.clone(), value.clone());
            }
        }
        _ => *acc = Some(next.clone()),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LifecycleCommand(pub(crate) String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct NetworkAllowEntry(pub(crate) String);

pub(crate) fn supported_devcontainer_subset() -> &'static [&'static str] {
    &[
        "name",
        "image",
        "mounts",
        "postCreateCommand",
        "postStartCommand",
        "containerEnv",
        "remoteEnv",
        "runArgs",
        "workspaceMount",
        "workspaceFolder",
        "network",
    ]
}
