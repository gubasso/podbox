use std::fs;

use camino::Utf8PathBuf;
use serde::Serialize;

use crate::{
    cli::manifest::{ManifestArgs, ManifestCommand},
    context::AppContext,
    domain::manifest::{LayerRef, ManifestDocument},
    error::AppError,
    services::compose::{ComposeInput, ComposeService, LayerInput},
};

pub(crate) fn run(ctx: &AppContext, args: ManifestArgs) -> Result<(), AppError> {
    match args.command {
        ManifestCommand::List => list(ctx),
        ManifestCommand::Show { name } => show(ctx, &name),
        ManifestCommand::Validate { name } => validate(ctx, &name),
        ManifestCommand::Compose { name } => compose(ctx, &name),
    }
}

fn list(ctx: &AppContext) -> Result<(), AppError> {
    crate::config::layout::validate_layout(&ctx.roots)?;
    let dir = ctx.roots.manifests_dir();
    let mut names = Vec::new();
    if dir.exists() {
        for entry in fs::read_dir(&dir).map_err(|err| AppError::io(dir.clone(), err))? {
            let entry = entry.map_err(|err| {
                AppError::unexpected(format!("failed to read manifest dir: {err}"))
            })?;
            let path = Utf8PathBuf::from_path_buf(entry.path())
                .map_err(|p| AppError::unexpected(format!("non UTF-8 path: {}", p.display())))?;
            if path.extension() == Some("toml") {
                if let Some(stem) = path.file_stem() {
                    names.push(stem.to_string());
                }
            }
        }
    }
    names.sort();
    if ctx.global.json {
        #[derive(Serialize)]
        struct Report {
            schema_version: u32,
            manifests: Vec<String>,
        }
        ctx.ui.json(&Report {
            schema_version: crate::util::schema_version(),
            manifests: names,
        })
    } else {
        for name in names {
            ctx.ui.stdout_line(&name)?;
        }
        Ok(())
    }
}

fn show(ctx: &AppContext, name: &str) -> Result<(), AppError> {
    let (manifest, _) = read_manifest(ctx, name)?;
    manifest.validate(name)?;
    #[derive(Serialize)]
    struct Report {
        schema_version: u32,
        name: String,
        layers: Vec<String>,
        leaf: Option<String>,
        shared: Vec<String>,
    }
    let report = Report {
        schema_version: crate::util::schema_version(),
        name: name.to_string(),
        layers: manifest.layers.iter().map(|l| l.0.clone()).collect(),
        leaf: manifest.leaf().map(|l| l.0.clone()),
        shared: manifest
            .layers
            .iter()
            .take(manifest.layers.len().saturating_sub(1))
            .map(|l| l.0.clone())
            .collect(),
    };
    if ctx.global.json {
        ctx.ui.json(&report)
    } else {
        ctx.ui.stdout_line(&format!("manifest: {}", report.name))?;
        ctx.ui
            .stdout_line(&format!("layers: {}", report.layers.join(" -> ")))?;
        ctx.ui
            .stdout_line(&format!("leaf: {}", report.leaf.unwrap_or_default()))
    }
}

fn validate(ctx: &AppContext, name: &str) -> Result<(), AppError> {
    let (manifest, raw) = read_manifest(ctx, name)?;
    manifest.validate(name)?;
    let layers = read_layers(ctx, &manifest)?;
    crate::services::compose::validate(&ComposeInput {
        manifest_name: name.to_string(),
        manifest_content: raw,
        manifest,
        layers,
    })?;
    ctx.ui.stdout_line("manifest valid")
}

fn compose(ctx: &AppContext, name: &str) -> Result<(), AppError> {
    let (manifest, raw) = read_manifest(ctx, name)?;
    let layers = read_layers(ctx, &manifest)?;
    let output = ComposeService::new(&ctx.roots).compose(ComposeInput {
        manifest_name: name.to_string(),
        manifest_content: raw,
        manifest,
        layers,
    })?;
    if ctx.global.json {
        ctx.ui.json(&output)
    } else {
        ctx.ui
            .stdout_line(&format!("composed: {}", output.json_path))?;
        ctx.ui
            .stdout_line(&format!("digest: {}", output.metadata.digest.as_str()))
    }
}

fn read_manifest(ctx: &AppContext, name: &str) -> Result<(ManifestDocument, String), AppError> {
    let path = ctx.roots.manifests_dir().join(format!("{name}.toml"));
    let raw = fs::read_to_string(&path).map_err(|err| AppError::io(path.clone(), err))?;
    let manifest: ManifestDocument = toml::from_str(&raw)
        .map_err(|err| AppError::validation(format!("manifest `{name}` is invalid: {err}")))?;
    Ok((manifest, raw))
}

fn read_layers(ctx: &AppContext, manifest: &ManifestDocument) -> Result<Vec<LayerInput>, AppError> {
    let mut layers = Vec::new();
    for LayerRef(name) in &manifest.layers {
        let path = ctx.roots.layers_dir().join(name).join("devcontainer.json");
        let content = fs::read_to_string(&path).map_err(|err| {
            AppError::validation(format!("layer `{name}` is missing or unreadable: {err}"))
        })?;
        layers.push(LayerInput {
            name: name.clone(),
            content,
        });
    }
    Ok(layers)
}
