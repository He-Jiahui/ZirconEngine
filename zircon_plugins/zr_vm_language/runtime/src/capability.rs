zircon_plugin_sdk::declare_plugin! {
    pub ZR_VM_LANGUAGE_DECLARATION {
        id: PLUGIN_ID = "zr_vm_language",
        display_name: "ZrVM Language",
        category: runtime,
        module: MODULE_NAME = "zr_vm_language.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_zr_vm_language_runtime",
        module_description: "ZrVM project language backend and runtime integration",
        targets: [client_runtime, server_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            ZR_VM_LANGUAGE_RUNTIME_CAPABILITY = "runtime.plugin.zr_vm_language" => runtime_registration,
            ZR_VM_PROJECT_BACKEND_CAPABILITY = "runtime.script.backend.zr_vm_project" => runtime_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_zr_vm_language_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.script.backend", contribution: "plugin.zr_vm_language.project_backend", schema: "zircon.runtime.script-backend/1" },
                ],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[
    ZR_VM_LANGUAGE_RUNTIME_CAPABILITY,
    ZR_VM_PROJECT_BACKEND_CAPABILITY,
];
