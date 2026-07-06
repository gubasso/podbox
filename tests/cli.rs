//! Integration lane placeholder: exercises the library seam the binary is
//! built on. Replace with real argv-contract tests as the CLI grows.

#[test]
fn greeting_matches_binary_banner() {
    assert_eq!(podbox::greeting(), "Hello, world!");
}
