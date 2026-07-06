use std::{env, path::PathBuf};

use camino::Utf8PathBuf;
use directories::ProjectDirs;

use crate::error::AppError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PodboxRoots {
    pub(crate) config: Utf8PathBuf,
    pub(crate) cache: Utf8PathBuf,
    pub(crate) state: Utf8PathBuf,
    pub(crate) data: Utf8PathBuf,
}

impl PodboxRoots {
    pub(crate) fn resolve() -> Result<Self, AppError> {
        if let Some(home) = env_path("PODBOX_HOME") {
            return Ok(Self {
                config: home.join("config"),
                cache: home.join("cache"),
                state: home.join("state"),
                data: home.join("data"),
            });
        }
        let project = ProjectDirs::from("", "", "podbox")
            .ok_or_else(|| AppError::unexpected("failed to resolve project directories"))?;
        Ok(Self {
            config: env_path("PODBOX_CONFIG_HOME")
                .or_else(|| env_path("XDG_CONFIG_HOME").map(|p| p.join("podbox")))
                .unwrap_or_else(|| path_to_utf8(project.config_dir().to_path_buf()).join("podbox")),
            cache: env_path("PODBOX_CACHE_HOME")
                .or_else(|| env_path("XDG_CACHE_HOME").map(|p| p.join("podbox")))
                .unwrap_or_else(|| path_to_utf8(project.cache_dir().to_path_buf()).join("podbox")),
            state: env_path("PODBOX_STATE_HOME")
                .or_else(|| env_path("XDG_STATE_HOME").map(|p| p.join("podbox")))
                .unwrap_or_else(|| {
                    env_path("HOME")
                        .map(|p| p.join(".local/state/podbox"))
                        .unwrap_or_else(|| Utf8PathBuf::from(".podbox/state"))
                }),
            data: env_path("PODBOX_DATA_HOME")
                .or_else(|| env_path("XDG_DATA_HOME").map(|p| p.join("podbox")))
                .unwrap_or_else(|| path_to_utf8(project.data_dir().to_path_buf()).join("podbox")),
        })
    }

    pub(crate) fn config_file(&self) -> Utf8PathBuf {
        self.config.join("config.toml")
    }

    pub(crate) fn manifests_dir(&self) -> Utf8PathBuf {
        self.config.join("manifests")
    }

    pub(crate) fn layers_dir(&self) -> Utf8PathBuf {
        self.config.join("devcontainer")
    }
}

fn env_path(key: &str) -> Option<Utf8PathBuf> {
    env::var_os(key).map(PathBuf::from).map(path_to_utf8)
}

fn path_to_utf8(path: PathBuf) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(path)
        .unwrap_or_else(|path| Utf8PathBuf::from(path.to_string_lossy().to_string()))
}
