use std::process::ExitCode as ProcessExitCode;

use crate::error::AppError;

pub(crate) const SUCCESS: u8 = 0;
pub(crate) const UNEXPECTED: u8 = 1;
pub(crate) const USAGE_OR_CONFIG: u8 = 2;
pub(crate) const HOST_RUNTIME: u8 = 3;
pub(crate) const VALIDATION: u8 = 4;
pub(crate) const WORKSPACE_STATE: u8 = 5;
pub(crate) const BUILD: u8 = 6;
pub(crate) const DESTRUCTIVE_REFUSED: u8 = 7;

impl AppError {
    pub(crate) fn exit_code(&self) -> u8 {
        match self {
            AppError::Usage { .. } => USAGE_OR_CONFIG,
            AppError::UsageOrConfigSyntax { .. } => USAGE_OR_CONFIG,
            AppError::Config { .. } => USAGE_OR_CONFIG,
            AppError::HostRuntime { .. } => HOST_RUNTIME,
            AppError::Validation { .. } => VALIDATION,
            AppError::WorkspaceState { .. } => WORKSPACE_STATE,
            AppError::Build { .. } => BUILD,
            AppError::DestructiveRefused { .. } => DESTRUCTIVE_REFUSED,
            AppError::Unexpected { .. } => UNEXPECTED,
            AppError::Io { .. } => UNEXPECTED,
        }
    }
}

pub(crate) fn exit_process(code: u8) -> ProcessExitCode {
    ProcessExitCode::from(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;

    #[test]
    fn exit_matrix_is_explicit() {
        let cases = [
            (
                AppError::Usage {
                    message: String::new(),
                },
                USAGE_OR_CONFIG,
            ),
            (
                AppError::UsageOrConfigSyntax {
                    message: String::new(),
                },
                USAGE_OR_CONFIG,
            ),
            (
                AppError::Config {
                    message: String::new(),
                },
                USAGE_OR_CONFIG,
            ),
            (
                AppError::HostRuntime {
                    message: String::new(),
                },
                HOST_RUNTIME,
            ),
            (
                AppError::Validation {
                    message: String::new(),
                },
                VALIDATION,
            ),
            (
                AppError::WorkspaceState {
                    message: String::new(),
                },
                WORKSPACE_STATE,
            ),
            (
                AppError::Build {
                    message: String::new(),
                },
                BUILD,
            ),
            (
                AppError::DestructiveRefused {
                    message: String::new(),
                },
                DESTRUCTIVE_REFUSED,
            ),
            (
                AppError::Unexpected {
                    message: String::new(),
                },
                UNEXPECTED,
            ),
            (
                AppError::Io {
                    path: Utf8PathBuf::from("/tmp/x"),
                    source: std::io::Error::other("x"),
                },
                UNEXPECTED,
            ),
        ];
        for (err, code) in cases {
            assert_eq!(err.exit_code(), code);
        }
    }
}
