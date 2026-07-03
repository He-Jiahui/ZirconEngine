use super::*;

#[test]
fn runtime_15_status_output_row_data_module_layout_children_stay_focused() {
    for (label, path, max_lines) in MODULE_LAYOUT_GUARD_OWNER_PATHS {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < *max_lines,
            "{label} at {path} should stay below {max_lines} lines; got {line_count}"
        );
    }
}
