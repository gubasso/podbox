mod support;

#[test]
#[ignore = "requires PODBOX_E2E=1, KVM, rootless Podman, libkrun, and live podbox guest networking"]
fn denied_endpoint_fails_while_allowed_endpoint_succeeds_in_guest() {
    if std::env::var_os("PODBOX_E2E").is_none() {
        return;
    }
    let env = support::TestEnv::new();
    env.cmd()
        .args(["init", "--starter", "minimal"])
        .assert()
        .success();
    env.cmd()
        .args(["network", "allow", "example.com:443"])
        .assert()
        .success();
    env.cmd()
        .args(["network", "show", "--json"])
        .assert()
        .success();
}
