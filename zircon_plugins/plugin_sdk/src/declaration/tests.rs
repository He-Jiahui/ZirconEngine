use super::*;

crate::declare_plugin! {
    TEST_PLUGIN_DECLARATION {
        id: TEST_PLUGIN_ID = "sdk_test_plugin",
        display_name: "SDK Test Plugin",
        category: runtime,
        module: TEST_MODULE_NAME = "sdk_test_plugin.runtime",
        crate_name: TEST_RUNTIME_CRATE_NAME = "zircon_plugin_sdk_test",
        module_description: "Runtime metadata declaration fixture",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            TEST_CAPABILITY = "runtime.plugin.sdk_test_plugin" => runtime_registration,
            TEST_EDITOR_CAPABILITY = "editor.extension.sdk_test_plugin" => editor_registration,
            TEST_SHARED_CAPABILITY = "plugin.shared.sdk_test_plugin" => runtime_editor_registration,
            TEST_REQUESTED_ONLY_CAPABILITY = "plugin.optional.sdk_test_plugin" => requested_only,
        ],
        maturity: beta,
        packaging: [source_template, native_dynamic],
        native_projection: {
            plugin_id: TEST_NATIVE_PLUGIN_ID,
            requested_capabilities: TEST_NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: TEST_NATIVE_RUNTIME_ENTRY = "zircon_plugin_sdk_test_runtime_entry_v3",
                registration_manifest: TEST_NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [{
                    id: "sdk_test_plugin.tick",
                    module: "runtime",
                    stage: "Update",
                    order: 1,
                    sets: ["sdk_test_plugin"],
                    access: ["read:world"],
                    thread_affinity: "main-thread-only",
                    bridge_interface: "sdk_test_plugin.runtime",
                    bridge_method: "tick",
                }],
                events: [{
                    namespace: "sdk_test_plugin",
                    name: "ticked",
                    stable_hash: 7,
                    schema: "bytes",
                }],
                extensions: [{
                    point: "runtime.test",
                    contribution: "plugin.sdk_test_plugin",
                    schema: "zircon.runtime.test/1",
                }],
            },
            editor: {
                entry: TEST_NATIVE_EDITOR_ENTRY = "zircon_plugin_sdk_test_editor_entry_v3",
                registration_manifest: TEST_NATIVE_EDITOR_REGISTRATION_MANIFEST,
                modules: [{ name: "editor", kind: "editor" }],
                systems: [],
                events: [],
                extensions: [],
            },
        },
    }
}

#[test]
fn declare_plugin_projects_runtime_independent_metadata_and_native_abi() {
    assert_eq!(TEST_RUNTIME_CRATE_NAME, "zircon_plugin_sdk_test");
    assert_eq!(
        TEST_PLUGIN_DECLARATION.declared_targets(),
        &[PluginTarget::ClientRuntime, PluginTarget::EditorHost]
    );
    assert_eq!(
        TEST_PLUGIN_DECLARATION.declared_platforms(),
        &[
            PluginPlatform::Windows,
            PluginPlatform::Linux,
            PluginPlatform::Macos,
        ]
    );
    assert_eq!(
        TEST_PLUGIN_DECLARATION.declared_maturity(),
        PluginMaturityLevel::Beta
    );
    assert_eq!(
        TEST_PLUGIN_DECLARATION.declared_packaging(),
        &[
            PluginPackaging::SourceTemplate,
            PluginPackaging::NativeDynamic
        ]
    );
    assert_eq!(
        TEST_PLUGIN_DECLARATION.capability_roles(),
        &[
            PluginCapabilityRole::RuntimeRegistration,
            PluginCapabilityRole::EditorRegistration,
            PluginCapabilityRole::RuntimeEditorRegistration,
            PluginCapabilityRole::RequestedOnly,
        ]
    );
    assert_eq!(TEST_NATIVE_PLUGIN_ID, b"sdk_test_plugin\0");
    assert_eq!(
        TEST_NATIVE_RUNTIME_ENTRY.cstr(),
        b"zircon_plugin_sdk_test_runtime_entry_v3\0"
    );
    assert_eq!(
        TEST_NATIVE_RUNTIME_ENTRY.name(),
        "zircon_plugin_sdk_test_runtime_entry_v3"
    );
    assert_eq!(
        TEST_NATIVE_REQUESTED_CAPABILITIES,
        concat!(
            "runtime.plugin.sdk_test_plugin\n",
            "editor.extension.sdk_test_plugin\n",
            "plugin.shared.sdk_test_plugin\n",
            "plugin.optional.sdk_test_plugin\0",
        )
        .as_bytes()
    );
    assert_eq!(
        TEST_NATIVE_RUNTIME_REGISTRATION_MANIFEST,
        concat!(
            "schema = \"zircon.native.registration-manifest/3\"\n",
            "capabilities = [\n",
            "  \"runtime.plugin.sdk_test_plugin\",\n",
            "  \"plugin.shared.sdk_test_plugin\",\n",
            "]\n",
            "[[modules]]\nname = \"runtime\"\nkind = \"runtime\"\n",
            "[[systems]]\nid = \"sdk_test_plugin.tick\"\n",
            "module = \"runtime\"\nstage = \"Update\"\norder = 1\n",
            "sets = [\"sdk_test_plugin\"]\n",
            "access = [\"read:world\"]\n",
            "thread_affinity = \"main-thread-only\"\n",
            "bridge_interface = \"sdk_test_plugin.runtime\"\n",
            "bridge_method = \"tick\"\n",
            "[[events]]\nnamespace = \"sdk_test_plugin\"\nname = \"ticked\"\n",
            "stable_hash = 7\nschema = \"bytes\"\n",
            "[[extensions]]\npoint = \"runtime.test\"\n",
            "contribution = \"plugin.sdk_test_plugin\"\n",
            "schema = \"zircon.runtime.test/1\"\n\0",
        )
        .as_bytes()
    );
    assert_eq!(
        TEST_NATIVE_EDITOR_ENTRY.cstr(),
        b"zircon_plugin_sdk_test_editor_entry_v3\0"
    );
    assert_eq!(
        TEST_NATIVE_EDITOR_ENTRY.name(),
        "zircon_plugin_sdk_test_editor_entry_v3"
    );
    assert_eq!(
        TEST_NATIVE_EDITOR_REGISTRATION_MANIFEST,
        concat!(
            "schema = \"zircon.native.registration-manifest/3\"\n",
            "capabilities = [\n",
            "  \"editor.extension.sdk_test_plugin\",\n",
            "  \"plugin.shared.sdk_test_plugin\",\n",
            "]\n",
            "[[modules]]\nname = \"editor\"\nkind = \"editor\"\n\0",
        )
        .as_bytes()
    );
}

#[cfg(feature = "runtime")]
#[test]
fn declare_plugin_projects_runtime_descriptor_metadata() {
    let descriptor = TEST_PLUGIN_DECLARATION.runtime_descriptor("zircon_plugin_sdk_test");

    assert_eq!(descriptor.package_id(), TEST_PLUGIN_ID);
    assert_eq!(descriptor.runtime_id().key(), TEST_PLUGIN_ID);
    assert_eq!(descriptor.category(), "runtime");
    assert_eq!(descriptor.maturity(), RuntimePluginMaturity::Beta);
    assert_eq!(
        descriptor.target_modes(),
        TEST_PLUGIN_DECLARATION.target_modes()
    );
    assert_eq!(
        descriptor.capabilities(),
        [
            TEST_CAPABILITY.to_string(),
            TEST_SHARED_CAPABILITY.to_string(),
        ]
    );
    assert_eq!(descriptor.module_descriptor().name, TEST_MODULE_NAME);
    assert_eq!(
        descriptor.module_descriptor().init_level,
        zircon_runtime::core::InitLevel::Post
    );
    assert!(
        descriptor
            .module_descriptor()
            .module_dependencies
            .is_empty()
    );
    let manifest = descriptor.package_manifest();
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == zircon_runtime::plugin::PluginModuleKind::Runtime)
        .expect("runtime module projection");
    assert_eq!(runtime_module.name, TEST_MODULE_NAME);
    assert_eq!(
        runtime_module.init_level,
        zircon_runtime::core::InitLevel::Post
    );
    assert!(runtime_module.module_dependencies.is_empty());
    assert_eq!(manifest.capabilities, descriptor.capabilities());
    assert_eq!(
        manifest.default_packaging.as_slice(),
        TEST_PLUGIN_DECLARATION.default_packaging()
    );
    assert_eq!(
        TEST_PLUGIN_DECLARATION.supported_platforms(),
        &[
            ExportTargetPlatform::Windows,
            ExportTargetPlatform::Linux,
            ExportTargetPlatform::Macos,
        ]
    );
}

#[cfg(feature = "runtime")]
#[test]
#[should_panic(expected = "must be a canonical RuntimePluginId key")]
fn declare_plugin_runtime_projection_rejects_non_canonical_plugin_ids() {
    let declaration = PluginDeclaration::new(
        "GLTF",
        "Non-canonical plugin",
        "runtime",
        "non_canonical.runtime",
        "Canonical runtime ID guard fixture",
        &[PluginTarget::ClientRuntime],
        &[PluginPlatform::Windows],
        &["runtime.plugin.non_canonical"],
        &[PluginCapabilityRole::RuntimeRegistration],
        PluginMaturityLevel::Experimental,
        &[PluginPackaging::LibraryEmbed],
    );

    let _ = declaration.runtime_declaration("non_canonical_runtime");
}
