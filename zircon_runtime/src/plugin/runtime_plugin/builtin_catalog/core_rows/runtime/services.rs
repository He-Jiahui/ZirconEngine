use crate::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

use super::super::super::rows::BuiltinCatalogRow;

pub(super) const RUNTIME_SERVICE_ROWS: &[BuiltinCatalogRow] = &[
    BuiltinCatalogRow {
        package_id: "physics",
        display_name: "Physics",
        runtime_id: RuntimePluginId::Physics,
        runtime_crate: "zircon_plugin_physics_runtime",
        capability: "runtime.plugin.physics",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::ServerRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
    BuiltinCatalogRow {
        package_id: "sound",
        display_name: "Sound",
        runtime_id: RuntimePluginId::Sound,
        runtime_crate: "zircon_plugin_sound_runtime",
        capability: "runtime.plugin.sound",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
    BuiltinCatalogRow {
        package_id: "texture",
        display_name: "Texture",
        runtime_id: RuntimePluginId::Texture,
        runtime_crate: "zircon_plugin_texture_runtime",
        capability: "runtime.plugin.texture",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
    BuiltinCatalogRow {
        package_id: "net",
        display_name: "Network",
        runtime_id: RuntimePluginId::Net,
        runtime_crate: "zircon_plugin_net_runtime",
        capability: "runtime.plugin.net",
        target_modes: &[
            RuntimeTargetMode::ServerRuntime,
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
];
