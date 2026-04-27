#[test]
fn optional_extension_module_registration_is_externalized_to_plugin_packages() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = runtime_root
        .parent()
        .expect("runtime crate should have a workspace parent");
    let builtin_source =
        std::fs::read_to_string(runtime_root.join("src/builtin/mod.rs")).unwrap_or_default();

    assert!(
        !runtime_root.join("src/extensions").exists(),
        "zircon_runtime should not keep optional extension implementations after plugin cutover"
    );

    for plugin in [
        "physics",
        "sound",
        "texture",
        "net",
        "navigation",
        "particles",
        "animation",
    ] {
        let plugin_root = repo_root.join("zircon_plugins").join(plugin);
        assert!(
            plugin_root.join("runtime/Cargo.toml").exists(),
            "runtime plugin {plugin} should own its runtime crate manifest"
        );
        assert!(
            plugin_root.join("plugin.toml").exists(),
            "runtime plugin {plugin} should own its plugin package manifest"
        );
    }

    assert!(
        !builtin_source.contains("Arc::new(zircon_"),
        "builtin runtime module list should stop constructing optional extension modules from external crates"
    );
}
