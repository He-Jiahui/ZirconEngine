#[test]
fn review_f5_native_plugin_descriptor_abi_uses_typed_error() {
    let native_plugin_abi =
        include_str!("../../../../../../../plugin/native_plugin_loader/native_plugin_abi.rs");
    let plugin_load_error =
        include_str!("../../../../../../../plugin/native_plugin_loader/plugin_load_error.rs");
    let load_discovered =
        include_str!("../../../../../../../plugin/native_plugin_loader/load_discovered.rs");
    let native_boundary = include_str!(
        "../../../../../../../../../docs/engine-architecture/native-plugin-boundary.md"
    );

    for required in [
        "pub enum PluginLoadStage",
        "DescriptorProbe",
        "pub enum PluginLoadError",
        "MissingSymbol",
        "ContractMismatch",
        "InvalidPayload",
        "NullPointer",
        "expected:",
        "actual:",
    ] {
        assert!(
            plugin_load_error.contains(required),
            "unified plugin load error owner should contain `{required}`"
        );
    }

    for required in [
        "PluginLoadError::missing_symbol",
        "PluginLoadError::contract_mismatch",
        "PluginLoadError::invalid_payload",
        "PluginLoadError::null_pointer",
        "PluginLoadStage::DescriptorProbe",
        ") -> PluginLoadResult<NativePluginDescriptor>",
    ] {
        assert!(
            native_plugin_abi.contains(required),
            "native descriptor ABI should use the unified error contract `{required}`"
        );
    }

    assert!(
        plugin_load_error.contains("actual: \"symbol not exported\".to_string()"),
        "missing descriptor export should retain an explicit actual value"
    );
    assert!(
        load_discovered.contains("Err(error) => {")
            && load_discovered.contains("report.diagnostics.push(error.to_string());")
            && load_discovered.contains("return;")
            && load_discovered.contains("descriptor: Some(descriptor)"),
        "descriptor failure should return before the library enters the loaded collection"
    );

    let production = native_plugin_abi
        .split("#[cfg(test)]")
        .next()
        .expect("native plugin ABI production source");
    for forbidden in [
        "NativePluginDescriptorAbiError",
        "NativePluginDescriptorAbiResult",
        "Result<Option<NativePluginDescriptor>, String>",
        ".map_err(|error| error.to_string())",
    ] {
        assert!(
            !production.contains(forbidden),
            "descriptor ABI hard cut should remove `{forbidden}`"
        );
    }
    assert!(
        !load_discovered.contains("descriptor: None"),
        "descriptor hard cut should not restore an accepted descriptor-less library"
    );

    for anchor in [
        "Frameworks04 M3 PluginLoadError ABI hard cut",
        "descriptor probe no longer projects typed errors to String",
        "plugin/native_plugin_loader/plugin_load_error.rs",
    ] {
        assert!(
            native_boundary.contains(anchor),
            "native plugin boundary should record `{anchor}`"
        );
    }
}
