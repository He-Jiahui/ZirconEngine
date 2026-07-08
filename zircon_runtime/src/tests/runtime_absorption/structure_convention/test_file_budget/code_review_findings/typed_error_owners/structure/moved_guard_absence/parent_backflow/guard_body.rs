use super::*;

pub(in super::super) fn assert_typed_error_parent_backflow_guards_are_absent() {
    let parent_sources = typed_error_parent_backflow_parent_sources();
    for child_owned_test in PARENT_BACKFLOW_GUARDS {
        for (label, path, source) in &parent_sources {
            assert!(
                !source.contains(child_owned_test),
                "child-owned review guard `{child_owned_test}` should not return to {label} at {path}"
            );
        }
    }
}
