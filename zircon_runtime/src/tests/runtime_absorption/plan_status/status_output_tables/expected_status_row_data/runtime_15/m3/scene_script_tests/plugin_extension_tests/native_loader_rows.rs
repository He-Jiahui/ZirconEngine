type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 native live-host tests folder split",
        &[
            "runtime_15_native_live_host_tests_folder_split_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_live_host/tests.rs",
            "plugin/native_plugin_loader/native_plugin_live_host/tests/bridge_bindings.rs",
            "plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs",
            "runtime_15_native_live_host_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 native plugin loader real fixture test folder split",
        &[
            "runtime_15_native_plugin_loader_real_fixture_tests_folder_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/native_plugin_loader.rs",
            "tests/plugin_extensions/native_plugin_loader/real_fixture.rs",
            "runtime_15_native_plugin_loader_real_fixture_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 extension registry bridge test folder split",
        &[
            "runtime_15_extension_registry_bridge_tests_folder_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/extension_registry_bridge.rs",
            "tests/plugin_extensions/extension_registry_bridge/basics.rs",
            "tests/plugin_extensions/extension_registry_bridge/diagnostics.rs",
            "runtime_15_extension_registry_bridge_tests_are_folder_backed",
        ],
    ),
];
