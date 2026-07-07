use std::{fs, io::IsTerminal};

use camino::Utf8PathBuf;
use serde::Serialize;

use crate::{
    adapters::{
        runtime::{
            GuestExecRequest, NetworkPolicyRequest, WorkspaceRuntimeSpec, credentials,
            net_policy::NetworkPolicyArtifact,
        },
        state_store::{StateMutationGuard, StateStoreError},
    },
    context::AppContext,
    domain::{
        digest::{Digest, DigestInput},
        image::PullPolicy,
        manifest::{LayerRef, ManifestDocument},
        network::{AllowEntry, Allowlist},
        state::SandboxState,
        workspace::{
            DriftCause, WorkspaceDrift, WorkspaceIdentity, WorkspaceLabel, WorkspacePath,
            WorkspaceState,
        },
    },
    error::AppError,
    services::{
        compose::{ComposeInput, ComposeOutput, ComposeService, LayerInput},
        image_build::{ImageBuildInput, ImageBuildOutcome, ImageBuildService},
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReconcileMode {
    Auto,
    Always,
    Never,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct WorkspaceStatusReport {
    pub(crate) schema_version: u32,
    pub(crate) identity: WorkspaceIdentity,
    pub(crate) state: SandboxState,
    pub(crate) manifest: String,
    pub(crate) config_sources: Vec<String>,
    pub(crate) image: Option<String>,
    pub(crate) image_freshness: Option<String>,
    pub(crate) drift: WorkspaceDrift,
    pub(crate) operation_in_progress: bool,
    pub(crate) runtime_profile: String,
    pub(crate) hardening_posture: String,
    pub(crate) network_policy: String,
    pub(crate) credential_policy: String,
    pub(crate) last_reconcile: Option<String>,
    pub(crate) relaxations: Vec<String>,
    pub(crate) allow: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ReconcileReport {
    pub(crate) schema_version: u32,
    pub(crate) identity: WorkspaceIdentity,
    pub(crate) dry_run: bool,
    pub(crate) steps: Vec<String>,
    pub(crate) state: SandboxState,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct NetworkShowReport {
    pub(crate) schema_version: u32,
    pub(crate) posture: &'static str,
    pub(crate) effective_allow: Vec<String>,
    pub(crate) manifest_allow: Vec<String>,
    pub(crate) user_allow: Vec<String>,
    pub(crate) relaxation: Option<String>,
}

pub(crate) struct WorkspaceLifecycleService<'a> {
    ctx: &'a AppContext,
}

impl<'a> WorkspaceLifecycleService<'a> {
    pub(crate) fn new(ctx: &'a AppContext) -> Self {
        Self { ctx }
    }

    pub(crate) fn up(&self) -> Result<ReconcileReport, AppError> {
        self.require_config()?;
        let identity = self.identity()?;
        let existing = self.read_state(&identity)?;
        if matches!(
            existing.as_ref().map(|s| s.state),
            Some(SandboxState::Running)
        ) {
            let compose = self.compose();
            let effective_allow = self.effective_allow_for_status(&compose);
            let drift = self.compute_drift(existing.as_ref(), compose.as_ref(), &effective_allow);
            let mut steps = vec!["already-running".to_string()];
            if drift.drifted {
                steps.push("drifted: run `podbox workspace reconcile`".to_string());
            }
            return Ok(ReconcileReport {
                schema_version: crate::util::schema_version(),
                identity,
                dry_run: false,
                steps,
                state: SandboxState::Running,
            });
        }
        self.reconcile(false, true)
    }

    pub(crate) fn reconcile(&self, dry_run: bool, yes: bool) -> Result<ReconcileReport, AppError> {
        self.require_config()?;
        let identity = self.identity()?;
        let previous = self.read_state(&identity)?;
        if matches!(
            previous.as_ref().map(|s| s.state),
            Some(SandboxState::Running)
        ) && !dry_run
            && !yes
            && !std::io::stdin().is_terminal()
        {
            return Err(AppError::DestructiveRefused {
                message: "workspace reconcile would replace a running sandbox; \
                    rerun with --yes or --dry-run"
                    .to_string(),
            });
        }
        let mut steps = vec![
            "compose".to_string(),
            "ensure-image".to_string(),
            "remove-old-sandbox".to_string(),
            "create-sandbox".to_string(),
            "apply-network-policy".to_string(),
            "run-lifecycle-hooks".to_string(),
            "summary".to_string(),
        ];
        if dry_run {
            return Ok(ReconcileReport {
                schema_version: crate::util::schema_version(),
                identity,
                dry_run: true,
                steps,
                state: previous
                    .map(|state| state.state)
                    .unwrap_or(SandboxState::Absent),
            });
        }

        let guard = self
            .ctx
            .state_store
            .begin_mutation(&identity)
            .map_err(state_error)?;
        // Any failure between the first destructive runtime call and the final
        // `Running` write must not leave the previous successful state behind: a
        // stale `Running` record would let later exec/shell/status trust a sandbox
        // that no longer exists. Record a `Failed` state under the held lock on any
        // error before returning (I3 serialized lifecycle + crash recovery).
        match self.reconcile_apply(&identity, previous.as_ref(), &*guard) {
            Ok(()) => Ok(ReconcileReport {
                schema_version: crate::util::schema_version(),
                identity,
                dry_run: false,
                steps: std::mem::take(&mut steps),
                state: SandboxState::Running,
            }),
            Err(err) => {
                let _ = self.write_failed(&identity, &err, &*guard);
                Err(err)
            }
        }
    }

    /// Perform the destructive reconcile steps under a held mutation guard, ending
    /// in a persisted `Running` state on success. Returns the first error without
    /// writing `Running`; the caller records a `Failed` state.
    fn reconcile_apply(
        &self,
        identity: &WorkspaceIdentity,
        previous: Option<&WorkspaceState>,
        guard: &dyn StateMutationGuard,
    ) -> Result<(), AppError> {
        let compose = self.compose()?;
        let image = compose
            .composed
            .image
            .clone()
            .ok_or_else(|| AppError::Build {
                message: "composed devcontainer has no image".to_string(),
            })?;
        self.ensure_image(&image)?;
        if matches!(
            previous.map(|s| s.state),
            Some(SandboxState::Running | SandboxState::Stopped | SandboxState::Drifted)
        ) {
            self.runtime_status(self.ctx.runtime_adapter.stop_workspace(identity))?;
            self.runtime_status(self.ctx.runtime_adapter.remove_workspace(identity))?;
        }
        let allow = self.effective_allow(compose.composed.network.allow.clone())?;
        let fingerprint = reconcile_fingerprint(&compose.metadata.digest, &allow);
        let artifact = NetworkPolicyArtifact::from_allowlist(&Allowlist(
            allow.iter().cloned().map(AllowEntry).collect(),
        ))?;
        self.runtime_status(self.ctx.runtime_adapter.apply_network_policy(
            &NetworkPolicyRequest {
                identity: identity.clone(),
                artifact,
            },
        ))?;
        self.runtime_status(
            self.ctx
                .runtime_adapter
                .start_workspace(&WorkspaceRuntimeSpec {
                    identity: identity.clone(),
                    image,
                    workspace: self.workspace_path()?,
                    command: vec!["/usr/bin/podbox-guest".to_string()],
                    network_allow: allow,
                    credentials: credentials::CredentialPolicy::ephemeral(),
                }),
        )?;
        self.write_state(identity, SandboxState::Running, Some(fingerprint), guard)
    }

    pub(crate) fn exec(&self, argv: Vec<String>) -> Result<u8, AppError> {
        let identity = self.identity()?;
        self.require_running(&identity)?;
        if argv.is_empty() {
            return Err(AppError::usage("workspace exec requires an argv after --"));
        }
        let status = self
            .ctx
            .runtime_adapter
            .exec(&GuestExecRequest {
                identity,
                argv,
                login: false,
                interactive: false,
            })
            .map_err(|err| err.into_app_error())?;
        Ok(status.code)
    }

    pub(crate) fn shell(&self, argv: Vec<String>, mode: ReconcileMode) -> Result<u8, AppError> {
        match mode {
            ReconcileMode::Always => {
                self.reconcile(false, self.ctx.global.yes)?;
            }
            ReconcileMode::Auto => {
                let identity = self.identity()?;
                if !matches!(
                    self.read_state(&identity)?
                        .as_ref()
                        .map(|state| state.state),
                    Some(SandboxState::Running)
                ) {
                    self.reconcile(false, true)?;
                }
            }
            ReconcileMode::Never => {}
        }
        let identity = self.identity()?;
        self.require_running(&identity)?;
        let request = GuestExecRequest {
            identity,
            argv: if argv.is_empty() {
                vec!["/bin/sh".to_string(), "-l".to_string()]
            } else {
                argv
            },
            login: true,
            interactive: true,
        };
        let status = self
            .ctx
            .runtime_adapter
            .shell(&request)
            .map_err(|err| err.into_app_error())?;
        Ok(status.code)
    }

    pub(crate) fn down(&self, dry_run: bool, yes: bool) -> Result<ReconcileReport, AppError> {
        let identity = self.identity()?;
        if !dry_run && !yes && !self.ctx.global.yes && !std::io::stdin().is_terminal() {
            return Err(AppError::DestructiveRefused {
                message: "workspace down requires --yes in a non-interactive terminal".to_string(),
            });
        }
        let previous = self.read_state(&identity)?;
        if dry_run {
            return Ok(ReconcileReport {
                schema_version: crate::util::schema_version(),
                identity,
                dry_run: true,
                steps: vec!["stop-sandbox".to_string(), "remove-sandbox".to_string()],
                state: previous
                    .map(|state| state.state)
                    .unwrap_or(SandboxState::Absent),
            });
        }
        let guard = self
            .ctx
            .state_store
            .begin_mutation(&identity)
            .map_err(state_error)?;
        self.runtime_status(self.ctx.runtime_adapter.stop_workspace(&identity))?;
        self.runtime_status(self.ctx.runtime_adapter.remove_workspace(&identity))?;
        self.write_state(&identity, SandboxState::Stopped, None, &*guard)?;
        Ok(ReconcileReport {
            schema_version: crate::util::schema_version(),
            identity,
            dry_run: false,
            steps: vec!["stop-sandbox".to_string(), "remove-sandbox".to_string()],
            state: SandboxState::Stopped,
        })
    }

    pub(crate) fn status(&self) -> Result<WorkspaceStatusReport, AppError> {
        let identity = self.identity()?;
        let persisted = self.read_state(&identity)?;
        let state = persisted
            .as_ref()
            .map(|state| state.state)
            .unwrap_or(SandboxState::Absent);
        let compose = self.compose();
        let image = match &compose {
            Ok(output) => output.composed.image.clone(),
            Err(_) => None,
        };
        let image_freshness = image
            .as_deref()
            .and_then(|name| ImageBuildService::new(self.ctx).status(name).ok().flatten())
            .map(|report| {
                if report.reused {
                    format!("fresh:{}", report.digest)
                } else {
                    format!("rebuilt:{}", report.digest)
                }
            });
        let effective_allow = self.effective_allow_for_status(&compose);
        let drift = self.compute_drift(persisted.as_ref(), compose.as_ref(), &effective_allow);
        let relaxations = self.relaxations(&effective_allow);
        let config_sources = if self.ctx.config.source_files.is_empty() {
            vec!["defaults".to_string()]
        } else {
            self.ctx
                .config
                .source_files
                .iter()
                .map(ToString::to_string)
                .collect()
        };
        Ok(WorkspaceStatusReport {
            schema_version: crate::util::schema_version(),
            identity,
            state,
            manifest: self.manifest_name(),
            config_sources,
            image,
            image_freshness,
            drift,
            operation_in_progress: false,
            runtime_profile: self.ctx.config.config.runtime.profile.clone(),
            hardening_posture: "drop-all-caps,no-new-privileges,tmpfs-tmp".to_string(),
            network_policy: self.ctx.config.config.network.egress.clone(),
            credential_policy: "ephemeral".to_string(),
            last_reconcile: persisted
                .as_ref()
                .and_then(|state| state.reconcile_fingerprint.as_ref())
                .map(|digest| digest.as_str().to_string()),
            relaxations,
            allow: effective_allow,
        })
    }

    pub(crate) fn network_show(&self) -> Result<NetworkShowReport, AppError> {
        let user = Allowlist(
            self.ctx
                .config
                .config
                .network
                .allow
                .iter()
                .cloned()
                .map(AllowEntry)
                .collect(),
        )
        .validate()
        .map_err(AppError::validation)?;
        // Propagate compose/manifest/layer failures instead of masking them as an
        // empty manifest allowlist: silently reporting a stricter `deny` policy than
        // the one `reconcile` would apply would misrepresent the effective S4 egress
        // posture to anyone auditing it.
        let manifest = Allowlist(
            self.compose()?
                .composed
                .network
                .allow
                .into_iter()
                .map(AllowEntry)
                .collect(),
        );
        let effective = manifest
            .union(&user)
            .validate()
            .map_err(AppError::validation)?;
        let effective_allow: Vec<String> = effective.0.into_iter().map(|entry| entry.0).collect();
        Ok(NetworkShowReport {
            schema_version: crate::util::schema_version(),
            posture: "deny",
            relaxation: (!effective_allow.is_empty()).then(|| "network.allow".to_string()),
            effective_allow,
            manifest_allow: manifest.0.into_iter().map(|entry| entry.0).collect(),
            user_allow: user.0.into_iter().map(|entry| entry.0).collect(),
        })
    }

    fn compose(&self) -> Result<ComposeOutput, AppError> {
        let name = self.manifest_name();
        let (manifest, raw) = self.read_manifest(&name)?;
        let layers = self.read_layers(&manifest)?;
        ComposeService::new(&self.ctx.roots).compose(ComposeInput {
            manifest_name: name,
            manifest_content: raw,
            manifest,
            layers,
        })
    }

    fn effective_allow(&self, manifest_allow: Vec<String>) -> Result<Vec<String>, AppError> {
        let manifest = Allowlist(manifest_allow.into_iter().map(AllowEntry).collect());
        let user = Allowlist(
            self.ctx
                .config
                .config
                .network
                .allow
                .iter()
                .cloned()
                .map(AllowEntry)
                .collect(),
        );
        Ok(manifest
            .union(&user)
            .validate()
            .map_err(AppError::validation)?
            .0
            .into_iter()
            .map(|entry| entry.0)
            .collect())
    }

    fn read_manifest(&self, name: &str) -> Result<(ManifestDocument, String), AppError> {
        let path = self.ctx.roots.manifests_dir().join(format!("{name}.toml"));
        let raw = fs::read_to_string(&path).map_err(|err| AppError::io(path.clone(), err))?;
        let manifest: ManifestDocument = toml::from_str(&raw)
            .map_err(|err| AppError::validation(format!("manifest `{name}` is invalid: {err}")))?;
        Ok((manifest, raw))
    }

    fn read_layers(&self, manifest: &ManifestDocument) -> Result<Vec<LayerInput>, AppError> {
        let mut layers = Vec::new();
        for LayerRef(name) in &manifest.layers {
            let path = self
                .ctx
                .roots
                .layers_dir()
                .join(name)
                .join("devcontainer.json");
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

    fn ensure_image(&self, image: &str) -> Result<(), AppError> {
        let image_dir = self.ctx.roots.config.join("images").join(image);
        if !image_dir.join("devcontainer.json").exists() {
            return Ok(());
        }
        let pull_policy = pull_policy_from_config(&self.ctx.config.config.images.pull_policy);
        match ImageBuildService::new(self.ctx).build(ImageBuildInput {
            name: image.to_string(),
            full_rebuild: false,
            pull_policy,
            dry_run: false,
        })? {
            ImageBuildOutcome::Report(_) => Ok(()),
            ImageBuildOutcome::ChildExit(_) => Err(AppError::Build {
                message: format!(
                    "image `{image}` is stale and could not be rebuilt; \
                    run `podbox image build {image}`"
                ),
            }),
        }
    }

    fn identity(&self) -> Result<WorkspaceIdentity, AppError> {
        let path = self.workspace_path()?;
        let label = path
            .file_name()
            .filter(|name| !name.is_empty())
            .unwrap_or("workspace")
            .to_string();
        Ok(WorkspaceIdentity {
            path: WorkspacePath(path.to_string()),
            label: WorkspaceLabel(label),
        })
    }

    fn workspace_path(&self) -> Result<Utf8PathBuf, AppError> {
        if let Some(path) = &self.ctx.global.workspace {
            return Utf8PathBuf::from_path_buf(path.clone()).map_err(|path| {
                AppError::usage(format!("workspace path is not UTF-8: {}", path.display()))
            });
        }
        Utf8PathBuf::from_path_buf(self.ctx.cwd.clone()).map_err(|path| {
            AppError::usage(format!("workspace path is not UTF-8: {}", path.display()))
        })
    }

    fn manifest_name(&self) -> String {
        self.ctx
            .global
            .manifest
            .clone()
            .unwrap_or_else(|| self.ctx.config.config.defaults.manifest.clone())
    }

    fn require_config(&self) -> Result<(), AppError> {
        if self.ctx.config.source_files.is_empty() {
            return Err(AppError::UsageOrConfigSyntax {
                message: format!(
                    "no config for `{}`; run `podbox init` or pass `--config`",
                    self.workspace_path()?
                ),
            });
        }
        Ok(())
    }

    fn read_state(&self, identity: &WorkspaceIdentity) -> Result<Option<WorkspaceState>, AppError> {
        self.ctx.state_store.read(identity).map_err(state_error)
    }

    fn write_state(
        &self,
        identity: &WorkspaceIdentity,
        state: SandboxState,
        reconcile_fingerprint: Option<Digest>,
        guard: &dyn StateMutationGuard,
    ) -> Result<(), AppError> {
        self.ctx
            .state_store
            .write_locked(
                &WorkspaceState {
                    identity: identity.clone(),
                    state,
                    failure: None,
                    image_freshness: None,
                    reconcile_fingerprint,
                },
                guard,
            )
            .map_err(state_error)
    }

    /// Record a `Failed` state under an already-held mutation guard. Uses the
    /// guard (not `mark_failed`, which re-acquires the advisory lock and would
    /// deadlock against the guard we already hold).
    fn write_failed(
        &self,
        identity: &WorkspaceIdentity,
        err: &AppError,
        guard: &dyn StateMutationGuard,
    ) -> Result<(), AppError> {
        self.ctx
            .state_store
            .write_locked(
                &WorkspaceState {
                    identity: identity.clone(),
                    state: SandboxState::Failed,
                    failure: Some(err.to_string()),
                    image_freshness: None,
                    reconcile_fingerprint: None,
                },
                guard,
            )
            .map_err(state_error)
    }

    /// Best-effort effective allowlist for read-only reporting (`status`, `up`):
    /// returns the composed + user union, or empty when composition fails.
    fn effective_allow_for_status(&self, compose: &Result<ComposeOutput, AppError>) -> Vec<String> {
        match compose {
            Ok(output) => self
                .effective_allow(output.composed.network.allow.clone())
                .unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    /// Report drift for an established sandbox by comparing the fingerprint of the
    /// current composed inputs against the baseline captured at the last successful
    /// reconcile. A configuration that no longer composes is itself drift.
    fn compute_drift(
        &self,
        persisted: Option<&WorkspaceState>,
        compose: Result<&ComposeOutput, &AppError>,
        effective_allow: &[String],
    ) -> WorkspaceDrift {
        let Some(state) = persisted else {
            return WorkspaceDrift::clean();
        };
        if !matches!(
            state.state,
            SandboxState::Running | SandboxState::Stopped | SandboxState::Drifted
        ) {
            return WorkspaceDrift::clean();
        }
        let Some(baseline) = state.reconcile_fingerprint.as_ref() else {
            // No recorded baseline (pre-fingerprint or failed state): cannot assert
            // drift, so do not fabricate a cause.
            return WorkspaceDrift::clean();
        };
        match compose {
            Ok(output) => {
                let current = reconcile_fingerprint(&output.metadata.digest, effective_allow);
                if &current == baseline {
                    WorkspaceDrift::clean()
                } else {
                    WorkspaceDrift::from_causes(vec![DriftCause(
                        "composed configuration changed since last reconcile; \
                        run `podbox workspace reconcile`"
                            .to_string(),
                    )])
                }
            }
            Err(_) => WorkspaceDrift::from_causes(vec![DriftCause(
                "workspace configuration no longer composes; run `podbox workspace reconcile`"
                    .to_string(),
            )]),
        }
    }

    fn relaxations(&self, effective_allow: &[String]) -> Vec<String> {
        let mut relaxations = Vec::new();
        if !effective_allow.is_empty() {
            relaxations.push("network.allow".to_string());
        }
        if self.ctx.config.config.network.egress != "deny" {
            relaxations.push(format!(
                "network.egress={}",
                self.ctx.config.config.network.egress
            ));
        }
        if self.ctx.config.config.runtime.profile != "microvm" {
            relaxations.push(format!(
                "runtime.profile={}",
                self.ctx.config.config.runtime.profile
            ));
        }
        relaxations
    }

    fn require_running(&self, identity: &WorkspaceIdentity) -> Result<(), AppError> {
        match self.read_state(identity)?.map(|state| state.state) {
            Some(SandboxState::Running) => Ok(()),
            _ => Err(AppError::WorkspaceState {
                message: "workspace sandbox is not running; run `podbox workspace up`".to_string(),
            }),
        }
    }

    fn runtime_status(
        &self,
        result: Result<
            crate::adapters::runtime::RuntimeStatus,
            crate::adapters::runtime::RuntimeError,
        >,
    ) -> Result<(), AppError> {
        let status = result.map_err(|err| err.into_app_error())?;
        if status.code == 0 {
            Ok(())
        } else {
            Err(AppError::HostRuntime {
                message: format!("runtime command failed with exit {}", status.code),
            })
        }
    }
}

/// Fingerprint of the composed inputs that `reconcile` applied, over the
/// composition digest (manifest + layers + image) and the effective allowlist.
/// `status`/`up` recompute it to detect drift since the last reconcile.
fn reconcile_fingerprint(composition: &Digest, effective_allow: &[String]) -> Digest {
    let mut input = DigestInput::new().part("composition", composition.as_str().as_bytes());
    for entry in effective_allow {
        input = input.part("allow", entry.as_bytes());
    }
    input.finish()
}

fn state_error(err: StateStoreError) -> AppError {
    AppError::WorkspaceState {
        message: err.to_string(),
    }
}

fn pull_policy_from_config(value: &str) -> PullPolicy {
    match value {
        "newer" => PullPolicy::Newer,
        "always" => PullPolicy::Always,
        "never" => PullPolicy::Never,
        _ => PullPolicy::Missing,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use camino::Utf8PathBuf;

    use super::*;
    use crate::{
        adapters::runtime::stub::StubRuntimeAdapter,
        cli::GlobalArgs,
        config::{
            loader::LoadedConfig, provenance::Provenance, roots::PodboxRoots, schema::Config,
        },
    };

    fn roots(temp: &tempfile::TempDir) -> PodboxRoots {
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        PodboxRoots {
            config: root.join("config"),
            cache: root.join("cache"),
            state: root.join("state"),
            data: root.join("data"),
        }
    }

    fn write_fixture(roots: &PodboxRoots) {
        std::fs::create_dir_all(roots.manifests_dir().as_std_path()).unwrap();
        std::fs::create_dir_all(roots.layers_dir().join("base").as_std_path()).unwrap();
        std::fs::write(
            roots.config_file().as_std_path(),
            r#"
[defaults]
manifest = "default"

[network]
allow = ["user.example:443"]
"#,
        )
        .unwrap();
        std::fs::write(
            roots.manifests_dir().join("default.toml").as_std_path(),
            r#"
layers = ["base"]

[network]
allow = ["manifest.example:443"]
"#,
        )
        .unwrap();
        std::fs::write(
            roots
                .layers_dir()
                .join("base")
                .join("devcontainer.json")
                .as_std_path(),
            r#"{"image":"alpine:latest","network":{"allow":["layer.example:443"]}}"#,
        )
        .unwrap();
    }

    fn context(roots: PodboxRoots, runtime: Arc<StubRuntimeAdapter>, config: Config) -> AppContext {
        let mut ctx = AppContext::for_test(roots.clone(), runtime);
        ctx.global = GlobalArgs {
            workspace: Some(std::path::PathBuf::from("/workspaces/demo")),
            ..ctx.global
        };
        ctx.config = LoadedConfig {
            config,
            provenance: Provenance::default(),
            source_files: vec![roots.config_file()],
        };
        ctx
    }

    #[test]
    fn reconcile_records_policy_handoff_and_start_on_stub_runtime() {
        let temp = tempfile::tempdir().unwrap();
        let roots = roots(&temp);
        write_fixture(&roots);
        let config = LoadedConfig::load(&roots, None, None).unwrap().config;
        let runtime = Arc::new(StubRuntimeAdapter::exiting(0));
        let ctx = context(roots, runtime.clone(), config);

        let report = WorkspaceLifecycleService::new(&ctx)
            .reconcile(false, true)
            .unwrap();

        assert_eq!(report.schema_version, crate::util::schema_version());
        assert_eq!(report.state, SandboxState::Running);
        let policies = runtime.policies.lock().unwrap();
        assert_eq!(policies.len(), 1);
        let rendered = policies[0].artifact.serialize().unwrap();
        assert!(rendered.contains("manifest.example"));
        assert!(rendered.contains("layer.example"));
        assert!(rendered.contains("user.example"));
        assert_eq!(runtime.starts.lock().unwrap().len(), 1);
    }

    #[test]
    fn exec_on_absent_workspace_is_exit_five_error_kind() {
        let temp = tempfile::tempdir().unwrap();
        let roots = roots(&temp);
        write_fixture(&roots);
        let config = LoadedConfig::load(&roots, None, None).unwrap().config;
        let ctx = context(roots, Arc::new(StubRuntimeAdapter::exiting(0)), config);

        let err = WorkspaceLifecycleService::new(&ctx)
            .exec(vec!["echo".to_string(), "ok".to_string()])
            .unwrap_err();

        assert_eq!(err.kind(), "workspace_state");
        assert_eq!(err.exit_code(), crate::exit::WORKSPACE_STATE);
    }

    #[test]
    fn status_reports_drift_when_composed_config_changes_after_reconcile() {
        let temp = tempfile::tempdir().unwrap();
        let roots = roots(&temp);
        write_fixture(&roots);
        let config = LoadedConfig::load(&roots, None, None).unwrap().config;
        let runtime = Arc::new(StubRuntimeAdapter::exiting(0));
        let ctx = context(roots.clone(), runtime, config);
        let service = WorkspaceLifecycleService::new(&ctx);

        service.reconcile(false, true).unwrap();
        // A clean status right after reconcile reports no drift.
        assert!(!service.status().unwrap().drift.drifted);

        // Change the composed inputs (manifest allowlist) after reconcile.
        std::fs::write(
            roots.manifests_dir().join("default.toml").as_std_path(),
            r#"
layers = ["base"]

[network]
allow = ["changed.example:443"]
"#,
        )
        .unwrap();

        let report = service.status().unwrap();
        assert!(report.drift.drifted, "expected drift after config change");
        assert!(
            report
                .drift
                .causes
                .iter()
                .any(|cause| cause.0.contains("reconcile")),
            "drift cause should point to reconcile: {:?}",
            report.drift.causes
        );
    }

    #[test]
    fn dry_run_reconcile_reports_schema_and_does_not_call_runtime() {
        let temp = tempfile::tempdir().unwrap();
        let roots = roots(&temp);
        write_fixture(&roots);
        let config = LoadedConfig::load(&roots, None, None).unwrap().config;
        let runtime = Arc::new(StubRuntimeAdapter::exiting(0));
        let ctx = context(roots, runtime.clone(), config);

        let report = WorkspaceLifecycleService::new(&ctx)
            .reconcile(true, false)
            .unwrap();

        assert!(report.dry_run);
        assert_eq!(report.schema_version, crate::util::schema_version());
        assert_eq!(runtime.starts.lock().unwrap().len(), 0);
        assert_eq!(runtime.policies.lock().unwrap().len(), 0);
    }
}
