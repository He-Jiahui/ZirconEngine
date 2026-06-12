use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_string_feature_facade_reexports_field_helpers() {
    assert!(
        STRING_FEATURE.contains("use display_name::feature_display_name_string_from_plugin_toml")
            && STRING_FEATURE.contains("use id::feature_id_string_from_plugin_toml")
            && STRING_FEATURE
                .contains("use owner_plugin::feature_owner_plugin_string_from_plugin_toml"),
        "feature string domain should expose child-owned field helpers"
    );
}
