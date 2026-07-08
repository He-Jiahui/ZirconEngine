pub(super) const EXPECTED_RUNTIME_06_SOURCE_FILES: &[&str] = &[
    "src/plugin/mod.rs",
    "src/plugin/native.rs",
    "src/plugin/native_plugin_loader/mod.rs",
    "src/plugin/native_plugin_loader/abi_declarations.rs",
    "src/plugin/native_plugin_loader/native_plugin_abi.rs",
    "src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs",
    "src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs",
    "src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_failures.rs",
    "src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs",
    "src/script/vm/tests.rs",
    "src/script/vm/tests/lifecycle_failures.rs",
    "src/tests/runtime_absorption/plan_status/cargo_gates/early.rs",
    "src/tests/runtime_absorption/plugin_surface_lifecycle.rs",
    "../zircon_plugins/native_dynamic_fixture/native/src/lib.rs",
];

pub(super) const EXPECTED_RUNTIME_06_MIRROR_DOCS: &[&str] = &[
    "../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md",
    "../docs/plans/zircon_runtime/runtime/index.md",
    "../docs/engine-architecture/native-plugin-boundary.md",
    "../docs/engine-architecture/runtime-interface-convergence.md",
    "../docs/engine-architecture/runtime-architecture-review-m0.md",
];

pub(super) const V1_V2_PATTERNS: &[&str] = &[
    "NativePluginAbiV1",
    "NativePluginAbiV2",
    "DESCRIPTOR_SYMBOL_V1",
    "DESCRIPTOR_SYMBOL_V2",
    "ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1",
    "ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2",
];

pub(super) const NATIVE_LOADER_TEST_PATTERNS: &[&str] = &[
    "NativePluginAbi",
    "NativePluginEntryReport",
    "NativePluginBehavior",
    "NativePluginLoader",
    "ZIRCON_NATIVE_PLUGIN_STATUS",
];

pub(super) const LIFECYCLE_FALLBACK_TESTS: &[&str] = &[
    "vm_lifecycle_fallback_activate_bad_entry_module_surfaces_vm_error",
    "vm_lifecycle_fallback_missing_optional_export_returns_none_not_error",
    "vm_lifecycle_fallback_deactivate_is_idempotent_after_unload",
    "vm_lifecycle_fallback_empty_arguments_do_not_require_real_backend",
];
