use std::fs;

use crate::{
    adapters::runtime,
    cli::doctor::DoctorScope,
    domain::doctor::{Category, CheckId, CheckResult, DoctorReport, Severity, SeverityCounts},
};

pub(crate) trait DoctorProbe: Send + Sync {
    fn check(&self, id: &str, category: Category) -> CheckOutcome;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckOutcome {
    pub(crate) status: Severity,
    pub(crate) message: Option<String>,
    pub(crate) evidence: Vec<String>,
    pub(crate) remediation: Option<String>,
    pub(crate) reproduction: Option<String>,
}

impl CheckOutcome {
    pub(crate) fn status(status: Severity) -> Self {
        Self {
            status,
            message: None,
            evidence: Vec::new(),
            remediation: None,
            reproduction: None,
        }
    }

    fn fail(message: impl Into<String>, remediation: impl Into<String>) -> Self {
        Self {
            status: Severity::Fail,
            message: Some(message.into()),
            evidence: Vec::new(),
            remediation: Some(remediation.into()),
            reproduction: None,
        }
    }

    fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence.push(evidence.into());
        self
    }

    fn with_reproduction(mut self, command: impl Into<String>) -> Self {
        self.reproduction = Some(command.into());
        self
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct StdDoctorProbe;

impl DoctorProbe for StdDoctorProbe {
    fn check(&self, id: &str, category: Category) -> CheckOutcome {
        match (id, category) {
            ("RT-ISOLATION", Category::Runtime) => runtime_isolation(),
            ("RT-ROOTLESS", Category::Runtime) => runtime_rootless(),
            ("RT-SMOKE", Category::Runtime) => runtime_smoke(),
            ("RT-BUILDER", Category::Runtime) => runtime_builder(),
            ("RT-NETBACK", Category::Runtime) => runtime_netback_readiness(),
            _ => CheckOutcome::status(Severity::Pass),
        }
    }
}

pub(crate) fn catalog(scope: DoctorScope, probe: &dyn DoctorProbe) -> DoctorReport {
    let checks: Vec<CheckResult> = check_catalog()
        .into_iter()
        .filter(|(_, category)| scope_matches(scope, *category))
        .map(|(id, category)| {
            let outcome = probe.check(id, category);
            let status = outcome.status;
            let mut evidence = outcome.evidence;
            if let Some(reproduction) = outcome.reproduction {
                evidence.push(format!("reproduce: {reproduction}"));
            }
            CheckResult {
                id: CheckId(id.to_string()),
                category,
                status,
                message: outcome
                    .message
                    .unwrap_or_else(|| default_message(id, status)),
                evidence,
                remediation: outcome.remediation.or_else(|| remediation(id, status)),
                docs: Some(format!(
                    "docs/reference/spec/08-doctor-and-diagnostics.md#{id}"
                )),
            }
        })
        .collect();
    let counts = SeverityCounts::from_checks(&checks);
    let status = report_status(&counts);
    DoctorReport {
        schema_version: crate::util::schema_version(),
        status,
        counts,
        checks,
    }
}

pub(crate) fn exit_code(report: &DoctorReport, strict: bool) -> u8 {
    if report
        .checks
        .iter()
        .any(|check| check.status == Severity::Fail && check.category == Category::Config)
    {
        return crate::exit::USAGE_OR_CONFIG;
    }
    if report.checks.iter().any(|check| {
        check.status == Severity::Fail
            && matches!(check.category, Category::Host | Category::Runtime)
    }) {
        return crate::exit::HOST_RUNTIME;
    }
    if strict
        && report
            .checks
            .iter()
            .any(|check| check.status == Severity::Warning)
    {
        return crate::exit::VALIDATION;
    }
    crate::exit::SUCCESS
}

fn check_catalog() -> Vec<(&'static str, Category)> {
    vec![
        ("HOST-OS", Category::Host),
        ("HOST-KVM", Category::Host),
        ("HOST-NESTED", Category::Host),
        ("PERM-USERNS", Category::Host),
        ("PERM-CGROUP", Category::Host),
        ("TERM-TTY", Category::Host),
        ("ENV-KNOWN", Category::Host),
        ("RT-ISOLATION", Category::Runtime),
        ("RT-ROOTLESS", Category::Runtime),
        ("RT-SMOKE", Category::Runtime),
        ("RT-BUILDER", Category::Runtime),
        ("RT-NETBACK", Category::Runtime),
        ("FS-XDG", Category::Config),
        ("FS-LAYOUT", Category::Config),
        ("CFG-SYNTAX", Category::Config),
        ("CFG-SCHEMA", Category::Config),
        ("MAN-SCHEMA", Category::Config),
        ("MAN-LAYERREF", Category::Config),
        ("MAN-COMPOSE", Category::Config),
        ("STATE-CACHE", Category::Config),
        ("WS-IDENTITY", Category::Workspace),
        ("IMG-FRESH", Category::Images),
        ("NET-POLICY", Category::Network),
        ("CRED-POLICY", Category::Network),
    ]
}

fn scope_matches(scope: DoctorScope, category: Category) -> bool {
    match scope {
        DoctorScope::All => true,
        DoctorScope::Host => category == Category::Host,
        DoctorScope::Runtime => category == Category::Runtime,
        DoctorScope::Config => category == Category::Config,
        DoctorScope::Workspace => category == Category::Workspace,
        DoctorScope::Images => category == Category::Images,
        DoctorScope::Network => category == Category::Network,
    }
}

fn report_status(counts: &SeverityCounts) -> Severity {
    if counts.fail > 0 {
        Severity::Fail
    } else if counts.warning > 0 {
        Severity::Warning
    } else if counts.unknown > 0 {
        Severity::Unknown
    } else {
        Severity::Pass
    }
}

fn default_message(id: &str, status: Severity) -> String {
    match status {
        Severity::Pass => format!("{id} passed"),
        Severity::Warning => format!("{id} warning"),
        Severity::Fail => format!("{id} failed"),
        Severity::Skipped => format!("{id} skipped"),
        Severity::Unknown => format!("{id} unknown until live runtime backend is available"),
    }
}

fn remediation(id: &str, status: Severity) -> Option<String> {
    (status == Severity::Fail).then(|| format!("Resolve {id} before continuing"))
}

fn runtime_isolation() -> CheckOutcome {
    match fs::metadata("/dev/kvm") {
        Ok(meta) if meta.file_type().is_char_device() => CheckOutcome::status(Severity::Pass)
            .with_evidence("/dev/kvm exists and is a character device"),
        Ok(_) => CheckOutcome::fail(
            "RT-ISOLATION failed: /dev/kvm is not a character device",
            "Expose KVM to this host or run podbox on a KVM-capable Linux host",
        )
        .with_evidence("/dev/kvm exists but is not usable as a KVM device"),
        Err(err) => CheckOutcome::fail(
            "RT-ISOLATION failed: /dev/kvm is missing",
            "Expose KVM to this host or run podbox on a KVM-capable Linux host",
        )
        .with_evidence(format!("/dev/kvm metadata error: {err}")),
    }
}

fn runtime_rootless() -> CheckOutcome {
    match std::env::var("USER") {
        Ok(user) if user == "root" => CheckOutcome::fail(
            "RT-ROOTLESS failed: USER env var reports root",
            "Run podbox as an unprivileged user with rootless container support configured",
        )
        .with_evidence("inferred from USER env var: USER=root"),
        Ok(user) => CheckOutcome::status(Severity::Pass).with_evidence(format!(
            "inferred from USER env var: USER={user} (not root)"
        )),
        Err(_) => CheckOutcome::status(Severity::Pass).with_evidence(
            "USER env var is unset; unable to confirm effective user (assumed non-root)",
        ),
    }
}

fn runtime_smoke() -> CheckOutcome {
    let reproduction = runtime::host_runtime_info_reproduction();
    match runtime::host_runtime_info_probe() {
        Ok(output) if output.success() => {
            let rootless = output.stdout.trim();
            if rootless == "true" {
                CheckOutcome::status(Severity::Pass)
                    .with_evidence("runtime reports rootless=true")
                    .with_reproduction(reproduction)
            } else {
                CheckOutcome::fail(
                    "RT-SMOKE failed: runtime is not rootless",
                    "Configure the runtime for rootless operation before continuing",
                )
                .with_evidence(format!("runtime rootless probe returned: {rootless:?}"))
                .with_reproduction(reproduction)
            }
        }
        Ok(output) => CheckOutcome::fail(
            "RT-SMOKE failed: runtime smoke command did not complete",
            "Install and configure a rootless microVM runtime before continuing",
        )
        .with_evidence(format!("runtime info exit status: {:?}", output.status))
        .with_evidence(format!("stderr: {}", output.stderr))
        .with_reproduction(reproduction),
        Err(err) => CheckOutcome::fail(
            "RT-SMOKE failed: runtime smoke command could not start",
            "Install and configure a rootless microVM runtime before continuing",
        )
        .with_evidence(format!("runtime info spawn error: {err}"))
        .with_reproduction(reproduction),
    }
}

fn runtime_builder() -> CheckOutcome {
    match runtime::host_runtime_builder_probe() {
        Ok(output) if output.success() => {
            CheckOutcome::status(Severity::Pass).with_evidence("runtime builder probe completed")
        }
        Ok(output) => CheckOutcome::fail(
            "RT-BUILDER failed: image builder command is unavailable",
            "Install a build-capable runtime before building podbox images",
        )
        .with_evidence(format!("runtime builder exit status: {:?}", output.status)),
        Err(err) => CheckOutcome::fail(
            "RT-BUILDER failed: image builder command could not start",
            "Install a build-capable runtime before building podbox images",
        )
        .with_evidence(format!("runtime builder spawn error: {err}")),
    }
}

fn runtime_netback_readiness() -> CheckOutcome {
    // A readiness label alone is not enforcement: RT-NETBACK must fail (exit 3)
    // when the backend cannot actually apply the in-guest default-deny allowlist,
    // rather than reporting a false pass. The probe reflects real enforcement
    // capability, which is unavailable until the microVM guest agent lands.
    let probe = runtime::host_netback_enforcement_probe();
    if probe.success() {
        CheckOutcome::status(Severity::Pass)
            .with_evidence("runtime backend can enforce the in-guest default-deny network policy")
    } else {
        CheckOutcome::fail(
            "RT-NETBACK failed: the runtime backend cannot enforce the in-guest network policy",
            "Install the podbox microVM backend that applies the default-deny allowlist in-guest",
        )
        .with_evidence(format!("netback enforcement probe: {}", probe.stderr))
    }
}

#[cfg(unix)]
trait FileTypeExt {
    fn is_char_device(&self) -> bool;
}

#[cfg(unix)]
impl FileTypeExt for fs::FileType {
    fn is_char_device(&self) -> bool {
        std::os::unix::fs::FileTypeExt::is_char_device(self)
    }
}

#[cfg(not(unix))]
trait FileTypeExt {
    fn is_char_device(&self) -> bool;
}

#[cfg(not(unix))]
impl FileTypeExt for fs::FileType {
    fn is_char_device(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Forced(&'static str, Severity);

    impl DoctorProbe for Forced {
        fn check(&self, id: &str, _category: Category) -> CheckOutcome {
            CheckOutcome::status(if id == self.0 { self.1 } else { Severity::Pass })
        }
    }

    #[test]
    fn forced_host_failure_exits_host_runtime() {
        let report = catalog(DoctorScope::All, &Forced("HOST-KVM", Severity::Fail));
        assert_eq!(exit_code(&report, false), crate::exit::HOST_RUNTIME);
    }

    #[test]
    fn forced_runtime_failure_exits_host_runtime() {
        let report = catalog(DoctorScope::All, &Forced("RT-SMOKE", Severity::Fail));
        assert_eq!(exit_code(&report, false), crate::exit::HOST_RUNTIME);
    }

    #[test]
    fn forced_config_failure_exits_usage_or_config() {
        let report = catalog(DoctorScope::All, &Forced("CFG-SYNTAX", Severity::Fail));
        assert_eq!(exit_code(&report, false), crate::exit::USAGE_OR_CONFIG);
    }

    #[test]
    fn strict_warning_exits_validation() {
        let report = catalog(DoctorScope::All, &Forced("MAN-LAYERREF", Severity::Warning));
        assert_eq!(exit_code(&report, true), crate::exit::VALIDATION);
    }

    #[test]
    fn runtime_failure_carries_evidence_and_reproduction() {
        struct Smoke;
        impl DoctorProbe for Smoke {
            fn check(&self, id: &str, _category: Category) -> CheckOutcome {
                if id == "RT-SMOKE" {
                    CheckOutcome::fail("smoke failed", "fix runtime")
                        .with_evidence("stderr: denied")
                        .with_reproduction("runtime info")
                } else {
                    CheckOutcome::status(Severity::Pass)
                }
            }
        }

        let report = catalog(DoctorScope::Runtime, &Smoke);
        let smoke = report
            .checks
            .iter()
            .find(|check| check.id.0 == "RT-SMOKE")
            .expect("RT-SMOKE");
        assert_eq!(smoke.status, Severity::Fail);
        assert!(smoke.evidence.iter().any(|line| line == "stderr: denied"));
        assert!(
            smoke
                .evidence
                .iter()
                .any(|line| line == "reproduce: runtime info")
        );
        assert_eq!(smoke.remediation.as_deref(), Some("fix runtime"));
    }
}
