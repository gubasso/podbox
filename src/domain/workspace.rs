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
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct DriftCause(pub(crate) String);
