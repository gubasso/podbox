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

[network]
allow = ["manifest.example:443"]
"#,
    );
    write(
        env.path("config/devcontainer/base/devcontainer.json"),
        r#"{"image":"alpine:latest","network":{"allow":["layer.example:443"]}}"#,
    );
}

#[test]
fn network_show_json_reports_schema_and_effective_default_deny_policy() {
    let env = TestEnv::new();
    fixture(&env);

    env.cmd()
        .args(["--json", "network", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""schema_version": 1"#))
        .stdout(predicate::str::contains(r#""posture": "deny""#))
        .stdout(predicate::str::contains("manifest.example:443"))
        .stdout(predicate::str::contains("layer.example:443"))
        .stdout(predicate::str::contains("user.example:443"));
}

#[test]
fn network_allow_validates_and_writes_user_config_idempotently() {
    let env = TestEnv::new();
    fixture(&env);

    env.cmd()
        .args(["network", "allow", "GitHub.COM:443"])
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace reconcile"));
    env.cmd()
        .args(["network", "allow", "github.com:443"])
        .assert()
        .success();

    let config = std::fs::read_to_string(env.path("config/config.toml")).unwrap();
    assert_eq!(config.matches("github.com:443").count(), 1);
}

#[test]
fn network_allow_dedups_against_noncanonical_stored_entry() {
    let env = TestEnv::new();
    // Seed a hand-edited, non-canonical stored entry (mixed case).
    write(
        env.path("config/config.toml"),
        r#"
[defaults]
manifest = "default"

[network]
allow = ["GitHub.COM:443"]
"#,
    );
    write(
        env.path("config/manifests/default.toml"),
        r#"
layers = []
"#,
    );

    env.cmd()
        .args(["network", "allow", "github.com:443"])
        .assert()
        .success();

    let config = std::fs::read_to_string(env.path("config/config.toml")).unwrap();
    // The canonical form already present (as `GitHub.COM:443`) must not be
    // appended a second time.
    assert_eq!(
        config.matches("443").count(),
        1,
        "expected no duplicate allow entry, got: {config}"
    );
}

#[test]
fn network_allow_manifest_target_updates_selected_manifest() {
    let env = TestEnv::new();
    fixture(&env);

    env.cmd()
        .args([
            "--manifest",
            "default",
            "network",
            "allow",
            "docs.example:443",
            "--manifest-target",
        ])
        .assert()
        .success();

    let manifest = std::fs::read_to_string(env.path("config/manifests/default.toml")).unwrap();
    assert!(manifest.contains("docs.example:443"));
}

#[test]
fn network_allow_malformed_entry_exits_four() {
    let env = TestEnv::new();
    fixture(&env);

    env.cmd()
        .args(["network", "allow", "bad host"])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("validation"));
}
