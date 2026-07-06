pub(crate) mod credentials;
pub(crate) mod stub;

use std::process::Command;

use camino::Utf8PathBuf;
use thiserror::Error;

use crate::{domain::image::PullPolicy, error::AppError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeStatus {
    pub(crate) code: u8,
}

impl RuntimeStatus {
    pub(crate) fn success() -> Self {
        Self { code: 0 }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BuildImageRequest {
    pub(crate) image_name: String,
    pub(crate) context_dir: Utf8PathBuf,
    pub(crate) devcontainer_json: Utf8PathBuf,
    pub(crate) pull_policy: PullPolicy,
}

#[derive(Clone, Debug)]
pub(crate) struct DevcontainerRunSpec {
    pub(crate) image: String,
    pub(crate) workspace: Utf8PathBuf,
    pub(crate) command: Vec<String>,
    pub(crate) network_allow: Vec<String>,
    pub(crate) credentials: credentials::CredentialPolicy,
}

pub(crate) trait RuntimeAdapter: Send + Sync {
    fn build_image(&self, request: &BuildImageRequest) -> Result<RuntimeStatus, RuntimeError>;
    fn run_devcontainer(&self, spec: &DevcontainerRunSpec) -> Result<RuntimeStatus, RuntimeError>;
}

#[derive(Debug, Error)]
pub(crate) enum RuntimeError {
    #[error("runtime backend is unavailable: {0}")]
    Unavailable(String),
    #[error("runtime executable is missing: {0}")]
    MissingExecutable(String),
    #[error("runtime executable is not executable: {0}")]
    NotExecutable(String),
    #[error("runtime child failed to spawn: {0}")]
    Spawn(String),
}

impl RuntimeError {
    pub(crate) fn into_app_error(self) -> AppError {
        match self {
            RuntimeError::Unavailable(message)
            | RuntimeError::MissingExecutable(message)
            | RuntimeError::NotExecutable(message)
            | RuntimeError::Spawn(message) => AppError::HostRuntime { message },
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct UnavailableRuntimeAdapter;

impl RuntimeAdapter for UnavailableRuntimeAdapter {
    fn build_image(&self, _request: &BuildImageRequest) -> Result<RuntimeStatus, RuntimeError> {
        Err(RuntimeError::Unavailable(
            "live runtime backend is not implemented in this round".to_string(),
        ))
    }

    fn run_devcontainer(&self, _spec: &DevcontainerRunSpec) -> Result<RuntimeStatus, RuntimeError> {
        Err(RuntimeError::Unavailable(
            "live runtime backend is not implemented in this round".to_string(),
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DevcontainerCommand {
    program: String,
    args: Vec<String>,
}

impl DevcontainerCommand {
    pub(crate) fn new(spec: &DevcontainerRunSpec) -> Self {
        let mut args = vec![
            "run".to_string(),
            "--rm".to_string(),
            "--podbox-zone=runtime".to_string(),
            "--security-opt=no-new-privileges".to_string(),
            "--cap-drop=ALL".to_string(),
            "--network=none".to_string(),
            format!("--workspace={}", spec.workspace),
            format!("--image={}", spec.image),
        ];
        // `--network=none` and `--podbox-network-allow` are complementary, not
        // contradictory (S4 default-deny + composed allowlist handoff): the former
        // disables the runtime's own networking so the guest starts fully isolated,
        // while the latter hands the composed allowlist to podbox-managed in-guest
        // enforcement that selectively re-permits only the listed hosts.
        if !spec.network_allow.is_empty() {
            args.push(format!(
                "--podbox-network-allow={}",
                spec.network_allow.join(",")
            ));
        }
        args.extend(spec.credentials.mount_args());
        args.extend(spec.command.iter().cloned());
        Self {
            program: "devcontainer".to_string(),
            args,
        }
    }

    pub(crate) fn to_args(&self) -> Vec<String> {
        std::iter::once(self.program.clone())
            .chain(self.args.iter().cloned())
            .collect()
    }

    pub(crate) fn into_command(self) -> Command {
        let mut command = Command::new(self.program);
        command.args(self.args);
        command
    }
}

pub(crate) fn status_from_std(status: std::process::ExitStatus) -> RuntimeStatus {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return RuntimeStatus {
                code: 128u8.saturating_add(signal as u8),
            };
        }
    }
    RuntimeStatus {
        code: status.code().unwrap_or(1) as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;

    #[test]
    fn golden_devcontainer_argv() {
        let spec = DevcontainerRunSpec {
            image: "podbox-test:latest".to_string(),
            workspace: Utf8PathBuf::from("/workspaces/project"),
            command: vec!["/bin/echo".to_string(), "ok".to_string()],
            network_allow: vec!["github.com:443".to_string()],
            credentials: credentials::CredentialPolicy::ephemeral(),
        };
        insta::assert_json_snapshot!(
            "runtime_golden_argv__devcontainer_basic",
            DevcontainerCommand::new(&spec).to_args()
        );
    }

    #[test]
    fn generated_argv_has_no_shell_or_embedded_newlines() {
        let spec = DevcontainerRunSpec {
            image: "image".to_string(),
            workspace: Utf8PathBuf::from("/w"),
            command: vec!["true".to_string()],
            network_allow: Vec::new(),
            credentials: credentials::CredentialPolicy::ephemeral(),
        };
        let argv = DevcontainerCommand::new(&spec).to_args();
        for window in argv.windows(2) {
            assert_ne!(window, ["sh", "-c"]);
        }
        assert!(!argv.iter().any(|arg| arg.contains('\n')));
    }
}
