use super::*;

#[test]
fn runtime_15_expected_slice_child_literal_ownership_sources_stay_budgeted() {
    for (label, source, budget) in [
        (
            "literal_ownership.rs",
            read_child_owner("literal_ownership.rs"),
            25usize,
        ),
        (
            "literal/budgets.rs",
            read_child_owner("literal/budgets.rs"),
            65usize,
        ),
        (
            "literal/date_literals.rs",
            read_child_owner("literal/date_literals.rs"),
            35usize,
        ),
        (
            "literal/folder_backed.rs",
            read_child_owner("literal/folder_backed.rs"),
            70usize,
        ),
        (
            "literal/paths.rs",
            read_child_owner("literal/paths.rs"),
            75usize,
        ),
        (
            "literal/source_groups.rs",
            read_child_owner("literal/source_groups.rs"),
            35usize,
        ),
        (
            "literal/status_literals.rs",
            read_child_owner("literal/status_literals.rs"),
            35usize,
        ),
        (
            "literal/status_mirrors.rs",
            read_child_owner("literal/status_mirrors.rs"),
            90usize,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= budget,
            "{label} should stay below {budget} lines; got {line_count}"
        );
    }
}
