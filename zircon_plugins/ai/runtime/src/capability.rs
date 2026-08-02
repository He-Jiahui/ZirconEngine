zircon_plugin_sdk::declare_plugin! {
    pub AI_DECLARATION {
        id: PLUGIN_ID = "ai",
        display_name: "AI",
        category: runtime,
        module: MODULE_NAME = "ai.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_ai_runtime",
        module_description: "Behavior tree, blackboard, and perception runtime services",
        targets: [client_runtime, server_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            AI_RUNTIME_CAPABILITY = "runtime.plugin.ai" => runtime_registration,
            AI_BEHAVIOR_TREE_CAPABILITY = "runtime.feature.ai.behavior_tree" => runtime_registration,
            AI_BLACKBOARD_CAPABILITY = "runtime.feature.ai.blackboard" => runtime_registration,
            AI_PERCEPTION_CAPABILITY = "runtime.feature.ai.perception" => runtime_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_ai_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[
    AI_RUNTIME_CAPABILITY,
    AI_BEHAVIOR_TREE_CAPABILITY,
    AI_BLACKBOARD_CAPABILITY,
    AI_PERCEPTION_CAPABILITY,
];
