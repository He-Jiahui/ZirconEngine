use crate::{RuntimePluginId, RuntimeTargetMode};

use super::super::super::rows::BuiltinCatalogRow;

pub(super) const RUNTIME_SYSTEM_ROWS: &[BuiltinCatalogRow] = &[
    BuiltinCatalogRow {
        package_id: "navigation",
        display_name: "Navigation",
        runtime_id: RuntimePluginId::Navigation,
        runtime_crate: "zircon_plugin_navigation_runtime",
        capability: "runtime.plugin.navigation",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::ServerRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
    BuiltinCatalogRow {
        package_id: "particles",
        display_name: "Particles",
        runtime_id: RuntimePluginId::Particles,
        runtime_crate: "zircon_plugin_particles_runtime",
        capability: "runtime.plugin.particles",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
    BuiltinCatalogRow {
        package_id: "animation",
        display_name: "Animation",
        runtime_id: RuntimePluginId::Animation,
        runtime_crate: "zircon_plugin_animation_runtime",
        capability: "runtime.plugin.animation",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::ServerRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
];
