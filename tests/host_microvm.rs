mod support;

#[test]
#[ignore = "requires PODBOX_E2E=1, Linux KVM, and a configured rootless microVM runtime"]
fn host_microvm_doctor_runtime_scope_reports_ready_or_actionable_failure() {
    if std::env::var("PODBOX_E2E").as_deref() != Ok("1") {
        return;
    }
    let env = support::TestEnv::new();
    let _assert = env
        .cmd()
        .args(["doctor", "--scope", "runtime", "--json"])
        .assert();
}
