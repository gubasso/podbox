mod support;

use predicates::prelude::*;

#[test]
fn version_json_is_schema_object() {
    let env = support::TestEnv::new();
    let assert = env.cmd().args(["version", "--json"]).assert().success();
    let stdout = support::stdout_utf8(assert);
    assert!(stdout.contains("\"schema_version\""));
    assert!(stdout.contains("\"build_sha\""));
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
    // editorconfig-checker-disable
    // The inline snapshot body must keep insta's relative indentation so it
    // dedents to serde_json's 2-space output; do not reflow to 4-space steps.
    insta::assert_json_snapshot!(serde_json::from_str::<serde_json::Value>(&stdout).unwrap(), @r###"
    {
      "build_date": "unknown",
      "build_sha": "unknown",
      "schema_version": 1,
      "version": "0.1.0"
    }
    "###);
    // editorconfig-checker-enable
}

#[test]
fn bad_flag_and_quiet_verbose_are_usage_errors() {
    let env = support::TestEnv::new();
    env.cmd().arg("--badflag").assert().code(2);
    env.cmd()
        .args(["-q", "-v", "version"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("quiet"));
}
