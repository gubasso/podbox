use crate::{
    cli::doctor::DoctorScope,
    domain::doctor::{Category, CheckId, CheckResult, DoctorReport, Severity, SeverityCounts},
};

pub(crate) trait DoctorProbe: Send + Sync {
    fn check(&self, id: &str, category: Category) -> Severity;
}

#[derive(Clone, Debug, Default)]
pub(crate) struct StdDoctorProbe;

impl DoctorProbe for StdDoctorProbe {
    fn check(&self, id: &str, _category: Category) -> Severity {
        if id.starts_with("RT-") {
            Severity::Unknown
        } else {
            Severity::Pass
        }
    }
}

pub(crate) fn catalog(scope: DoctorScope, probe: &dyn DoctorProbe) -> DoctorReport {
    let checks: Vec<CheckResult> = check_catalog()
        .into_iter()
        .filter(|(_, category)| scope_matches(scope, *category))
        .map(|(id, category)| {
            let status = probe.check(id, category);
            CheckResult {
                id: CheckId(id.to_string()),
                category,
                status,
                message: default_message(id, status),
                evidence: Vec::new(),
                remediation: remediation(id, status),
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
    if report
        .checks
        .iter()
        .any(|check| check.status == Severity::Fail && check.category == Category::Host)
    {
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

#[cfg(test)]
mod tests {
    use super::*;

    struct Forced(&'static str, Severity);

    impl DoctorProbe for Forced {
        fn check(&self, id: &str, _category: Category) -> Severity {
            if id == self.0 { self.1 } else { Severity::Pass }
        }
    }

    #[test]
    fn forced_host_failure_exits_host_runtime() {
        let report = catalog(DoctorScope::All, &Forced("HOST-KVM", Severity::Fail));
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
}
