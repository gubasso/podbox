use camino::Utf8PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Starter {
    Minimal,
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SeedFile {
    pub(crate) relative_path: &'static str,
    pub(crate) content: String,
    pub(crate) leaf_protected: bool,
}

pub(crate) fn files(starter: Starter, manifest_name: &str) -> Vec<SeedFile> {
    let mut files = vec![SeedFile {
        relative_path: "config.toml",
        content: starter_config(manifest_name),
        leaf_protected: false,
    }];
    if starter == Starter::Minimal {
        files.extend([
            SeedFile {
                relative_path: "manifests/minimal.toml",
                content: MINIMAL_MANIFEST.to_string(),
                leaf_protected: false,
            },
            SeedFile {
                relative_path: "devcontainer/base/devcontainer.json",
                content: BASE_LAYER.to_string(),
                leaf_protected: false,
            },
            SeedFile {
                relative_path: "devcontainer/minimal/devcontainer.json",
                content: LEAF_LAYER.to_string(),
                leaf_protected: true,
            },
            SeedFile {
                relative_path: "images/minimal/devcontainer.json",
                content: IMAGE_SOURCE.to_string(),
                leaf_protected: false,
            },
        ]);
    }
    files
}

pub(crate) fn dirs() -> &'static [&'static str] {
    &["manifests", "devcontainer", "images"]
}

pub(crate) fn target(root: &camino::Utf8Path, relative: &str) -> Utf8PathBuf {
    root.join(relative)
}

fn starter_config(manifest_name: &str) -> String {
    CONFIG_TEMPLATE.replace("__MANIFEST__", manifest_name)
}

const CONFIG_TEMPLATE: &str = r#"[meta]
schema_version = 1

[defaults]
manifest = "__MANIFEST__"
reconcile = "auto"
color = "auto"
output = "human"

[runtime]
profile = "microvm"
memory_mib = 2048
cpus = 2

[images]
pull_policy = "missing"

[network]
egress = "deny"
allow = []
"#;

const MINIMAL_MANIFEST: &str = r#"layers = ["base", "minimal"]

[network]
allow = []
"#;

const BASE_LAYER: &str = r#"{
    "name": "podbox-minimal",
    "image": "alpine:latest",
    "remoteEnv": {
        "PODBOX": "1"
    }
}
"#;

const LEAF_LAYER: &str = r#"{
    "name": "podbox-minimal-workspace",
    "postCreateCommand": "true"
}
"#;

const IMAGE_SOURCE: &str = r#"{
    "image": "alpine:latest"
}
"#;
