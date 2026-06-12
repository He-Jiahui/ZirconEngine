use super::super::super::super::sources::*;

#[test]
fn optional_feature_runtime_capability_collection_entry_composes_projection_and_ordering() {
    assert!(
        RUNTIME_CAPABILITIES_ENTRY.contains("pub(in super::super) fn capability_signatures")
            && RUNTIME_CAPABILITIES_ENTRY
                .contains("super::projection::project_capability_signatures(feature)")
            && RUNTIME_CAPABILITIES_ENTRY
                .contains("super::ordering::sort_capability_signatures(&mut capabilities)"),
        "runtime capabilities entry child should be facade-visible and compose projection and ordering"
    );
}
