use std::{fs, io::ErrorKind};

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::{
    config::roots::PodboxRoots,
    domain::{
        digest::{Digest, DigestInput},
        manifest::{
            COMPOSITION_RULES_VERSION, ComposedDevcontainer, DevcontainerFragment,
            ManifestDocument, merge_fragments,
        },
    },
    error::AppError,
};

#[derive(Clone, Debug)]
pub(crate) struct ComposeInput {
    pub(crate) manifest_name: String,
    pub(crate) manifest_content: String,
    pub(crate) manifest: ManifestDocument,
    pub(crate) layers: Vec<LayerInput>,
}

#[derive(Clone, Debug)]
pub(crate) struct LayerInput {
    pub(crate) name: String,
    pub(crate) content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ComposeMetadata {
    pub(crate) schema_version: u32,
    pub(crate) digest: Digest,
    pub(crate) manifest: String,
    pub(crate) leaf_layer: String,
    pub(crate) shared_layers: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ComposeOutput {
    pub(crate) composed: ComposedDevcontainer,
    pub(crate) metadata: ComposeMetadata,
    pub(crate) json_path: String,
    pub(crate) metadata_path: String,
    pub(crate) reused: bool,
}

pub(crate) struct ComposeService<'a> {
    roots: &'a PodboxRoots,
}

impl<'a> ComposeService<'a> {
    pub(crate) fn new(roots: &'a PodboxRoots) -> Self {
        Self { roots }
    }

    pub(crate) fn compose(&self, input: ComposeInput) -> Result<ComposeOutput, AppError> {
        let (fragments, digest) = validate_and_digest(&input)?;
        let cache_dir = self.roots.cache.join("composed").join(&input.manifest_name);
        fs::create_dir_all(&cache_dir).map_err(|err| AppError::io(cache_dir.clone(), err))?;
        let json_path = cache_dir.join("devcontainer.json");
        let metadata_path = cache_dir.join("metadata.json");
        let reused = metadata_matches(&metadata_path, &digest)?;
        let manifest_allow = input
            .manifest
            .network
            .as_ref()
            .map(|network| network.allow.clone())
            .unwrap_or_default();
        let composed = merge_fragments(&fragments, &manifest_allow);
        let leaf_layer = input
            .manifest
            .leaf()
            .map(|layer| layer.0.clone())
            .unwrap_or_default();
        let shared_layers = input
            .manifest
            .layers
            .iter()
            .take(input.manifest.layers.len().saturating_sub(1))
            .map(|layer| layer.0.clone())
            .collect();
        let metadata = ComposeMetadata {
            schema_version: crate::util::schema_version(),
            digest,
            manifest: input.manifest_name,
            leaf_layer,
            shared_layers,
        };
        if !reused {
            let rendered = serde_json::to_string_pretty(&composed).map_err(|err| {
                AppError::unexpected(format!("failed to serialize composed devcontainer: {err}"))
            })?;
            fs::write(&json_path, rendered).map_err(|err| AppError::io(json_path.clone(), err))?;
            let rendered_meta = serde_json::to_string_pretty(&metadata).map_err(|err| {
                AppError::unexpected(format!("failed to serialize compose metadata: {err}"))
            })?;
            fs::write(&metadata_path, rendered_meta)
                .map_err(|err| AppError::io(metadata_path.clone(), err))?;
        }
        Ok(ComposeOutput {
            composed,
            metadata,
            json_path: json_path.to_string(),
            metadata_path: metadata_path.to_string(),
            reused,
        })
    }
}

/// Parse and validate the manifest plus every layer fragment without writing anything,
/// returning the parsed fragments and the freshness digest. Used both by `compose` and by
/// the non-writing `validate` path so `manifest validate` enforces the same fragment rules.
fn validate_and_digest(
    input: &ComposeInput,
) -> Result<(Vec<DevcontainerFragment>, Digest), AppError> {
    input.manifest.validate(&input.manifest_name)?;
    let mut fragments = Vec::new();
    let mut digest = DigestInput::new()
        .part("manifest", input.manifest_content.as_bytes())
        .part(
            "schema",
            crate::domain::manifest::SCHEMA_VERSION.to_string(),
        )
        .part("rules", COMPOSITION_RULES_VERSION);
    for layer in &input.layers {
        let fragment: DevcontainerFragment =
            serde_json::from_str(&layer.content).map_err(|err| {
                AppError::validation(format!("layer `{}` is invalid: {err}", layer.name))
            })?;
        digest = digest.part(format!("layer:{}", layer.name), layer.content.as_bytes());
        fragments.push(fragment);
    }
    Ok((fragments, digest.finish()))
}

/// Non-writing validation of a compose input: manifest shape plus every layer fragment's
/// JSON. Returns the same `exit 4` validation errors `compose` would, but touches no cache.
pub(crate) fn validate(input: &ComposeInput) -> Result<(), AppError> {
    validate_and_digest(input).map(|_| ())
}

fn metadata_matches(path: &Utf8PathBuf, digest: &Digest) -> Result<bool, AppError> {
    match fs::read_to_string(path) {
        Ok(raw) => {
            let metadata: ComposeMetadata = serde_json::from_str(&raw).map_err(|err| {
                AppError::validation(format!("compose metadata is unreadable: {err}"))
            })?;
            Ok(metadata.digest == *digest)
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(false),
        Err(err) => Err(AppError::io(path.clone(), err)),
    }
}
