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
    pub(crate) counts: SeverityCounts,
    pub(crate) checks: Vec<CheckResult>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct SeverityCounts {
    pub(crate) pass: usize,
    pub(crate) warning: usize,
    pub(crate) fail: usize,
    pub(crate) skipped: usize,
    pub(crate) unknown: usize,
}

impl SeverityCounts {
    pub(crate) fn from_checks(checks: &[CheckResult]) -> Self {
        let mut counts = Self::default();
        for check in checks {
            match check.status {
                Severity::Pass => counts.pass += 1,
                Severity::Warning => counts.warning += 1,
                Severity::Fail => counts.fail += 1,
                Severity::Skipped => counts.skipped += 1,
                Severity::Unknown => counts.unknown += 1,
            }
        }
        counts
    }
}
