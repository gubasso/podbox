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

#[test]
#[ignore = "requires PODBOX_E2E=1, Linux KVM, rootless Podman, libkrun, and the live guest agent"]
fn host_full_product_flow_launches_hardened_microvm() {
    if std::env::var("PODBOX_E2E").as_deref() != Ok("1") {
        return;
    }
    let env = support::TestEnv::new();
    env.cmd()
        .args(["init", "--starter", "minimal"])
        .assert()
        .success();
    env.cmd()
        .args(["doctor", "--scope", "all"])
        .assert()
        .success();
    env.cmd()
        .args(["manifest", "compose", "minimal"])
        .assert()
        .success();
    env.cmd()
        .args(["workspace", "up", "--manifest", "minimal"])
        .assert()
        .success();
    env.cmd()
        .args([
            "shell",
            "--manifest",
            "minimal",
            "--reconcile",
            "always",
            "--",
            "echo",
            "podbox-e2e",
        ])
        .assert()
        .success();
    env.cmd()
        .args(["workspace", "exec", "--", "echo", "podbox-exec"])
        .assert()
        .success();
    env.cmd().args(["network", "show"]).assert().success();
    env.cmd().arg("status").assert().success();
    env.cmd()
        .args(["workspace", "reconcile", "--yes"])
        .assert()
        .success();
    env.cmd()
        .args(["workspace", "down", "--force"])
        .assert()
        .success();
}
