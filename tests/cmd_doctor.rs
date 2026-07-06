mod support;

#[test]
fn doctor_catalog_json_has_counts_and_unknown_runtime_checks() {
    let env = support::TestEnv::new();
    let assert = env.cmd().args(["doctor", "--json"]).assert().success();
    let stdout = support::stdout_utf8(assert);
    assert!(stdout.contains("\"schema_version\": 1"));
    assert!(stdout.contains("\"counts\""));
    assert!(stdout.contains("\"id\": \"RT-ROOTLESS\""));
    assert!(stdout.contains("\"status\": \"unknown\""));
}
