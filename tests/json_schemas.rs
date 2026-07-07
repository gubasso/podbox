mod support;

use support::{TestEnv, stdout_utf8, write};

fn fixture(env: &TestEnv) {
    write(
        env.path("config/config.toml"),
        r#"[defaults]
manifest = "default"
"#,
    );
    write(
        env.path("config/manifests/default.toml"),
        "layers = [\"base\"]\n",
    );
    write(
        env.path("config/devcontainer/base/devcontainer.json"),
        r#"{"image":"alpine:latest"}"#,
    );
}

#[test]
fn json_commands_include_schema_version() {
    let env = TestEnv::new();
    fixture(&env);
    for args in [
        vec!["--json", "version"],
        vec!["--json", "status"],
        vec!["--json", "doctor", "--scope", "config"],
        vec!["--json", "manifest", "show", "default"],
        vec!["--json", "network", "show"],
    ] {
        let stdout = stdout_utf8(env.cmd().args(args).assert().success());
        let value: serde_json::Value = serde_json::from_str(&stdout).expect("json object");
        assert_eq!(value["schema_version"], 1);
    }
}
