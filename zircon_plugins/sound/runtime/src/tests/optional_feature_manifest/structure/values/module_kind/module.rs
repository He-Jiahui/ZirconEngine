use super::super::super::sources::*;

#[test]
fn optional_feature_module_kind_projection_calls_raw_parser_owner() {
    assert!(
        MODULE_KIND_MODULE.contains("super::raw::module_kind_from_plugin_toml(value)"),
        "module-kind semantic projection should call the raw parser from its child owner"
    );
}
