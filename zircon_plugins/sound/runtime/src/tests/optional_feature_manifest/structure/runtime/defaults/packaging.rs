use super::super::super::sources::*;

#[test]
fn optional_feature_runtime_defaults_packaging_child_owns_projection() {
    assert!(
        RUNTIME_DEFAULTS_PACKAGING.contains("feature.default_packaging.clone()"),
        "runtime defaults packaging child should own default-packaging projection"
    );
}
