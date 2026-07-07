mod support;

use predicates::prelude::*;
use support::{TestEnv, write};

#[test]
fn usage_errors_exit_two() {
    let env = TestEnv::new();
    env.cmd().arg("--badflag").assert().code(2);
}

#[test]
fn validation_errors_exit_four() {
    let env = TestEnv::new();
    write(env.path("config/config.toml"), "[network]\nallow = []\n");
    write(env.path("config/manifests/default.toml"), "layers = []\n");
    env.cmd()
        .args(["manifest", "validate", "default"])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("validation"));
}

#[test]
fn workspace_state_errors_exit_five() {
    let env = TestEnv::new();
    write(
        env.path("config/config.toml"),
        r#"[defaults]
manifest = "default"
"#,
    );
    env.cmd()
        .args(["workspace", "exec", "--", "true"])
        .assert()
        .code(5);
}

#[test]
fn destructive_refusals_exit_seven() {
    let env = TestEnv::new();
    env.cmd().args(["image", "prune"]).assert().code(7);
}
