mod support;

use predicates::prelude::*;
use support::TestEnv;

#[test]
fn completion_generates_supported_shells() {
    let env = TestEnv::new();
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        env.cmd()
            .args(["completion", shell])
            .assert()
            .success()
            .stdout(predicate::str::contains("podbox"));
    }
}

#[test]
fn hidden_manpage_emitter_generates_man_source() {
    let env = TestEnv::new();
    env.cmd()
        .arg("manpage")
        .assert()
        .success()
        .stdout(predicate::str::contains(".TH podbox"));
}
