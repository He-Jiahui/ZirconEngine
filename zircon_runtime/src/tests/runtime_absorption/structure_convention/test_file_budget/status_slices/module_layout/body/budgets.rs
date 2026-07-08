use super::*;

#[test]
fn runtime_15_expected_slice_module_layout_guard_body_children_stay_budgeted() {
    for (path, limit) in [
        (MODULE_LAYOUT_PARENT, 20usize),
        (MODULE_LAYOUT_GUARD_BODY_PARENT, 30),
        (MODULE_LAYOUT_GUARD_BODY_CHILDREN[0], 85),
        (MODULE_LAYOUT_GUARD_BODY_CHILDREN[1], 105),
        (MODULE_LAYOUT_GUARD_BODY_CHILDREN[2], 105),
        (MODULE_LAYOUT_GUARD_BODY_CHILDREN[3], 55),
        (MODULE_LAYOUT_GUARD_BODY_CHILDREN[4], 120),
        (MODULE_LAYOUT_GUARD_BODY_CHILDREN[5], 160),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the Runtime 15 module-layout guard-body budget {limit}; got {line_count} lines"
        );
    }

    for path in [
        EXPECTED_SLICES_PARENT,
        "structure_convention/test_file_budget/status_slices/maps.rs",
        "structure_convention/test_file_budget/status_slices/maps/top_level_maps.rs",
        "structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions.rs",
        "structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/runtime_15_maps.rs",
        "structure_convention/test_file_budget/status_slices/legacy_maps.rs",
        "structure_convention/test_file_budget/status_slices/legacy_maps/guard_body.rs",
        "structure_convention/test_file_budget/status_slices/legacy_group_maps.rs",
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
