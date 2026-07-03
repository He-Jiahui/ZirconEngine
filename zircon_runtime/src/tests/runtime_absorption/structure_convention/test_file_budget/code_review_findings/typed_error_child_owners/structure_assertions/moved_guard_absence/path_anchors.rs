use super::super::super::super::super::*;
use super::*;

const STRUCTURE_ASSERTION_CHILD_PATH_ANCHORS: &[&str] = &[
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/string_helpers.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/descriptor_abi.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/entry_abi.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input/surface_effects.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input/surrounding_text.rs",
];

pub(super) fn assert_typed_error_child_path_anchors_are_current() {
    let child = read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_PATH_ANCHORS_CHILD);
    for child_path in STRUCTURE_ASSERTION_CHILD_PATH_ANCHORS {
        assert!(
            child.contains(child_path),
            "typed-error moved-guard path anchor child should keep full child-owner path anchor `{child_path}`"
        );
    }
}

#[test]
fn runtime_15_typed_error_moved_guard_absence_path_anchors_are_child_owned() {
    assert_typed_error_child_path_anchors_are_current();
}
