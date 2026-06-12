use super::super::super::super::sources::*;

#[test]
fn optional_feature_packaging_list_maps_strings_through_raw_enum_parser() {
    assert!(
        PACKAGING_LIST.contains(".map(packaging_strategy_from_plugin_toml)"),
        "packaging enum-list projection should map strings through the raw enum parser"
    );
}
