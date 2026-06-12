use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_string_module_facade_reexports_field_helpers() {
    assert!(
        STRING_MODULE.contains("use crate_name::module_crate_name_string_from_plugin_toml")
            && STRING_MODULE.contains("use name::module_name_string_from_plugin_toml"),
        "module string domain should expose child-owned field helpers"
    );
}
