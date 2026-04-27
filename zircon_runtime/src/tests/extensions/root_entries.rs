#[test]
fn runtime_extensions_root_is_removed_after_plugin_cutover() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    assert!(
        !runtime_root.join("src/extensions").exists(),
        "optional runtime extension implementations should live under zircon_plugins, not zircon_runtime/src/extensions"
    );
}

#[test]
fn optional_extension_module_roots_live_in_zircon_plugins() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime crate should have a workspace parent");

    for plugin in ["navigation", "net", "particles", "sound", "texture"] {
        let plugin_root = repo_root.join("zircon_plugins").join(plugin);
        let runtime_lib = plugin_root.join("runtime/src/lib.rs");
        let plugin_manifest = plugin_root.join("plugin.toml");

        assert!(
            runtime_lib.exists(),
            "expected optional runtime plugin {plugin} to own runtime/src/lib.rs"
        );
        assert!(
            plugin_manifest.exists(),
            "expected optional runtime plugin {plugin} to declare plugin.toml"
        );
    }
}
