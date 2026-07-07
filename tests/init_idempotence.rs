mod support;

use support::{TestEnv, write};

#[test]
fn init_is_idempotent_and_force_preserves_user_leaf() {
    let env = TestEnv::new();

    env.cmd().arg("init").assert().success();
    let first_config = std::fs::read_to_string(env.path("config/config.toml")).unwrap();
    write(
        env.path("config/devcontainer/minimal/devcontainer.json"),
        r#"{"name":"user-leaf"}"#,
    );
    write(
        env.path("config/devcontainer/base/devcontainer.json"),
        r#"{"name":"modified-shared"}"#,
    );

    env.cmd().args(["init", "--force"]).assert().success();

    let second_config = std::fs::read_to_string(env.path("config/config.toml")).unwrap();
    let leaf =
        std::fs::read_to_string(env.path("config/devcontainer/minimal/devcontainer.json")).unwrap();
    let shared =
        std::fs::read_to_string(env.path("config/devcontainer/base/devcontainer.json")).unwrap();
    assert_eq!(first_config, second_config);
    assert!(
        leaf.contains("user-leaf"),
        "leaf layer was overwritten: {leaf}"
    );
    assert!(
        !shared.contains("modified-shared"),
        "shared layer was not reconciled: {shared}"
    );
}
