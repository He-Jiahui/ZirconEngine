use super::super::super::super::*;
use super::*;

pub(super) fn assert_typed_error_structure_guard_delegation_is_current() {
    let structure_child = read_runtime_src(STRUCTURE_GUARD_CHILD_OWNER);
    let parent = read_runtime_src(STRUCTURE_GUARD_TYPED_ERROR_CHILD_OWNER);
    let child_tree = typed_error_structure_guard_child_source_blob();

    assert_contains_all(
        "code-review structure guard delegates typed-error child checks",
        &structure_child,
        &[
            "#[path = \"structure_guard_children/typed_error.rs\"]",
            "mod typed_error;",
            "typed_error::assert_typed_error_structure_children_are_mounted",
        ],
    );
    for backflow_guard in [
        "typed-error top-level folder-backed children own actual guard bodies",
        "typed-error structure assertions child keeps focused guard mounts",
        "typed-error moved-guard child keeps review guard ownership checks",
        "typed-error source inventory child keeps fine-grained typed-error source paths",
    ] {
        assert!(
            !parent.contains(backflow_guard),
            "typed-error structure guard `{backflow_guard}` should stay in focused children under {STRUCTURE_GUARD_TYPED_ERROR_CHILD_OWNER}"
        );
    }
    assert_contains_all(
        "typed-error structure guard parent mounts focused children",
        &parent,
        &[
            "#[path = \"typed_error/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"typed_error/top_level.rs\"]",
            "mod top_level;",
            "#[path = \"typed_error/structure_assertions.rs\"]",
            "mod structure_assertions;",
            "#[path = \"typed_error/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"typed_error/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "pub(super) fn assert_typed_error_structure_children_are_mounted",
            "typed_error_structure_guard_child_sources",
            "typed_error_structure_guard_child_source_blob",
        ],
    );
    assert_contains_all(
        "typed-error structure guard children own delegated checks",
        &child_tree,
        &[
            "runtime_15_code_review_findings_structure_guard_typed_error_is_child_owner",
            "runtime_15_code_review_findings_structure_guard_typed_error_top_level_checks_are_child_owned",
            "runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_are_child_owned",
            "runtime_15_code_review_findings_structure_guard_typed_error_children_line_budgets_are_current",
        ],
    );
    for (_, child_path, anchor) in STRUCTURE_GUARD_TYPED_ERROR_CHILDREN {
        assert!(
            parent.contains(child_path),
            "typed-error structure guard parent should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error structure guard child {child_path} should own anchor {anchor}"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_structure_guard_typed_error_delegation_is_child_owned() {
    assert_typed_error_structure_guard_delegation_is_current();
}
