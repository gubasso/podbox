// Shared integration-test support helpers.
#![allow(dead_code)]

use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

pub(crate) struct TestEnv {
    pub(crate) temp: TempDir,
}

impl TestEnv {
    pub(crate) fn new() -> Self {
        Self {
            temp: tempfile::tempdir().expect("tempdir"),
        }
    }

    pub(crate) fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("podbox").expect("binary");
        cmd.env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("PODBOX_HOME", self.temp.path())
            .env("RUST_LOG", "off");
        cmd
    }

    pub(crate) fn path(&self, child: impl AsRef<Path>) -> std::path::PathBuf {
        self.temp.path().join(child)
    }
}

pub(crate) fn stdout_utf8(assert: assert_cmd::assert::Assert) -> String {
    String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8")
}

pub(crate) fn write(path: impl AsRef<Path>, contents: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent");
    }
    std::fs::write(path, contents).expect("write file");
}
