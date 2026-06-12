use super::super::super::super::sources::*;

#[test]
fn optional_feature_target_mode_list_maps_strings_through_raw_enum_parser() {
    assert!(
        TARGET_MODE_LIST.contains(".map(runtime_target_mode_from_plugin_toml)"),
        "target-mode enum-list projection should map strings through the raw enum parser"
    );
}
