use std::{fs, io::ErrorKind};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    adapters::runtime::BuildImageRequest,
    context::AppContext,
    domain::{
        digest::DigestInput,
        image::{FreshnessProof, PullPolicy, SourceGraph, SourceGraphFile},
    },
    error::AppError,
};

#[derive(Clone, Debug)]
pub(crate) struct ImageBuildInput {
    pub(crate) name: String,
    pub(crate) full_rebuild: bool,
    pub(crate) pull_policy: PullPolicy,
    pub(crate) dry_run: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ImageBuildReport {
    pub(crate) schema_version: u32,
    pub(crate) image: String,
    pub(crate) digest: String,
    pub(crate) reused: bool,
    pub(crate) metadata_path: String,
    /// Dry-run rebuild/reuse rationale (06 §5). `None` outside `--dry-run`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ImageMetadata {
    schema_version: u32,
    image: String,
    freshness: FreshnessProof,
}

pub(crate) enum ImageBuildOutcome {
    Report(ImageBuildReport),
    ChildExit(u8),
}

pub(crate) struct ImageBuildService<'a> {
    ctx: &'a AppContext,
}

impl<'a> ImageBuildService<'a> {
    pub(crate) fn new(ctx: &'a AppContext) -> Self {
        Self { ctx }
    }

    pub(crate) fn build(&self, input: ImageBuildInput) -> Result<ImageBuildOutcome, AppError> {
        let image_dir = self.ctx.roots.config.join("images").join(&input.name);
        let devcontainer = image_dir.join("devcontainer.json");
        if !devcontainer.exists() {
            return Err(AppError::Build {
                message: format!("image `{}` has no devcontainer.json", input.name),
            });
        }
        let graph = source_graph(&image_dir, input.pull_policy)?;
        let freshness = FreshnessProof {
            digest: graph.digest.clone(),
            graph,
        };
        let meta_path = self
            .ctx
            .roots
            .cache
            .join("images")
            .join(&input.name)
            .join("metadata.json");
        // `--dry-run` reports the rebuild/reuse decision AND why (06 §5) without
        // invoking the runtime or writing freshness metadata.
        if input.dry_run {
            let (reused, reason) = dry_run_decision(&meta_path, &freshness, input.full_rebuild)?;
            let mut r = report(&input.name, &freshness, &meta_path, reused);
            r.reason = Some(reason);
            return Ok(ImageBuildOutcome::Report(r));
        }
        if !input.full_rebuild && metadata_matches(&meta_path, &freshness)? {
            return Ok(ImageBuildOutcome::Report(report(
                &input.name,
                &freshness,
                &meta_path,
                true,
            )));
        }
        let status = self
            .ctx
            .runtime_adapter
            .build_image(&BuildImageRequest {
                image_name: input.name.clone(),
                context_dir: image_dir.clone(),
                devcontainer_json: devcontainer,
                pull_policy: input.pull_policy,
            })
            .map_err(|err| err.into_app_error())?;
        if status.code != 0 {
            return Ok(ImageBuildOutcome::ChildExit(status.code));
        }
        write_metadata(&meta_path, &input.name, freshness.clone())?;
        Ok(ImageBuildOutcome::Report(report(
            &input.name,
            &freshness,
            &meta_path,
            false,
        )))
    }

    pub(crate) fn status(&self, name: &str) -> Result<Option<ImageBuildReport>, AppError> {
        let meta_path = self
            .ctx
            .roots
            .cache
            .join("images")
            .join(name)
            .join("metadata.json");
        match fs::read_to_string(&meta_path) {
            Ok(raw) => {
                let meta: ImageMetadata =
                    serde_json::from_str(&raw).map_err(|err| AppError::Build {
                        message: format!("image metadata is unreadable: {err}"),
                    })?;
                Ok(Some(report(name, &meta.freshness, &meta_path, true)))
            }
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(AppError::io(meta_path, err)),
        }
    }

    pub(crate) fn prune(&self) -> Result<usize, AppError> {
        let dir = self.ctx.roots.cache.join("images");
        if !dir.exists() {
            return Ok(0);
        }
        let count = fs::read_dir(&dir)
            .map_err(|err| AppError::io(dir.clone(), err))?
            .filter_map(Result::ok)
            .count();
        fs::remove_dir_all(&dir).map_err(|err| AppError::io(dir, err))?;
        Ok(count)
    }
}

fn source_graph(root: &Utf8Path, pull_policy: PullPolicy) -> Result<SourceGraph, AppError> {
    let mut files = Vec::new();
    let mut digest = DigestInput::new()
        .part("source-graph-schema", "1")
        .part("pull-policy", format!("{pull_policy:?}"));
    for entry in WalkDir::new(root).sort_by_file_name() {
        let entry = entry.map_err(|err| AppError::Build {
            message: format!("failed to walk image source graph: {err}"),
        })?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = Utf8PathBuf::from_path_buf(entry.path().to_path_buf()).map_err(|path| {
            AppError::Build {
                message: format!("non UTF-8 image source path: {}", path.display()),
            }
        })?;
        let rel = path.strip_prefix(root).unwrap_or(&path).to_string();
        let bytes = fs::read(&path).map_err(|err| AppError::io(path.clone(), err))?;
        let file_digest = DigestInput::new().part(&rel, &bytes).finish();
        digest = digest.part(format!("file:{rel}"), file_digest.as_str());
        files.push(SourceGraphFile {
            path: rel,
            digest: file_digest,
        });
    }
    Ok(SourceGraph {
        digest: digest.finish(),
        pull_policy,
        files,
    })
}

/// Compute the dry-run rebuild/reuse decision and its rationale (06 §5) by
/// comparing the current source graph against the recorded freshness proof.
fn dry_run_decision(
    path: &Utf8Path,
    freshness: &FreshnessProof,
    full_rebuild: bool,
) -> Result<(bool, String), AppError> {
    if full_rebuild {
        return Ok((
            false,
            "would rebuild: --full-rebuild forces a no-cache rebuild".to_string(),
        ));
    }
    match fs::read_to_string(path) {
        Ok(raw) => {
            // Unreadable freshness metadata means freshness cannot be proven, so a
            // dry run reports a rebuild with that rationale rather than erroring
            // out (06 §4/§5) — the strict AppError::Build stays on the real build.
            let Ok(meta) = serde_json::from_str::<ImageMetadata>(&raw) else {
                return Ok((
                    false,
                    "would rebuild: cached freshness proof is unreadable".to_string(),
                ));
            };
            if meta.freshness.digest == freshness.digest {
                Ok((
                    true,
                    "would reuse: all source-graph inputs unchanged".to_string(),
                ))
            } else {
                Ok((
                    false,
                    format!(
                        "would rebuild: source graph changed ({})",
                        describe_graph_changes(&meta.freshness, freshness)
                    ),
                ))
            }
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok((
            false,
            "would rebuild: no cached freshness proof (image not yet built)".to_string(),
        )),
        Err(err) => Err(AppError::io(path.to_path_buf(), err)),
    }
}

/// Summarize which source-graph files were added, removed, or changed between the
/// recorded and current freshness proofs.
fn describe_graph_changes(recorded: &FreshnessProof, current: &FreshnessProof) -> String {
    use std::collections::BTreeMap;
    let old: BTreeMap<&str, &str> = recorded
        .graph
        .files
        .iter()
        .map(|f| (f.path.as_str(), f.digest.as_str()))
        .collect();
    let new: BTreeMap<&str, &str> = current
        .graph
        .files
        .iter()
        .map(|f| (f.path.as_str(), f.digest.as_str()))
        .collect();
    let mut parts = Vec::new();
    for (path, digest) in &new {
        match old.get(path) {
            None => parts.push(format!("added {path}")),
            Some(old_digest) if old_digest != digest => parts.push(format!("changed {path}")),
            Some(_) => {}
        }
    }
    for path in old.keys() {
        if !new.contains_key(path) {
            parts.push(format!("removed {path}"));
        }
    }
    if parts.is_empty() {
        // Digests differ but no per-file change (e.g. pull-policy input changed).
        "build inputs changed".to_string()
    } else {
        parts.join(", ")
    }
}

fn metadata_matches(path: &Utf8Path, freshness: &FreshnessProof) -> Result<bool, AppError> {
    match fs::read_to_string(path) {
        Ok(raw) => {
            let meta: ImageMetadata =
                serde_json::from_str(&raw).map_err(|err| AppError::Build {
                    message: format!("image metadata is unreadable: {err}"),
                })?;
            Ok(meta.freshness.digest == freshness.digest)
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(false),
        Err(err) => Err(AppError::io(path.to_path_buf(), err)),
    }
}

fn write_metadata(path: &Utf8Path, image: &str, freshness: FreshnessProof) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| AppError::io(parent.to_path_buf(), err))?;
    }
    let metadata = ImageMetadata {
        schema_version: crate::util::schema_version(),
        image: image.to_string(),
        freshness,
    };
    let body = serde_json::to_string_pretty(&metadata).map_err(|err| {
        AppError::unexpected(format!("failed to serialize image metadata: {err}"))
    })?;
    fs::write(path, body).map_err(|err| AppError::io(path.to_path_buf(), err))
}

fn report(
    name: &str,
    freshness: &FreshnessProof,
    metadata_path: &Utf8Path,
    reused: bool,
) -> ImageBuildReport {
    ImageBuildReport {
        schema_version: crate::util::schema_version(),
        image: name.to_string(),
        digest: freshness.digest.as_str().to_string(),
        reused,
        metadata_path: metadata_path.to_string(),
        reason: None,
    }
}
