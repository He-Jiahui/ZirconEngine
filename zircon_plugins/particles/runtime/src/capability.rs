zircon_plugin_sdk::declare_plugin! {
    pub PARTICLES_DECLARATION {
        id: PLUGIN_ID = "particles",
        display_name: "Particles",
        category: runtime,
        module: MODULE_NAME = "particles.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_particles_runtime",
        module_description: "Particle simulation runtime services",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            PARTICLES_RUNTIME_CAPABILITY = "runtime.plugin.particles" => runtime_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_particles_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.particles.simulation", contribution: "plugin.particles.runtime", schema: "zircon.runtime.particles-simulation/1" },
                ],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[PARTICLES_RUNTIME_CAPABILITY];
