use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_parent_route_sources_stay_budgeted() {
    for (path, source) in [
        (STATUS_PARENT, read_runtime_src(STATUS_PARENT)),
        (DATE_PARENT, read_runtime_src(DATE_PARENT)),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 240,
            "{path} should stay below the Runtime 15 structure parent route budget; got {line_count} lines"
        );
    }
    for (path, source) in STATUS_STRUCTURE_PARENT_ROUTE_CHILDREN
        .iter()
        .chain(DATE_STRUCTURE_PARENT_ROUTE_CHILDREN.iter())
        .map(|path| (*path, read_runtime_src(path)))
    {
        let line_count = source.lines().count();
        assert!(
            line_count < 160,
            "{path} should stay below the Runtime 15 structure parent child budget; got {line_count} lines"
        );
    }
}
