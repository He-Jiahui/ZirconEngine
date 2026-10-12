#[test]
fn nested_cancel_does_not_remove_from_the_front_of_a_vec() {
    let source = include_str!("../transaction.rs");
    let front_remove = ["frames", ".remove(0)"].concat();
    assert!(!source.contains(&front_remove));
}

#[test]
fn transaction_root_remains_a_structural_facade() {
    let source = include_str!("../transaction.rs");
    assert!(
        source.lines().count() <= 96,
        "transaction root exceeded facade budget"
    );
    for behavioral_item in [
        ["fn ", "begin_transaction"].concat(),
        ["fn ", "replay"].concat(),
        ["fn ", "commit_after_apply"].concat(),
        ["fn ", "cancel_frame"].concat(),
    ] {
        assert!(
            !source.contains(&behavioral_item),
            "transaction root retained behavioral item: {behavioral_item}"
        );
    }
}
