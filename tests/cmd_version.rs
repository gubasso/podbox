mod support;

use predicates::prelude::*;

#[test]
fn version_json_is_schema_object() {
    let env = support::TestEnv::new();
    let assert = env.cmd().args(["version", "--json"]).assert().success();
    let stdout = support::stdout_utf8(assert);
    let mut value: serde_json::Value =
        serde_json::from_str(&stdout).expect("version --json emits valid JSON");
    assert!(value.get("schema_version").is_some());
    assert!(value.get("build_sha").is_some());
    // `version` is derived from Cargo.toml at compile time. Validate it against
    // the manifest (the single source of truth) here, then pin it to a constant
    // in the snapshot below so routine release version bumps never churn this
    // test — the snapshot keeps locking the schema shape, not the version value.
    assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
    value["version"] = serde_json::Value::String("[cargo-pkg-version]".into());
    // editorconfig-checker-disable
    // The inline snapshot body must keep insta's relative indentation so it
    // dedents to serde_json's 2-space output; do not reflow to 4-space steps.
    insta::assert_json_snapshot!(value, @r###"
    {
      "build_date": "unknown",
      "build_sha": "unknown",
      "schema_version": 1,
      "version": "[cargo-pkg-version]"
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
