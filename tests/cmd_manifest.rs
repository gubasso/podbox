mod support;

#[test]
fn manifest_compose_writes_cache_only() {
    let env = support::TestEnv::new();
    support::write(
        env.path("config/manifests/app.toml"),
        "layers = [\"base\", \"leaf\"]\n",
    );
    support::write(
        env.path("config/devcontainer/base/devcontainer.json"),
        r#"{
            "name": "base",
            "mounts": ["base"],
            "containerEnv": {"A": "1"},
            "network": {"allow": ["example.com"]}
        }"#,
    );
    support::write(
        env.path("config/devcontainer/leaf/devcontainer.json"),
        r#"{
            "name": "leaf",
            "mounts": ["leaf"],
            "containerEnv": {"B": "2"},
            "network": {"allow": ["example.com", "api.example.com"]}
        }"#,
    );
    env.cmd()
        .args(["manifest", "validate", "app"])
        .assert()
        .success();
    let assert = env
        .cmd()
        .args(["manifest", "compose", "app", "--json"])
        .assert()
        .success();
    let stdout = support::stdout_utf8(assert);
    assert!(stdout.contains("cache/composed/app/devcontainer.json"));
    assert!(env.path("cache/composed/app/devcontainer.json").is_file());
    assert!(!env.path(".devcontainer/devcontainer.json").exists());
}

#[test]
fn manifest_validate_rejects_invalid_layer_fragment() {
    let env = support::TestEnv::new();
    support::write(
        env.path("config/manifests/app.toml"),
        "layers = [\"base\"]\n",
    );
    // Malformed JSON must be caught by `manifest validate`, not only at compose time.
    support::write(
        env.path("config/devcontainer/base/devcontainer.json"),
        "{ not json",
    );
    env.cmd()
        .args(["manifest", "validate", "app"])
        .assert()
        .failure()
        .code(4);
}

#[test]
fn manifest_level_network_allow_unions_into_composed_output() {
    let env = support::TestEnv::new();
    support::write(
        env.path("config/manifests/app.toml"),
        "layers = [\"base\"]\n[network]\nallow = [\"manifest.example.com\"]\n",
    );
    support::write(
        env.path("config/devcontainer/base/devcontainer.json"),
        r#"{"name":"base","network":{"allow":["layer.example.com"]}}"#,
    );
    env.cmd()
        .args(["manifest", "compose", "app"])
        .assert()
        .success();
    let composed = std::fs::read_to_string(env.path("cache/composed/app/devcontainer.json"))
        .expect("composed");
    let json: serde_json::Value = serde_json::from_str(&composed).expect("composed json");
    // Documented public shape: network.allow, not a top-level network_allow key.
    assert!(
        json.get("network_allow").is_none(),
        "unexpected top-level key: {composed}"
    );
    let allow = json
        .get("network")
        .and_then(|n| n.get("allow"))
        .and_then(|a| a.as_array())
        .expect("network.allow array");
    // Manifest-level entry is unioned first, then layer entries (dedup, stable order).
    let allow: Vec<&str> = allow.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(allow, vec!["manifest.example.com", "layer.example.com"]);
}

#[test]
fn lifecycle_command_string_array_and_object_forms_compose() {
    // Spec 04: postCreateCommand/postStartCommand — merge by key (object form);
    // string/array forms preserved. All three forms must validate and compose.
    let env = support::TestEnv::new();
    support::write(
        env.path("config/manifests/app.toml"),
        "layers = [\"base\", \"leaf\"]\n",
    );
    support::write(
        env.path("config/devcontainer/base/devcontainer.json"),
        r#"{"postCreateCommand":"echo base","postStartCommand":{"svc":"start"}}"#,
    );
    support::write(
        env.path("config/devcontainer/leaf/devcontainer.json"),
        r#"{"postCreateCommand":["echo","leaf"],"postStartCommand":{"other":"go"}}"#,
    );
    env.cmd()
        .args(["manifest", "validate", "app"])
        .assert()
        .success();
    env.cmd()
        .args(["manifest", "compose", "app"])
        .assert()
        .success();
    let composed = std::fs::read_to_string(env.path("cache/composed/app/devcontainer.json"))
        .expect("composed");
    // Leaf array form replaces the accumulator whole (last-wins over the base string).
    assert!(composed.contains("\"echo\""), "array form lost: {composed}");
    assert!(
        !composed.contains("echo base"),
        "base string form should be replaced by leaf array: {composed}"
    );
    // Object forms merge by key across layers.
    assert!(
        composed.contains("\"svc\""),
        "base object key lost: {composed}"
    );
    assert!(
        composed.contains("\"other\""),
        "leaf object key lost: {composed}"
    );
}

#[test]
fn manifest_list_excludes_layers() {
    let env = support::TestEnv::new();
    support::write(
        env.path("config/manifests/app.toml"),
        "layers = [\"base\"]\n",
    );
    support::write(env.path("config/devcontainer/base/devcontainer.json"), "{}");
    let assert = env.cmd().args(["manifest", "list"]).assert().success();
    let stdout = support::stdout_utf8(assert);
    assert_eq!(stdout.trim(), "app");
}
