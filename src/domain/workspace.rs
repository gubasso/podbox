use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct WorkspacePath(pub(crate) String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct WorkspaceLabel(pub(crate) String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct WorkspaceIdentity {
    pub(crate) path: WorkspacePath,
    pub(crate) label: WorkspaceLabel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReconcilePolicy {
    Auto,
    Always,
    Never,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct WorkspaceState {
    pub(crate) identity: WorkspaceIdentity,
    pub(crate) state: crate::domain::state::SandboxState,
    pub(crate) failure: Option<String>,
    pub(crate) image_freshness: Option<crate::domain::image::FreshnessProof>,
    /// Fingerprint of the composed inputs (manifest + layers + effective
    /// allowlist) captured at the last successful reconcile. `status` recomputes
    /// the current fingerprint and reports drift when it diverges. `None` means no
    /// baseline was recorded (e.g. a failed or pre-fingerprint state).
    #[serde(default)]
    pub(crate) reconcile_fingerprint: Option<crate::domain::digest::Digest>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct DriftCause(pub(crate) String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct WorkspaceDrift {
    pub(crate) drifted: bool,
    pub(crate) causes: Vec<DriftCause>,
}

impl WorkspaceDrift {
    pub(crate) fn clean() -> Self {
        Self {
            drifted: false,
            causes: Vec::new(),
        }
    }

    pub(crate) fn from_causes(causes: Vec<DriftCause>) -> Self {
        Self {
            drifted: !causes.is_empty(),
            causes,
        }
    }
}
