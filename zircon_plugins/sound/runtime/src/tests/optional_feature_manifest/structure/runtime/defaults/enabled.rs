use super::super::super::sources::*;

#[test]
fn optional_feature_runtime_defaults_enabled_child_owns_projection() {
    assert!(
        RUNTIME_DEFAULTS_ENABLED.contains("feature.enabled_by_default"),
        "runtime defaults enabled child should own enabled-by-default projection"
    );
}
