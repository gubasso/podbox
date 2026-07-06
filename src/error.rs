use camino::Utf8PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error("{message}")]
    Usage { message: String },
    #[error("{message}")]
    UsageOrConfigSyntax { message: String },
    #[error("{message}")]
    Config { message: String },
    #[error("{message}")]
    HostRuntime { message: String },
    #[error("{message}")]
    Validation { message: String },
    #[error("{message}")]
    WorkspaceState { message: String },
    #[error("{message}")]
    Build { message: String },
    #[error("{message}")]
    DestructiveRefused { message: String },
    #[error("{message}")]
    Unexpected { message: String },
    #[error("I/O error at {path}: {source}")]
    Io {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl AppError {
    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self::Usage {
            message: message.into(),
        }
    }

    pub(crate) fn config_syntax(message: impl Into<String>) -> Self {
        Self::UsageOrConfigSyntax {
            message: message.into(),
        }
    }

    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub(crate) fn unexpected(message: impl Into<String>) -> Self {
        Self::Unexpected {
            message: message.into(),
        }
    }

    pub(crate) fn io(path: impl Into<Utf8PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub(crate) fn kind(&self) -> &'static str {
        match self {
            AppError::Usage { .. } => "usage",
            AppError::UsageOrConfigSyntax { .. } => "usage_or_config_syntax",
            AppError::Config { .. } => "config",
            AppError::HostRuntime { .. } => "host_runtime",
            AppError::Validation { .. } => "validation",
            AppError::WorkspaceState { .. } => "workspace_state",
            AppError::Build { .. } => "build",
            AppError::DestructiveRefused { .. } => "destructive_refused",
            AppError::Unexpected { .. } => "unexpected",
            AppError::Io { .. } => "io",
        }
    }
}
