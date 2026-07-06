use std::path::Path;

#[test]
fn domain_has_no_io_or_adapter_imports() {
    for path in rust_files(Path::new("src/domain")) {
        let text = std::fs::read_to_string(&path).expect("read source");
        for forbidden in [
            "std::fs",
            "tokio::fs",
            "std::process",
            "Command::new",
            "anstream",
            "tracing",
            "crate::adapters",
            "crate::services",
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

fn rust_files(dir: &Path) -> Vec<std::path::PathBuf> {
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
