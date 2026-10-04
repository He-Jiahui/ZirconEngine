use super::exact_builtin_editor_crate_name;

#[test]
fn exact_builtin_editor_crate_names_preserve_package_identity() {
    assert_eq!(
        exact_builtin_editor_crate_name("net"),
        "zircon_plugin_net_editor"
    );
    assert_eq!(
        exact_builtin_editor_crate_name("rendering_deferred"),
        "zircon_plugin_rendering_deferred_editor"
    );
}
