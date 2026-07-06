mod support;

#[test]
fn image_build_fails_closed_with_unavailable_production_runtime() {
    let env = support::TestEnv::new();
    support::write(
        env.path("config/images/demo/devcontainer.json"),
        r#"{"image":"alpine:latest"}"#,
    );
    env.cmd()
        .args(["image", "build", "demo"])
        .assert()
        .code(3)
        .stderr(predicates::str::contains(
            "live runtime backend is not implemented",
        ));
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
