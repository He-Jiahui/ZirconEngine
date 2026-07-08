use super::*;

#[test]
fn runtime_15_naming_boundary_render_graphics_map_rows_guard_children_stay_budgeted() {
    for (path, max_lines) in [
        (MAP_ROWS_GUARD_PARENT, 35usize),
        (MAP_ROWS_GUARD_CHILDREN[0], 45),
        (MAP_ROWS_GUARD_CHILDREN[1], 55),
        (MAP_ROWS_GUARD_CHILDREN[2], 95),
        (MAP_ROWS_GUARD_CHILDREN[3], 85),
        (MAP_ROWS_GUARD_CHILDREN[4], 130),
        (MAP_ROWS_GUARD_CHILDREN[5], 105),
    ] {
        let line_count = read_runtime_absorption_child(path).lines().count();
        assert!(
            line_count <= max_lines,
            "{path} should stay below the render-graphics map row guard budget {max_lines}; got {line_count} lines"
        );
    }
}
