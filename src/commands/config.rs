use std::fs;

use serde::Serialize;
use toml_edit::{DocumentMut, Item};

use crate::{
    cli::config::{ConfigArgs, ConfigCommand},
    config::{layout, loader::LoadedConfig},
    context::AppContext,
    error::AppError,
};

#[derive(Serialize)]
struct PathsReport {
    schema_version: u32,
    config: String,
    cache: String,
    state: String,
    data: String,
}

pub(crate) fn run(ctx: &AppContext, args: ConfigArgs) -> Result<(), AppError> {
    match args.command {
        ConfigCommand::Paths => paths(ctx),
        ConfigCommand::Show => show(ctx),
        ConfigCommand::Validate => validate(ctx),
        ConfigCommand::Get { key } => get(ctx, &key),
        ConfigCommand::Set { key, value } => set(ctx, &key, &value),
        ConfigCommand::Unset { key } => unset(ctx, &key),
    }
}

fn paths(ctx: &AppContext) -> Result<(), AppError> {
    let report = PathsReport {
        schema_version: crate::util::schema_version(),
        config: ctx.roots.config.to_string(),
        cache: ctx.roots.cache.to_string(),
        state: ctx.roots.state.to_string(),
        data: ctx.roots.data.to_string(),
    };
    if ctx.global.json {
        ctx.ui.json(&report)
    } else {
        ctx.ui.stdout_line(&format!("config: {}", report.config))?;
        ctx.ui.stdout_line(&format!("cache: {}", report.cache))?;
        ctx.ui.stdout_line(&format!("state: {}", report.state))?;
        ctx.ui.stdout_line(&format!("data: {}", report.data))
    }
}

fn show(ctx: &AppContext) -> Result<(), AppError> {
    #[derive(Serialize)]
    struct Show<'a> {
        schema_version: u32,
        config: &'a crate::config::schema::Config,
        provenance: &'a crate::config::provenance::Provenance,
    }
    let report = Show {
        schema_version: crate::util::schema_version(),
        config: &ctx.config.config,
        provenance: &ctx.config.provenance,
    };
    if ctx.global.json {
        ctx.ui.json(&report)
    } else {
        ctx.ui.stdout_line(&format!(
            "defaults.manifest: {} ({})",
            ctx.config.config.defaults.manifest,
            ctx.config.provenance.source("defaults.manifest")
        ))?;
        ctx.ui.stdout_line(&format!(
            "runtime.profile: {} ({})",
            ctx.config.config.runtime.profile,
            ctx.config.provenance.source("runtime.profile")
        ))
    }
}

fn validate(ctx: &AppContext) -> Result<(), AppError> {
    layout::validate_layout(&ctx.roots)?;
    let _ = LoadedConfig::load(
        &ctx.roots,
        ctx.global.config.as_deref(),
        ctx.global.workspace.as_deref(),
    )?;
    ctx.ui.stdout_line("config valid")
}

fn get(ctx: &AppContext, key: &str) -> Result<(), AppError> {
    let value = match key {
        "defaults.manifest" => ctx.config.config.defaults.manifest.clone(),
        "runtime.profile" => ctx.config.config.runtime.profile.clone(),
        "images.pull_policy" => ctx.config.config.images.pull_policy.clone(),
        "network.egress" => ctx.config.config.network.egress.clone(),
        _ => return Err(AppError::usage(format!("unknown config key `{key}`"))),
    };
    ctx.ui.stdout_line(&value)
}

fn set(ctx: &AppContext, key: &str, value: &str) -> Result<(), AppError> {
    let path = ctx
        .global
        .config
        .clone()
        .and_then(|p| camino::Utf8PathBuf::from_path_buf(p).ok())
        .unwrap_or_else(|| ctx.roots.config_file());
    let mut doc = read_doc(&path)?;
    set_item(&mut doc, key, value)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| AppError::io(parent.to_path_buf(), err))?;
    }
    fs::write(&path, doc.to_string()).map_err(|err| AppError::io(path, err))?;
    ctx.ui.stdout_line("config updated")
}

fn unset(ctx: &AppContext, key: &str) -> Result<(), AppError> {
    let path = ctx
        .global
        .config
        .clone()
        .and_then(|p| camino::Utf8PathBuf::from_path_buf(p).ok())
        .unwrap_or_else(|| ctx.roots.config_file());
    let mut doc = read_doc(&path)?;
    unset_item(&mut doc, key)?;
    fs::write(&path, doc.to_string()).map_err(|err| AppError::io(path, err))?;
    ctx.ui.stdout_line("config updated")
}

fn read_doc(path: &camino::Utf8Path) -> Result<DocumentMut, AppError> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(AppError::io(path.to_path_buf(), err)),
    };
    raw.parse::<DocumentMut>()
        .map_err(|err| AppError::config_syntax(format!("{path}: {err}")))
}

fn set_item(doc: &mut DocumentMut, key: &str, value: &str) -> Result<(), AppError> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() != 2 {
        return Err(AppError::usage("config keys must use section.key syntax"));
    }
    if !doc[parts[0]].is_table() {
        doc[parts[0]] = Item::Table(toml_edit::Table::new());
    }
    doc[parts[0]][parts[1]] = toml_edit::value(value);
    Ok(())
}

fn unset_item(doc: &mut DocumentMut, key: &str) -> Result<(), AppError> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() != 2 {
        return Err(AppError::usage("config keys must use section.key syntax"));
    }
    if let Some(table) = doc[parts[0]].as_table_mut() {
        table.remove(parts[1]);
    }
    Ok(())
}
