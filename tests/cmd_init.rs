mod support;

use predicates::prelude::*;
use support::TestEnv;

#[test]
fn init_minimal_creates_lean_seed_and_runs_doctor_report() {
    let env = TestEnv::new();

    env.cmd()
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("starter: minimal"))
        .stdout(predicate::str::contains("doctor:"));

    assert!(env.path("config/config.toml").exists());
    assert!(env.path("config/manifests/minimal.toml").exists());
    assert!(
        env.path("config/devcontainer/base/devcontainer.json")
            .exists()
    );
    assert!(
        env.path("config/devcontainer/minimal/devcontainer.json")
            .exists()
    );
    assert!(!env.path(".devcontainer/devcontainer.json").exists());
}

#[test]
fn init_none_creates_dirs_and_config_without_sample_layers() {
    let env = TestEnv::new();

    env.cmd()
        .args(["init", "--starter", "none"])
        .assert()
        .success();

    assert!(env.path("config/config.toml").exists());
    assert!(env.path("config/manifests").exists());
    assert!(env.path("config/devcontainer").exists());
    assert!(!env.path("config/manifests/minimal.toml").exists());
}

#[test]
fn init_dry_run_reports_without_mutation_and_skips_doctor() {
    let env = TestEnv::new();

    env.cmd()
        .args(["init", "--dry-run", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""dry_run": true"#))
        .stdout(predicate::str::contains(r#""doctor": null"#));

    assert!(!env.path("config/config.toml").exists());
}
