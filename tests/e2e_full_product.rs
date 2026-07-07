mod support;

use predicates::prelude::*;
use support::TestEnv;

#[test]
fn hermetic_full_product_surface_flow() {
    let env = TestEnv::new();

    env.cmd().arg("init").assert().success();
    env.cmd()
        .args(["doctor", "--scope", "config"])
        .assert()
        .success();
    env.cmd()
        .args(["manifest", "compose", "minimal"])
        .assert()
        .success()
        .stdout(predicate::str::contains("digest:"));
    env.cmd()
        .args(["network", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("posture: deny"));
    env.cmd()
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("state: absent"));
    env.cmd()
        .args(["completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("podbox"));
    env.cmd()
        .args(["workspace", "down", "--dry-run"])
        .assert()
        .success();
}
