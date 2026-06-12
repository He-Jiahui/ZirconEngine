use super::super::super::super::sources::*;

#[test]
fn optional_feature_module_kind_facade_does_not_reexport_raw_parser() {
    assert!(
        !MODULE_KIND_ROOT.contains("use raw::module_kind_from_plugin_toml"),
        "module_kind parent must not re-export the raw TOML parser"
    );
}
