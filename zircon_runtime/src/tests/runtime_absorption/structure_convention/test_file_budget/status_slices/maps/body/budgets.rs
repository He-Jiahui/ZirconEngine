use super::*;

#[test]
fn runtime_15_status_output_expected_slice_guard_maps_children_stay_budgeted() {
    for (path, max_lines) in [
        (MAPS_PARENT_PATH, 30),
        (GUARD_BODY_PARENT_PATH, 30),
        (TOP_LEVEL_MAPS_PATH, 80),
        (TOP_LEVEL_SUPPORT_LAYOUT_PATH, 80),
        (RUNTIME_15_TOPICS_PATH, 30),
        (RUNTIME_15_TOPIC_EXPECTED_MAPS_PATH, 120),
        (RUNTIME_15_TOPIC_REVIEW_MAPS_PATH, 80),
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} should stay below the focused guard budget {max_lines}; got {line_count} lines"
        );
    }

    for (path, max_lines) in GUARD_BODY_CHILD_PATHS
        .iter()
        .copied()
        .zip([60, 70, 80, 70, 90, 160])
    {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} should stay below the focused guard-body budget {max_lines}; got {line_count} lines"
        );
    }
}
