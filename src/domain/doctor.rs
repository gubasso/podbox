use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub(crate) struct CheckId(pub(crate) String);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Category {
    Host,
    Runtime,
    Config,
    Workspace,
    Images,
    Network,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Severity {
    Pass,
    Warning,
    Fail,
    Skipped,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct CheckResult {
    pub(crate) id: CheckId,
    pub(crate) category: Category,
    pub(crate) status: Severity,
    pub(crate) message: String,
    pub(crate) evidence: Vec<String>,
    pub(crate) remediation: Option<String>,
    pub(crate) docs: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct DoctorReport {
    pub(crate) schema_version: u32,
    pub(crate) status: Severity,
    pub(crate) checks: Vec<CheckResult>,
}
