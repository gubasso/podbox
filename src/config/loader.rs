use std::{fs, path::Path};

use crate::{
    config::{provenance::Provenance, roots::PodboxRoots, schema::Config},
    error::AppError,
};
use camino::{Utf8Path, Utf8PathBuf};

#[derive(Clone, Debug)]
pub(crate) struct LoadedConfig {
    pub(crate) config: Config,
    pub(crate) provenance: Provenance,
    pub(crate) source_files: Vec<Utf8PathBuf>,
}

impl LoadedConfig {
    pub(crate) fn empty() -> Self {
        Self {
            config: Config::default(),
            provenance: Provenance::default(),
            source_files: Vec::new(),
        }
    }

    pub(crate) fn load(
        roots: &PodboxRoots,
        cli_config: Option<&Path>,
        workspace: Option<&Path>,
    ) -> Result<Self, AppError> {
        let _figment_marker = figment::Figment::new();
        let mut merged = toml::Value::Table(toml::map::Map::new());
        let mut provenance = Provenance::default();
        let mut source_files = Vec::new();

        // Sources are merged low-precedence first, high-precedence last, because
        // `merge_value` is last-write-wins. The C8 precedence chain (highest first) is:
        // `--config` > `PODBOX_CONFIG` > project registry > local devcontainer >
        // sibling devcontainer, above the base global `config.toml`. Applying them in the
        // reverse of that ranking makes the highest-precedence layer the final writer.

        // Base: the global `config.toml` (also the source of the `[[projects]]` registry).
        let discovered = roots.config_file();
        if discovered.exists() {
            merge_toml_file(
                &mut merged,
                &mut provenance,
                &mut source_files,
                &discovered,
                "config.toml",
            )?;
        }

        if let Some(workspace) =
            workspace.and_then(|p| Utf8PathBuf::from_path_buf(p.to_path_buf()).ok())
        {
            // Lowest workspace layer first.
            let sibling = workspace.join("../.devcontainer/devcontainer.json");
            if sibling.exists() {
                merge_devcontainer_json(
                    &mut merged,
                    &mut provenance,
                    &mut source_files,
                    &sibling,
                    "sibling devcontainer",
                )?;
            }
            let local = workspace.join(".devcontainer/devcontainer.json");
            if local.exists() {
                merge_devcontainer_json(
                    &mut merged,
                    &mut provenance,
                    &mut source_files,
                    &local,
                    "local devcontainer",
                )?;
            }
            // Project registry outranks the workspace devcontainer layers.
            apply_project_registry(&mut merged, &mut provenance, &workspace, "projects");
        }

        // Environment override outranks all discovery.
        if let Some(env_path) = std::env::var_os("PODBOX_CONFIG") {
            let path = Utf8PathBuf::from_path_buf(env_path.into()).map_err(|p| {
                AppError::usage(format!("PODBOX_CONFIG is not UTF-8: {}", p.display()))
            })?;
            merge_toml_file(
                &mut merged,
                &mut provenance,
                &mut source_files,
                &path,
                "PODBOX_CONFIG",
            )?;
        }
        // Explicit `--config` is the highest-precedence layer, applied last.
        if let Some(cli_path) = cli_config {
            let path = Utf8PathBuf::from_path_buf(cli_path.to_path_buf()).map_err(|p| {
                AppError::usage(format!("--config path is not UTF-8: {}", p.display()))
            })?;
            merge_toml_file(
                &mut merged,
                &mut provenance,
                &mut source_files,
                &path,
                "--config",
            )?;
        }

        // C8 (spec `03` §5): when a workspace is being resolved and no config source in the
        // precedence chain resolved, fail with the actionable remedy and exit 2 rather than
        // silently succeeding with defaults. Commands without a `--workspace` (e.g. `version`,
        // `config paths`) do not require workspace config resolution and keep defaults.
        if let Some(workspace) = workspace
            && source_files.is_empty()
        {
            return Err(AppError::config_syntax(format!(
                "no config for `{}`; run `podbox init` or pass `--config`",
                workspace.display()
            )));
        }

        let config: Config = merged
            .try_into()
            .map_err(|err| AppError::config_syntax(format!("invalid config.toml: {err}")))?;
        config.validate()?;
        record_defaults(&config, &mut provenance);
        Ok(Self {
            config,
            provenance,
            source_files,
        })
    }
}

fn merge_toml_file(
    merged: &mut toml::Value,
    provenance: &mut Provenance,
    source_files: &mut Vec<Utf8PathBuf>,
    path: &Utf8Path,
    label: &str,
) -> Result<(), AppError> {
    let raw = fs::read_to_string(path).map_err(|err| AppError::io(path.to_path_buf(), err))?;
    let value: toml::Value =
        toml::from_str(&raw).map_err(|err| AppError::config_syntax(format!("{path}: {err}")))?;
    merge_value(merged, value, provenance, label, "");
    source_files.push(path.to_path_buf());
    Ok(())
}

fn merge_devcontainer_json(
    merged: &mut toml::Value,
    provenance: &mut Provenance,
    source_files: &mut Vec<Utf8PathBuf>,
    path: &Utf8Path,
    label: &str,
) -> Result<(), AppError> {
    let raw = fs::read_to_string(path).map_err(|err| AppError::io(path.to_path_buf(), err))?;
    let json: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|err| AppError::config_syntax(format!("{path}: {err}")))?;
    let mut table = toml::map::Map::new();
    if let Some(name) = json.get("name").and_then(serde_json::Value::as_str) {
        table.insert(
            "defaults".to_string(),
            toml::Value::Table({
                let mut defaults = toml::map::Map::new();
                defaults.insert(
                    "manifest".to_string(),
                    toml::Value::String(name.to_string()),
                );
                defaults
            }),
        );
    }
    if let Some(args) = json.get("runArgs").and_then(serde_json::Value::as_array) {
        let allow: Vec<toml::Value> = args
            .iter()
            .filter_map(serde_json::Value::as_str)
            .map(|s| toml::Value::String(s.to_string()))
            .collect();
        table.insert(
            "network".to_string(),
            toml::Value::Table({
                let mut network = toml::map::Map::new();
                network.insert("allow".to_string(), toml::Value::Array(allow));
                network
            }),
        );
    }
    merge_value(merged, toml::Value::Table(table), provenance, label, "");
    source_files.push(path.to_path_buf());
    Ok(())
}

fn apply_project_registry(
    merged: &mut toml::Value,
    provenance: &mut Provenance,
    workspace: &Utf8Path,
    label: &str,
) {
    let Some(projects) = merged
        .get("projects")
        .and_then(toml::Value::as_array)
        .cloned()
    else {
        return;
    };
    for project in &projects {
        let Some(path) = project.get("path").and_then(toml::Value::as_str) else {
            continue;
        };
        if path == workspace.as_str() {
            if let Some(manifest) = project.get("manifest").and_then(toml::Value::as_str) {
                let overlay: toml::Value = toml::toml! { [defaults] manifest = manifest }.into();
                merge_value(merged, overlay, provenance, label, "");
            }
        }
    }
}

fn merge_value(
    target: &mut toml::Value,
    source: toml::Value,
    provenance: &mut Provenance,
    label: &str,
    prefix: &str,
) {
    match (target, source) {
        (toml::Value::Table(dst), toml::Value::Table(src)) => {
            for (key, value) in src {
                let next = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                match dst.get_mut(&key) {
                    Some(existing) => merge_value(existing, value, provenance, label, &next),
                    None => {
                        record_leaves(&value, provenance, label, &next);
                        dst.insert(key, value);
                    }
                }
            }
        }
        (target, value) => {
            record_leaves(&value, provenance, label, prefix);
            *target = value;
        }
    }
}

fn record_leaves(value: &toml::Value, provenance: &mut Provenance, label: &str, prefix: &str) {
    match value {
        toml::Value::Table(table) => {
            for (key, nested) in table {
                let next = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                record_leaves(nested, provenance, label, &next);
            }
        }
        _ => provenance.record(prefix, label),
    }
}

fn record_defaults(config: &Config, provenance: &mut Provenance) {
    for key in [
        "defaults.manifest",
        "defaults.reconcile",
        "defaults.color",
        "defaults.output",
        "runtime.profile",
        "runtime.memory_mib",
        "runtime.cpus",
        "images.pull_policy",
        "network.egress",
    ] {
        if provenance.source(key) == "default" {
            provenance.record(key, "default");
        }
    }
    if config.network.allow.is_empty() && provenance.source("network.allow") == "default" {
        provenance.record("network.allow", "default");
    }
}
