use std::process::Command;

use super::{
    BuildImageRequest, DevcontainerRunSpec, HostProbeOutput, RuntimeAdapter, RuntimeError,
    RuntimeStatus, status_from_std,
};
use crate::domain::image::PullPolicy;

#[derive(Clone, Debug, Default)]
pub(crate) struct PodmanKrunRuntime;

impl PodmanKrunRuntime {
    pub(crate) fn new() -> Self {
        Self
    }

    fn build_command(&self, request: &BuildImageRequest) -> PodmanCommand {
        PodmanCommand::build_image(request)
    }

    fn run_command(&self, spec: &DevcontainerRunSpec) -> PodmanCommand {
        PodmanCommand::run_devcontainer(spec)
    }
}

impl RuntimeAdapter for PodmanKrunRuntime {
    fn build_image(&self, request: &BuildImageRequest) -> Result<RuntimeStatus, RuntimeError> {
        run_child(self.build_command(request).into_command())
    }

    fn run_devcontainer(&self, spec: &DevcontainerRunSpec) -> Result<RuntimeStatus, RuntimeError> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = spec;
            Err(RuntimeError::Unsupported(
                "microVM runtime backend requires Linux KVM and rootless containers".to_string(),
            ))
        }
        #[cfg(target_os = "linux")]
        {
            run_child(self.run_command(spec).into_command())
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PodmanCommand {
    program: String,
    args: Vec<String>,
}

impl PodmanCommand {
    fn build_image(request: &BuildImageRequest) -> Self {
        let pull = match request.pull_policy {
            PullPolicy::Missing => "missing",
            PullPolicy::Newer => "newer",
            PullPolicy::Always => "always",
            PullPolicy::Never => "never",
        };
        Self {
            program: "podman".to_string(),
            args: vec![
                "build".to_string(),
                format!("--pull={pull}"),
                "--tag".to_string(),
                request.image_name.clone(),
                "--file".to_string(),
                request.devcontainer_json.to_string(),
                request.context_dir.to_string(),
            ],
        }
    }

    fn run_devcontainer(spec: &DevcontainerRunSpec) -> Self {
        let mut args = vec![
            "run".to_string(),
            "--rm".to_string(),
            "--runtime=krun".to_string(),
            "--security-opt=no-new-privileges".to_string(),
            "--cap-drop=ALL".to_string(),
            "--network=none".to_string(),
            "--workdir".to_string(),
            spec.workspace.to_string(),
        ];
        args.extend(spec.credentials.mount_args());
        args.extend(network_readiness_args(&spec.network_allow));
        args.push(spec.image.clone());
        args.extend(spec.command.iter().cloned());
        Self {
            program: "podman".to_string(),
            args,
        }
    }

    fn to_args(&self) -> Vec<String> {
        std::iter::once(self.program.clone())
            .chain(self.args.iter().cloned())
            .collect()
    }

    fn into_command(self) -> Command {
        let mut command = Command::new(self.program);
        command.args(self.args);
        command
    }
}

fn network_readiness_args(network_allow: &[String]) -> Vec<String> {
    if network_allow.is_empty() {
        Vec::new()
    } else {
        vec![format!(
            "--label=dev.podbox.netback.ready={}",
            network_allow.join(",")
        )]
    }
}

fn run_child(mut command: Command) -> Result<RuntimeStatus, RuntimeError> {
    let status = command.status().map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            RuntimeError::MissingExecutable("runtime backend executable".to_string())
        } else if err.kind() == std::io::ErrorKind::PermissionDenied {
            RuntimeError::NotExecutable("runtime backend executable".to_string())
        } else {
            RuntimeError::Spawn(format!("runtime backend executable: {err}"))
        }
    })?;
    Ok(status_from_std(status))
}

/// Runnable reproduction for the runtime smoke probe. The concrete tool name is
/// confined to this adapter module (N2); callers surface it as an opaque string.
pub(super) fn host_runtime_info_reproduction() -> &'static str {
    "podman info --format '{{json .Host.Security.Rootless}}'"
}

pub(super) fn host_runtime_info_probe() -> Result<HostProbeOutput, std::io::Error> {
    let output = Command::new("podman")
        .args(["info", "--format", "{{json .Host.Security.Rootless}}"])
        .output()?;
    Ok(HostProbeOutput {
        status: output.status.code(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
    })
}

pub(super) fn host_runtime_builder_probe() -> Result<HostProbeOutput, std::io::Error> {
    let output = Command::new("podman").arg("build").arg("--help").output()?;
    Ok(HostProbeOutput {
        status: output.status.code(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
    })
}

#[allow(dead_code)]
fn _assert_no_shell(argv: &[String]) -> bool {
    argv.windows(2).all(|window| window != ["sh", "-c"])
        && argv.iter().all(|arg| !arg.contains('\n'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;

    use crate::adapters::runtime::credentials::CredentialPolicy;

    fn spec() -> DevcontainerRunSpec {
        DevcontainerRunSpec {
            image: "localhost/podbox-test:latest".to_string(),
            workspace: Utf8PathBuf::from("/workspaces/project"),
            command: vec!["/usr/bin/podbox-guest".to_string()],
            network_allow: vec!["github.com:443".to_string()],
            credentials: CredentialPolicy::ephemeral(),
        }
    }

    #[test]
    fn golden_podman_krun_run_argv() {
        let argv = PodmanKrunRuntime::new().run_command(&spec()).to_args();
        insta::assert_json_snapshot!("runtime_golden_argv__podman_krun_run_basic", argv);
    }

    #[test]
    fn podman_krun_argv_has_no_shell_or_embedded_newlines() {
        let argv = PodmanKrunRuntime::new().run_command(&spec()).to_args();
        assert!(_assert_no_shell(&argv));
        assert!(argv.iter().any(|arg| arg == "--runtime=krun"));
    }

    #[test]
    fn constructor_and_argv_are_lazy() {
        let request = BuildImageRequest {
            image_name: "podbox-test".to_string(),
            context_dir: Utf8PathBuf::from("/tmp/context"),
            devcontainer_json: Utf8PathBuf::from("/tmp/context/devcontainer.json"),
            pull_policy: PullPolicy::Missing,
        };
        let argv = PodmanKrunRuntime::new().build_command(&request).to_args();
        assert_eq!(argv[0], "podman");
        assert!(argv.contains(&"--pull=missing".to_string()));
    }

    #[test]
    fn credential_mounts_are_tmpfs_only_in_run_argv() {
        let argv = PodmanKrunRuntime::new().run_command(&spec()).to_args();
        let rendered = argv.join(" ");
        assert!(rendered.contains("--mount=type=tmpfs,destination=/run/podbox/credentials"));
        for denied in CredentialPolicy::ephemeral().deny_sources() {
            assert!(!rendered.contains(denied.as_str()));
        }
    }
}
