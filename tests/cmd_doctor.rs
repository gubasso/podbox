mod support;

#[test]
fn doctor_skeleton_is_unknown_not_implemented() {
    let env = support::TestEnv::new();
    let assert = env.cmd().args(["doctor", "--json"]).assert().success();
    let stdout = support::stdout_utf8(assert);
    assert!(stdout.contains("\"catalog_status\": \"not_implemented\""));
    assert!(stdout.contains("\"status\": \"unknown\""));
}
