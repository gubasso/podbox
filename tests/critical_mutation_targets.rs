#[test]
#[ignore = "documentation marker for ADR-0013 cargo-mutants floor"]
fn critical_mutation_targets_and_threshold() {
    let targets = [
        "src/services/compose.rs",
        "src/domain/digest.rs",
        "src/exit.rs",
        "src/config/loader.rs",
    ];
    assert_eq!(targets.len(), 4);
    let required_caught_percent = 60;
    assert!(required_caught_percent >= 60);
}
