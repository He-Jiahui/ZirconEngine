use super::feature_definition_key;

#[test]
fn exact_feature_definition_key_preserves_both_identities() {
    assert_eq!(
        feature_definition_key("sound.timeline", "sound_core"),
        "sound.timeline@sound_core"
    );
    assert_eq!(feature_definition_key("", ""), "@");
    assert_eq!(feature_definition_key("feature", ""), "feature@");
    assert_eq!(feature_definition_key("", "provider"), "@provider");
}
