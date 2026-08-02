zircon_plugin_sdk::declare_plugin! {
    pub NAVIGATION_DECLARATION {
        id: PLUGIN_ID = "navigation",
        display_name: "Navigation",
        category: runtime,
        module: MODULE_NAME = "navigation.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_navigation_runtime",
        module_description: "Navigation runtime services and Recast backend",
        targets: [client_runtime, server_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            NAVIGATION_RUNTIME_CAPABILITY = "runtime.plugin.navigation" => runtime_registration,
            NAVIGATION_RECAST_CAPABILITY = "runtime.plugin.navigation.recast" => runtime_registration,
        ],
        maturity: beta,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_navigation_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.navigation.backend", contribution: "plugin.navigation.recast", schema: "zircon.runtime.navigation-backend/1" },
                ],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] =
    &[NAVIGATION_RUNTIME_CAPABILITY, NAVIGATION_RECAST_CAPABILITY];
