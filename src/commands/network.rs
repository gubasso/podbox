use std::fs;

use toml_edit::{DocumentMut, Item};

use crate::{
    cli::network::{NetworkArgs, NetworkCommand},
    context::AppContext,
    domain::network::AllowEntry,
    error::AppError,
    services::workspace_lifecycle::WorkspaceLifecycleService,
};

pub(crate) fn run(ctx: &AppContext, args: NetworkArgs) -> Result<(), AppError> {
    match args.command {
        NetworkCommand::Show => show(ctx),
        NetworkCommand::Allow { entry, manifest } => allow(ctx, &entry, manifest),
    }
}

fn show(ctx: &AppContext) -> Result<(), AppError> {
    let report = WorkspaceLifecycleService::new(ctx).network_show()?;
    if ctx.global.json {
        ctx.ui.json(&report)
    } else {
        ctx.ui
            .stdout_line(&format!("posture: {}", report.posture))?;
        if report.effective_allow.is_empty() {
            ctx.ui.stdout_line("allow: []")
        } else {
            ctx.ui
                .stdout_line(&format!("allow: {}", report.effective_allow.join(", ")))
        }
    }
}

fn allow(ctx: &AppContext, entry: &str, manifest: bool) -> Result<(), AppError> {
    let entry = AllowEntry::parse(entry).map_err(AppError::validation)?;
    if manifest {
        write_manifest_allow(ctx, &entry.0)?;
    } else {
        write_user_allow(ctx, &entry.0)?;
    }
    ctx.ui
        .stdout_line("network allowlist updated; run `podbox workspace reconcile` to apply")
}

fn write_user_allow(ctx: &AppContext, entry: &str) -> Result<(), AppError> {
    let path = ctx
        .global
        .config
        .clone()
        .and_then(|p| camino::Utf8PathBuf::from_path_buf(p).ok())
        .unwrap_or_else(|| ctx.roots.config_file());
    let mut doc = read_doc(&path)?;
    append_network_allow(&mut doc, entry)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| AppError::io(parent.to_path_buf(), err))?;
    }
    fs::write(&path, doc.to_string()).map_err(|err| AppError::io(path, err))
}

fn write_manifest_allow(ctx: &AppContext, entry: &str) -> Result<(), AppError> {
    let name = ctx
        .global
        .manifest
        .clone()
        .unwrap_or_else(|| ctx.config.config.defaults.manifest.clone());
    let path = ctx.roots.manifests_dir().join(format!("{name}.toml"));
    if !path.exists() {
        return Err(AppError::validation(format!(
            "manifest `{name}` does not exist"
        )));
    }
    let mut doc = read_doc(&path)?;
    append_network_allow(&mut doc, entry)?;
    fs::write(&path, doc.to_string()).map_err(|err| AppError::io(path, err))
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

fn append_network_allow(doc: &mut DocumentMut, entry: &str) -> Result<(), AppError> {
    if !doc["network"].is_table() {
        doc["network"] = Item::Table(toml_edit::Table::new());
    }
    if !doc["network"]["allow"].is_array() {
        doc["network"]["allow"] = toml_edit::value(toml_edit::Array::default());
    }
    let Some(array) = doc["network"]["allow"].as_array_mut() else {
        return Err(AppError::config_syntax("network.allow must be an array"));
    };
    // `entry` is already canonical (the caller parsed it). Compare against the
    // canonical form of each stored value so a pre-existing or hand-edited
    // non-canonical entry (e.g. `GitHub.COM:443`) is recognised as a duplicate
    // rather than appended a second time. Values that do not parse are compared
    // verbatim so we never silently drop them.
    let already_present = array.iter().any(|value| {
        value.as_str().is_some_and(|existing| {
            AllowEntry::parse(existing)
                .map(|parsed| parsed.0 == entry)
                .unwrap_or(existing == entry)
        })
    });
    if !already_present {
        array.push(entry);
    }
    Ok(())
}
