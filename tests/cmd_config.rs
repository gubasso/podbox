mod support;

#[test]
fn paths_honor_podbox_home() {
    let env = support::TestEnv::new();
    let assert = env.cmd().args(["config", "paths"]).assert().success();
    let stdout = support::stdout_utf8(assert);
    assert!(stdout.contains(&format!("config: {}", env.path("config").display())));
    assert!(stdout.contains(&format!("cache: {}", env.path("cache").display())));
}

#[test]
fn no_resolvable_config_for_workspace_exits_two_with_remedy() {
    // C8 / spec 03 §5: a workspace with no config source in the precedence chain must fail
    // with the actionable remedy and exit code 2, not silently fall back to defaults.
    let env = support::TestEnv::new();
    let workspace = env.path("ws");
    std::fs::create_dir_all(&workspace).unwrap();
    let assert = env
        .cmd()
        .args(["--workspace", workspace.to_str().unwrap(), "config", "show"])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("run `podbox init` or pass `--config`"),
        "missing remedy in stderr: {stderr}"
    );
}

#[test]
fn workspace_without_config_still_works_when_paths_only() {
    // Commands that do not resolve workspace config (no --workspace) keep defaults.
    let env = support::TestEnv::new();
    env.cmd().args(["config", "paths"]).assert().success();
}

#[test]
fn explicit_config_flag_outranks_workspace_devcontainer() {
    // C8 precedence: an explicit `--config` file must beat an auto-discovered workspace
    // `.devcontainer/devcontainer.json`, never the reverse.
    let env = support::TestEnv::new();
    let cli_config = env.path("explicit.toml");
    support::write(&cli_config, "[defaults]\nmanifest = \"from-cli\"\n");
    let workspace = env.path("ws");
    support::write(
        workspace.join(".devcontainer/devcontainer.json"),
        r#"{"name":"from-workspace"}"#,
    );
    let assert = env
        .cmd()
        .args([
            "--config",
            cli_config.to_str().unwrap(),
            "--workspace",
            workspace.to_str().unwrap(),
            "config",
            "get",
            "defaults.manifest",
        ])
        .assert()
        .success();
    assert_eq!(support::stdout_utf8(assert).trim(), "from-cli");
}

#[test]
fn set_preserves_nested_unknown_sibling() {
    let env = support::TestEnv::new();
    let config = env.path("config/config.toml");
    support::write(
        &config,
        r#"
[defaults]
custom_unknown = "keep"
manifest = "old"
"#,
    );
    env.cmd()
        .args([
            "--config",
            config.to_str().unwrap(),
            "config",
            "set",
            "defaults.manifest",
            "new",
        ])
        .assert()
        .success();
    let raw = std::fs::read_to_string(config).expect("config");
    assert!(raw.contains("custom_unknown = \"keep\""));
    assert!(raw.contains("manifest = \"new\""));
}

#[test]
fn non_prefixed_env_does_not_bleed_into_config() {
    let env = support::TestEnv::new();
    let assert = env
        .cmd()
        .env("XDG_RUNTIME_DIR", "/tmp/nope")
        .env("PODBOX_CACHE_HOME", env.path("other-cache"))
        .args(["config", "show", "--json"])
        .assert()
        .success();
    let stdout = support::stdout_utf8(assert);
    assert!(!stdout.contains("XDG_RUNTIME_DIR"));
    assert!(!stdout.contains("PODBOX_CACHE_HOME"));
}
