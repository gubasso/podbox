use std::path::{Path, PathBuf};

#[test]
fn main_rs_stays_tiny_entrypoint() {
    let text = std::fs::read_to_string("src/main.rs").expect("main");
    assert!(text.lines().count() <= 120);
    assert!(text.contains("podbox::run()"));
}

#[test]
fn command_dispatch_is_exhaustive_and_includes_final_surface() {
    let text = std::fs::read_to_string("src/commands/mod.rs").expect("commands mod");
    assert!(text.contains("match command"));
    assert!(!text.contains("_ =>"));
    for variant in [
        "Commands::Shell",
        "Commands::Init",
        "Commands::Doctor",
        "Commands::Status",
        "Commands::Completion",
        "Commands::Version",
        "Commands::Workspace",
        "Commands::Network",
        "Commands::Config",
        "Commands::Manifest",
        "Commands::Image",
    ] {
        assert!(text.contains(variant), "missing dispatch arm for {variant}");
    }
}

#[test]
fn lib_surface_is_only_run() {
    let text = std::fs::read_to_string("src/lib.rs").expect("lib");
    let public_functions = text.matches("pub fn ").count();
    assert_eq!(public_functions, 1);
    assert!(text.contains("pub fn run()"));
}

#[test]
fn domain_has_no_io_or_cross_layer_imports() {
    for path in rust_files(Path::new("src/domain")) {
        let text = std::fs::read_to_string(&path).expect("read source");
        for forbidden in [
            "std::fs",
            "tokio::fs",
            "std::process",
            "Command::new",
            "crate::adapters",
            "crate::services",
            "crate::config",
            "crate::ui",
        ] {
            assert!(
                !text.contains(forbidden),
                "{} contains {forbidden}",
                path.display()
            );
        }
    }
}

#[test]
fn non_test_source_has_no_unjustified_debug_or_panic_macros() {
    for path in rust_files(Path::new("src")) {
        if path.components().any(|c| c.as_os_str() == "tests") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read source");
        for forbidden in ["todo!(", "dbg!(", "panic!("] {
            assert!(
                !text.contains(forbidden),
                "{} contains {forbidden}",
                path.display()
            );
        }
    }
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).expect("read dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.is_dir() {
            out.extend(rust_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
    out
}
