use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_literal_ownership_children_stay_budgeted() {
    for (path, source, limit) in [
        (
            LITERAL_OWNERSHIP_PARENT_PATH,
            read_literal_owner_source("literal_ownership.rs"),
            25usize,
        ),
        (
            LITERAL_OWNERSHIP_CHILDREN[0],
            read_literal_owner_source("literal/budgets.rs"),
            70,
        ),
        (
            LITERAL_OWNERSHIP_CHILDREN[1],
            read_literal_owner_source("literal/folder_backed.rs"),
            75,
        ),
        (
            LITERAL_OWNERSHIP_CHILDREN[2],
            read_literal_owner_source("literal/naming_literals.rs"),
            35,
        ),
        (
            LITERAL_OWNERSHIP_CHILDREN[3],
            read_literal_owner_source("literal/paths.rs"),
            75,
        ),
        (
            LITERAL_OWNERSHIP_CHILDREN[4],
            read_literal_owner_source("literal/review_literals.rs"),
            45,
        ),
        (
            LITERAL_OWNERSHIP_CHILDREN[5],
            read_literal_owner_source("literal/sources.rs"),
            30,
        ),
        (
            LITERAL_OWNERSHIP_CHILDREN[6],
            read_literal_owner_source("literal/status_mirrors.rs"),
            95,
        ),
        (
            LITERAL_OWNERSHIP_CHILDREN[7],
            read_literal_owner_source("literal/status_support_literals.rs"),
            55,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= limit,
            "{path} should stay below the structure-support literal ownership budget {limit}; got {line_count}"
        );
    }
}
