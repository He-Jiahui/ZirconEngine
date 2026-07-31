#[test]
fn review_f5_native_plugin_entry_abi_uses_typed_error() {
    let native_plugin_abi =
        include_str!("../../../../../../../plugin/native_plugin_loader/native_plugin_abi.rs");
    let plugin_load_error =
        include_str!("../../../../../../../plugin/native_plugin_loader/plugin_load_error.rs");
    let load_discovered =
        include_str!("../../../../../../../plugin/native_plugin_loader/load_discovered.rs");
    let runtime_abi =
        include_str!("../../../../../../../plugin/native_plugin_loader/abi_declarations.rs");
    let sdk_native =
        include_str!("../../../../../../../../../zircon_plugins/plugin_sdk/src/native.rs");
    let sdk_dist = include_str!("../../../../../../../../../zircon_plugins/plugin_sdk/src/dist.rs");
    let fixture_manifest = include_str!(
        "../../../../../../../../../zircon_plugins/native_dynamic_fixture/native/Cargo.toml"
    );
    let real_fixture = include_str!(
        "../../../../../../../tests/plugin_extensions/native_plugin_loader/real_fixture.rs"
    );
    let native_boundary = include_str!(
        "../../../../../../../../../docs/engine-architecture/native-plugin-boundary.md"
    );

    for required in [
        "RuntimeEntry",
        "EditorEntry",
        "MissingSymbol",
        "ContractMismatch",
        "InvalidPayload",
        "NullPointer",
        "expected:",
        "actual:",
    ] {
        assert!(
            plugin_load_error.contains(required),
            "unified plugin entry error owner should contain `{required}`"
        );
    }

    for required in [
        "PluginLoadError::missing_symbol",
        "PluginLoadError::contract_mismatch",
        "PluginLoadError::invalid_payload",
        "PluginLoadError::null_pointer",
        "PluginLoadStage::from(module_kind)",
        ") -> PluginLoadResult<NativePluginEntryReport>",
    ] {
        assert!(
            native_plugin_abi.contains(required),
            "native entry ABI should use the unified error contract `{required}`"
        );
    }
    assert!(
        plugin_load_error.contains("actual: \"symbol not exported\".to_string()"),
        "missing entry export should retain an explicit actual value"
    );
    assert!(
        load_discovered.contains("fn load_requested_entry(")
            && load_discovered.contains("PluginModuleKind::Runtime")
            && load_discovered.contains("PluginModuleKind::Editor")
            && load_discovered.contains("report.diagnostics.push(error.to_string());")
            && load_discovered.contains("return;"),
        "a requested entry failure should reject the library after recording its typed diagnostic"
    );
    for source in [runtime_abi, sdk_native] {
        assert!(
            source.contains("ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH: u32 = 5")
                && source.contains("pub layout_epoch: u32"),
            "V3 entry protocol should use a distinct report layout epoch that rejects old binaries"
        );
        for field in [
            "required_capabilities: *const c_char",
            "denied_capabilities: *const c_char",
        ] {
            assert!(
                source.contains(field),
                "runtime and SDK V3 entry reports should share `{field}`"
            );
        }
    }
    for required in [
        "CapabilityNegotiation",
        "missing_required",
        "denied",
        "diagnostics: Vec<String>",
        "capability_negotiation_details",
    ] {
        assert!(
            plugin_load_error.contains(required) || native_plugin_abi.contains(required),
            "unified entry loading should retain structured capability outcome `{required}`"
        );
    }
    assert!(
        sdk_dist.contains("concat!(")
            && sdk_dist.contains("required_capabilities:")
            && sdk_dist.contains("denied_capabilities:"),
        "SDK dist reports should materialize required and denied capability declarations without runtime text parsing"
    );
    assert!(
        fixture_manifest.contains("required_capability_missing = []")
            && real_fixture
                .contains("native_loader_reports_structured_missing_required_capability")
            && real_fixture.contains("native v3 entry missing negotiated host ABI table"),
        "real native fixture should inject a missing required capability and preserve entry diagnostics"
    );

    let raw_epoch_read = native_plugin_abi
        .find("report.cast::<u32>().read_unaligned()")
        .expect("raw entry report layout epoch read");
    let full_report_reference = native_plugin_abi
        .find("&*report")
        .expect("full entry report reference");
    assert!(
        raw_epoch_read < full_report_reference,
        "the loader must reject an old short report from its raw header before creating a full new-layout reference"
    );
    let layout_check = native_plugin_abi[raw_epoch_read..]
        .find("entry_report.layout_epoch")
        .map(|offset| raw_epoch_read + offset)
        .expect("entry report layout epoch check");
    let required_read = native_plugin_abi
        .find("read_required_c_string(abi.required_capabilities")
        .expect("required capability pointer read");
    assert!(
        layout_check < required_read,
        "the loader must reject an old report layout before reading newly appended pointers"
    );
    assert!(
        native_plugin_abi.contains("report.diagnostics,")
            && plugin_load_error.contains("diagnostics={diagnostics:?}"),
        "capability rejection should preserve entry and callback diagnostics"
    );

    let production = native_plugin_abi
        .split("#[cfg(test)]")
        .next()
        .expect("native plugin ABI production source");
    for forbidden in [
        "NativePluginEntryAbiError",
        "NativePluginEntryAbiResult",
        "Result<NativePluginEntryReport, String>",
        "call_native_plugin_entry_result",
        ".map_err(|error| error.to_string())",
    ] {
        assert!(
            !production.contains(forbidden),
            "entry ABI hard cut should remove `{forbidden}`"
        );
    }

    for anchor in [
        "Frameworks04 M3 PluginLoadError ABI hard cut",
        "entry ABI no longer projects typed errors to String",
        "plugin/native_plugin_loader/plugin_load_error.rs",
    ] {
        assert!(
            native_boundary.contains(anchor),
            "native plugin boundary should record `{anchor}`"
        );
    }
}
