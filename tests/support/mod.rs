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

    /// Like [`cmd`], but with a `PATH` that contains no container runtime
    /// executable. This makes the "live runtime missing" path deterministic on
    /// hosts where `podman` happens to be installed: the runtime spawn fails
    /// with `NotFound` regardless of the ambient environment.
    ///
    /// [`cmd`]: Self::cmd
    pub(crate) fn cmd_without_runtime(&self) -> Command {
        let mut cmd = Command::cargo_bin("podbox").expect("binary");
        cmd.env_clear()
            .env("PATH", self.temp.path())
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

/// Strip trailing whitespace from each line, keeping snapshots stable against
/// the repo's `trailing-whitespace` hook. clap pads help lines that have no
/// description with trailing spaces; the hook trims them from the stored
/// `.snap`, which would otherwise make the assertion fail forever.
pub(crate) fn trim_line_endings(s: &str) -> String {
    let mut out: String = s.lines().map(str::trim_end).collect::<Vec<_>>().join("\n");
    if s.ends_with('\n') {
        out.push('\n');
    }
    out
}

pub(crate) fn write(path: impl AsRef<Path>, contents: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent");
    }
    std::fs::write(path, contents).expect("write file");
}
