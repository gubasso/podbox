mod support;

use support::{TestEnv, stdout_utf8, trim_line_endings};

#[test]
fn root_help_locks_public_command_surface() {
    let env = TestEnv::new();
    let output = trim_line_endings(&stdout_utf8(env.cmd().arg("--help").assert().success()));
    assert!(output.contains("shell"));
    assert!(output.contains("init"));
    assert!(output.contains("status"));
    assert!(output.contains("completion"));
    assert!(!output.contains("deploy"));
    assert!(!output.contains("manpage"));

    let commands = output.split("Commands:").nth(1).expect("commands section");
    let first = commands
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("first command")
        .trim();
    assert!(first.starts_with("shell"), "first command was {first:?}");
    insta::assert_snapshot!(output);
}

#[test]
fn doctor_help_has_reconciled_signature() {
    let env = TestEnv::new();
    let output = trim_line_endings(&stdout_utf8(
        env.cmd().args(["doctor", "--help"]).assert().success(),
    ));
    assert!(output.contains("--scope"));
    assert!(output.contains("--json"));
    assert!(output.contains("--quiet"));
    assert!(!output.contains("--strict"));
    insta::assert_snapshot!(output);
}

#[test]
fn subcommand_help_surface_has_no_deploy_or_public_manpage() {
    let env = TestEnv::new();
    for args in [
        vec!["shell", "--help"],
        vec!["init", "--help"],
        vec!["status", "--help"],
        vec!["completion", "--help"],
        vec!["workspace", "--help"],
        vec!["image", "--help"],
        vec!["manifest", "--help"],
        vec!["config", "--help"],
        vec!["network", "--help"],
    ] {
        let output = stdout_utf8(env.cmd().args(args).assert().success());
        assert!(!output.contains("deploy"));
        assert!(!output.contains("manpage"));
    }
}
