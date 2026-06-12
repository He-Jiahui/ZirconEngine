use super::super::super::super::sources::*;

#[test]
fn optional_feature_runtime_dependency_collection_entry_composes_projection_and_ordering() {
    assert!(
        RUNTIME_DEPENDENCIES_ENTRY.contains("pub(in super::super) fn dependency_signatures")
            && RUNTIME_DEPENDENCIES_ENTRY
                .contains("super::projection::project_dependency_signatures(feature)")
            && RUNTIME_DEPENDENCIES_ENTRY
                .contains("super::ordering::sort_dependency_signatures(&mut dependencies)"),
        "runtime dependencies entry child should be facade-visible and compose projection and ordering"
    );
}
