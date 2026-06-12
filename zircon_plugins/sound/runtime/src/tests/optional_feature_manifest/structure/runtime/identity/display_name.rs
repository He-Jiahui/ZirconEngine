use super::super::super::sources::*;

#[test]
fn optional_feature_runtime_identity_display_name_child_owns_projection() {
    assert!(
        RUNTIME_IDENTITY_DISPLAY_NAME.contains("feature.display_name.clone()"),
        "runtime identity display-name child should own display-name projection"
    );
}
