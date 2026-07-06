use serde::{Deserialize, Serialize};

use crate::domain::digest::Digest;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ImageName(pub(crate) String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ImageDefinition {
    pub(crate) name: ImageName,
    pub(crate) context: BuildContext,
    pub(crate) args: Vec<BuildArg>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct BuildContext(pub(crate) String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct SourceGraph {
    pub(crate) digest: Digest,
    pub(crate) pull_policy: PullPolicy,
    pub(crate) files: Vec<SourceGraphFile>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct SourceGraphFile {
    pub(crate) path: String,
    pub(crate) digest: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct BuildArg {
    pub(crate) key: String,
    pub(crate) value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct BaseImageIdentity(pub(crate) String);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PullPolicy {
    Missing,
    Newer,
    Always,
    Never,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct FreshnessProof {
    pub(crate) digest: Digest,
    pub(crate) graph: SourceGraph,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum BuildDecision {
    Reuse(FreshnessProof),
    Build(FreshnessProof),
}
