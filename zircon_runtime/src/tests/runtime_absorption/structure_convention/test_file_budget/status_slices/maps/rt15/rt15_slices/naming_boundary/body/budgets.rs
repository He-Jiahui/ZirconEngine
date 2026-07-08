use super::*;

#[test]
fn runtime_15_status_output_naming_boundary_expected_slice_sources_stay_budgeted() {
    for (path, max_lines) in [
        (STATUS_PARENT_PATH, 40usize),
        (DATE_PARENT_PATH, 40),
        (STATUS_CHILD_PATHS[0], 80),
        (STATUS_CHILD_PATHS[1], 130),
        (STATUS_CHILD_PATHS[2], 130),
        (STATUS_CHILD_PATHS[3], 45),
        (DATE_CHILD_PATHS[0], 80),
        (DATE_CHILD_PATHS[1], 130),
        (DATE_CHILD_PATHS[2], 130),
        (DATE_CHILD_PATHS[3], 45),
    ] {
        assert_line_budget(path, max_lines);
    }

    for path in STATUS_RENDER_GRAPHICS_CHILDREN
        .iter()
        .chain(DATE_RENDER_GRAPHICS_CHILDREN.iter())
    {
        assert_line_budget(path, 80);
    }
}

fn assert_line_budget(path: &str, max_lines: usize) {
    let line_count = read_runtime_src(path).lines().count();
    assert!(
        line_count <= max_lines,
        "{path} has {line_count} lines, expected <= {max_lines}"
    );
}
