use super::*;

#[test]
fn runtime_15_p0_native_fixture_leaf_owner_guard_budgets_are_focused() {
    for (path, source) in [
        (
            STRUCTURE_GUARD_OWNER,
            read_runtime_src(STRUCTURE_GUARD_OWNER),
        ),
        (PARENT, read_runtime_src(PARENT)),
        (SDK_MACRO_LEAF, read_runtime_src(SDK_MACRO_LEAF)),
        (IMPORTER_LEAF, read_runtime_src(IMPORTER_LEAF)),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
    for (path, source) in folder_backed_child_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < 200,
            "{path} should stay below the focused P0 native fixture structure-guard child budget; got {line_count} lines"
        );
    }
}
