mod support;

use predicates::prelude::*;
use support::{TestEnv, stdout_utf8};

#[test]
fn quiet_verbose_is_usage_error() {
    let env = TestEnv::new();
    env.cmd()
        .args(["--quiet", "--verbose", "version"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("usage"));
}

#[test]
fn json_success_outputs_schema_and_no_ansi() {
    let env = TestEnv::new();
    let stdout = stdout_utf8(env.cmd().args(["--json", "version"]).assert().success());
    assert!(stdout.contains(r#""schema_version": 1"#));
    assert!(!stdout.contains("\u{1b}["));
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
}

#[test]
fn json_errors_are_structured_on_stdout() {
    let env = TestEnv::new();
    let assert = env
        .cmd()
        .args(["--json", "manifest", "show", "missing"])
        .assert();
    let output = assert.code(1).get_output().clone();
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""schema_version": 1"#));
    assert!(stdout.contains(r#""code": "io""#));
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
}
