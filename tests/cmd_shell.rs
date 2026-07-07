mod support;

use support::{TestEnv, stdout_utf8};

#[test]
fn shell_is_first_top_level_help_command() {
    let env = TestEnv::new();
    let output = stdout_utf8(env.cmd().arg("--help").assert().success());
    let commands = output.split("Commands:").nth(1).expect("commands section");
    let first = commands
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("first command")
        .trim();
    assert!(first.starts_with("shell"), "first command was {first:?}");
}
