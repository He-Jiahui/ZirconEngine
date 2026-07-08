use super::*;

#[test]
fn runtime_15_expected_slice_child_owner_sources_stay_budgeted() {
    for (path, runtime_src_path) in EXPECTED_SLICE_BUDGET_SOURCE_PATHS {
        let source = read_runtime_src(runtime_src_path);
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the Runtime 15 focused expected-slice budget; got {line_count} lines"
        );
    }
}
