use super::super::super::super::sources::*;

#[test]
fn optional_feature_module_kind_facade_does_not_own_projection_forwarding_body() {
    assert!(
        !MODULE_KIND_ROOT.contains("fn module_kind_value_from_plugin_toml"),
        "module_kind parent must not own semantic projection forwarding bodies"
    );
}
