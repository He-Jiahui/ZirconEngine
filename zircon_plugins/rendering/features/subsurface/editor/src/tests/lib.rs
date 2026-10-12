use super::*;

#[test]
fn editor_feature_exports_manifest_capability() {
    assert_eq!(feature_manifest().id, FEATURE_ID);
    assert_eq!(editor_capabilities(), vec![CAPABILITY.to_string()]);
}
