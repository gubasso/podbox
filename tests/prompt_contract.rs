mod support;

use predicates::prelude::*;
use support::{TestEnv, write};

#[test]
fn destructive_image_prune_requires_yes() {
    let env = TestEnv::new();
    env.cmd()
        .args(["image", "prune"])
        .assert()
        .code(7)
        .stderr(predicate::str::contains("destructive_refused"));
    env.cmd()
        .args(["image", "prune", "--yes"])
        .assert()
        .success();
}

#[test]
fn workspace_down_dry_run_does_not_require_yes() {
    let env = TestEnv::new();
    write(
        env.path("config/config.toml"),
        r#"[defaults]
manifest = "default"
"#,
    );
    env.cmd()
        .args(["workspace", "down", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dry-run").or(predicate::str::contains("state:")));
}
