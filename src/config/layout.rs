use std::fs;

use camino::Utf8Path;

use crate::{config::roots::PodboxRoots, error::AppError};

pub(crate) fn validate_layout(roots: &PodboxRoots) -> Result<(), AppError> {
    let forbidden_default = roots.config.join("default").join("devcontainer.json");
    if forbidden_default.exists() {
        return Err(AppError::validation(
            "default/devcontainer.json is not a valid podbox config layer",
        ));
    }
    if roots.layers_dir().exists() {
        for entry in
            fs::read_dir(roots.layers_dir()).map_err(|err| AppError::io(roots.layers_dir(), err))?
        {
            let entry = entry.map_err(|err| {
                AppError::unexpected(format!("failed to read devcontainer entry: {err}"))
            })?;
            if entry.path().extension().is_some_and(|ext| ext == "toml") {
                return Err(AppError::validation(
                    "manifest files are not valid under devcontainer/",
                ));
            }
        }
    }
    if roots.manifests_dir().exists() {
        for entry in fs::read_dir(roots.manifests_dir())
            .map_err(|err| AppError::io(roots.manifests_dir(), err))?
        {
            let entry = entry.map_err(|err| {
                AppError::unexpected(format!("failed to read manifests entry: {err}"))
            })?;
            if entry
                .file_type()
                .map_err(|err| AppError::unexpected(format!("failed to read file type: {err}")))?
                .is_dir()
            {
                return Err(AppError::validation(
                    "layer directories are not valid under manifests/",
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn require_child_file(
    base: &Utf8Path,
    name: &str,
) -> Result<camino::Utf8PathBuf, AppError> {
    let path = base.join(name);
    if path.is_file() {
        Ok(path)
    } else {
        Err(AppError::validation(format!("expected file `{path}`")))
    }
}
