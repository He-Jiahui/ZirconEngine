#[test]
fn runtime_resource_foundation_keeps_editor_inspector_surface_internal() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let resource_mod_source =
        std::fs::read_to_string(runtime_root.join("src/core/resource/mod.rs")).unwrap_or_default();
    let resource_runtime_source =
        std::fs::read_to_string(runtime_root.join("src/core/resource/runtime.rs"))
            .unwrap_or_default();

    for required in ["Resource", "ResourceRuntimeInfo", "RuntimeResourceState"] {
        assert!(
            resource_mod_source.contains(required),
            "runtime resource foundation should keep runtime-owned surface `{required}` public"
        );
    }

    for forbidden in [
        "ResourceInspectorAdapterKey",
        "ResourceTypeDescriptor",
        "inspector_adapter",
    ] {
        assert!(
            !resource_mod_source.contains(forbidden)
                && !resource_runtime_source.contains(forbidden),
            "runtime resource foundation should stop exposing editor inspector residue `{forbidden}`"
        );
    }
}
