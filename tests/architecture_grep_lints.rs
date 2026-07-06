use std::path::Path;

#[test]
fn no_print_macros_outside_ui_and_main() {
    for path in rust_files(Path::new("src")) {
        let text = std::fs::read_to_string(&path).expect("read source");
        let display = path.display().to_string();
        if display.starts_with("src/ui/") || display == "src/main.rs" {
            continue;
        }
        assert!(!text.contains("println!"), "{display} contains println!");
        assert!(!text.contains("eprintln!"), "{display} contains eprintln!");
        assert!(!text.contains("print!"), "{display} contains print!");
        assert!(!text.contains("eprint!"), "{display} contains eprint!");
    }
}

#[test]
fn dispatch_has_no_wildcard_arm() {
    let text = std::fs::read_to_string("src/commands/mod.rs").expect("dispatch source");
    assert!(!text.contains("_ =>"));
    assert!(text.contains("match command"));
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
