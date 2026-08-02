zircon_plugin_sdk::declare_plugin! {
    pub PHYSICS_DECLARATION {
        id: PLUGIN_ID = "physics",
        display_name: "Physics",
        category: runtime,
        module: MODULE_NAME = "physics.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_physics_runtime",
        module_description: "Physics world, query, constraint, and event services",
        targets: [client_runtime, server_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            PHYSICS_RUNTIME_CAPABILITY = "runtime.plugin.physics" => runtime_registration,
            PHYSICS_RAYCAST_CAPABILITY = "runtime.capability.physics.raycast" => runtime_registration,
            PHYSICS_OVERLAP_CAPABILITY = "runtime.capability.physics.overlap" => runtime_registration,
            PHYSICS_SHAPE_CAST_CAPABILITY = "runtime.capability.physics.shape_cast" => runtime_registration,
            PHYSICS_TRIGGER_EVENTS_CAPABILITY = "runtime.capability.physics.trigger_events" => runtime_registration,
            PHYSICS_CONSTRAINTS_CAPABILITY = "runtime.capability.physics.constraints" => runtime_registration,
            PHYSICS_SKELETAL_JOINTS_CAPABILITY = "runtime.capability.physics.skeletal_joints" => runtime_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_physics_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.physics.backend", contribution: "plugin.physics.runtime", schema: "zircon.runtime.physics-backend/1" },
                ],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[
    PHYSICS_RUNTIME_CAPABILITY,
    PHYSICS_RAYCAST_CAPABILITY,
    PHYSICS_OVERLAP_CAPABILITY,
    PHYSICS_SHAPE_CAST_CAPABILITY,
    PHYSICS_TRIGGER_EVENTS_CAPABILITY,
    PHYSICS_CONSTRAINTS_CAPABILITY,
    PHYSICS_SKELETAL_JOINTS_CAPABILITY,
];
