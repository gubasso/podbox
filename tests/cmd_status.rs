mod support;

use predicates::prelude::*;
use support::{TestEnv, write};

fn fixture(env: &TestEnv) {
    write(
        env.path("config/config.toml"),
        r#"
[defaults]
manifest = "default"

[network]
allow = ["user.example:443"]
"#,
    );
    write(
        env.path("config/manifests/default.toml"),
        r#"
layers = ["base"]
"#,
    );
    write(
        env.path("config/devcontainer/base/devcontainer.json"),
        r#"{"image":"alpine:latest"}"#,
    );
}

#[test]
fn status_json_reports_schema_state_config_and_relaxations() {
    let env = TestEnv::new();
    fixture(&env);

    env.cmd()
        .args(["--json", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""schema_version": 1"#))
        .stdout(predicate::str::contains(r#""state": "absent""#))
        .stdout(predicate::str::contains(r#""runtime_profile": "microvm""#))
        .stdout(predicate::str::contains("network.allow"));
}

#[test]
fn workspace_status_uses_same_report_path() {
    let env = TestEnv::new();
    fixture(&env);

    env.cmd()
        .args(["workspace", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace:"))
        .stdout(predicate::str::contains("relaxations: network.allow"));
}
