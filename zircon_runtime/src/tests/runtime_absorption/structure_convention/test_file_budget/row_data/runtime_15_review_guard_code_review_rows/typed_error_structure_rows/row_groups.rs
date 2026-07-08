use super::*;

pub(super) fn assert_typed_error_structure_row_groups_are_child_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_ROWS_PATH);
    for (module, export) in TYPED_ERROR_STRUCTURE_ROW_GROUPS {
        let mount = format!("#[path = \"typed_error_structure_rows/{module}.rs\"]");
        let export = format!("typed_error_structure_rows::{export}");
        assert_contains_all(
            "typed-error structure row parent mounts grouped children",
            &parent,
            &[mount.as_str()],
        );
        assert_contains_all(
            "code-review parent exports typed-error structure row groups",
            &read_runtime_src(CODE_REVIEW_ROWS_PATH),
            &[export.as_str()],
        );
    }
    assert!(
        !parent.contains("Runtime 15 M3 typed-error status-doc paths child split"),
        "typed_error_structure_rows.rs should route row groups instead of owning status-doc row tuples",
    );
}
