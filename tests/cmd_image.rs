mod support;

#[test]
fn image_build_fails_closed_when_live_runtime_is_missing() {
    let env = support::TestEnv::new();
    support::write(
        env.path("config/images/demo/devcontainer.json"),
        r#"{"image":"alpine:latest"}"#,
    );
    env.cmd()
        .args(["image", "build", "demo"])
        .assert()
        .code(3)
        .stderr(predicates::str::contains("runtime backend executable"));
}

#[test]
fn image_build_dry_run_reports_decision_without_touching_runtime_or_cache() {
    let env = support::TestEnv::new();
    support::write(
        env.path("config/images/demo/devcontainer.json"),
        r#"{"image":"alpine:latest"}"#,
    );
    // A live-runtime-missing host fails a real `image build` with exit 3; under
    // --dry-run the build must succeed by reporting the rebuild decision without
    // invoking the runtime or writing freshness metadata.
    let output = env
        .cmd()
        .args(["image", "build", "demo", "--dry-run", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    // Stdout under --json must be exactly one well-formed JSON document.
    let doc: serde_json::Value = serde_json::from_slice(&output).expect("single JSON document");
    assert_eq!(doc["images"].as_array().unwrap().len(), 1);
    assert_eq!(doc["images"][0]["reused"], false);
    // 06 §5: dry-run JSON must name why each target would rebuild/reuse.
    assert!(
        doc["images"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("rebuild"),
        "dry-run report must carry a rebuild/reuse reason: {doc}"
    );
    assert!(!env.path("cache/images/demo/metadata.json").exists());
}

#[test]
fn image_build_multi_target_json_is_a_single_document() {
    let env = support::TestEnv::new();
    for name in ["a", "b"] {
        support::write(
            env.path(format!("config/images/{name}/devcontainer.json")),
            r#"{"image":"alpine:latest"}"#,
        );
    }
    let output = env
        .cmd()
        .args(["image", "build", "a", "b", "--dry-run", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let doc: serde_json::Value = serde_json::from_slice(&output).expect("single JSON document");
    let images = doc["images"].as_array().unwrap();
    assert_eq!(images.len(), 2);
    assert_eq!(images[0]["image"], "a");
    assert_eq!(images[1]["image"], "b");
    // Each target carries its own dry-run rebuild/reuse rationale (06 §5).
    assert!(images[0]["reason"].is_string());
    assert!(images[1]["reason"].is_string());
}

#[test]
fn image_build_dry_run_reports_rebuild_on_unreadable_metadata() {
    let env = support::TestEnv::new();
    support::write(
        env.path("config/images/demo/devcontainer.json"),
        r#"{"image":"alpine:latest"}"#,
    );
    // Corrupt cached freshness proof: freshness cannot be proven, so dry-run must
    // still succeed and report a rebuild with a reason (06 §4/§5), not error.
    support::write(env.path("cache/images/demo/metadata.json"), "{ not json");
    let output = env
        .cmd()
        .args(["image", "build", "demo", "--dry-run", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let doc: serde_json::Value = serde_json::from_slice(&output).expect("single JSON document");
    assert_eq!(doc["images"][0]["reused"], false);
    assert!(
        doc["images"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("unreadable"),
        "reason must explain unprovable freshness: {doc}"
    );
}

#[test]
fn image_build_rejects_names_with_all() {
    let env = support::TestEnv::new();
    env.cmd()
        .args(["image", "build", "demo", "--all"])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("not both"));
}

#[test]
fn image_prune_requires_yes_and_preserves_sources() {
    let env = support::TestEnv::new();
    support::write(
        env.path("config/images/demo/devcontainer.json"),
        r#"{"image":"alpine:latest"}"#,
    );
    env.cmd()
        .args(["image", "prune"])
        .assert()
        .code(7)
        .stderr(predicates::str::contains("requires --yes"));
    assert!(env.path("config/images/demo/devcontainer.json").exists());
}

#[test]
fn image_list_and_inspect_are_public_surface() {
    let env = support::TestEnv::new();
    support::write(
        env.path("config/images/demo/devcontainer.json"),
        r#"{"image":"alpine:latest"}"#,
    );
    env.cmd()
        .args(["image", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("demo"));
    env.cmd()
        .args(["image", "inspect", "demo"])
        .assert()
        .code(6)
        .stderr(predicates::str::contains("not built"));
}
