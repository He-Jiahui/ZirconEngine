use crate::builtin::{RuntimePluginId, RuntimeTargetMode};

use super::super::rows::BuiltinCatalogRow;

pub(super) const CONTENT_BUILTIN_CATALOG_ROWS: &[BuiltinCatalogRow] = &[
    BuiltinCatalogRow {
        package_id: "terrain",
        display_name: "Terrain",
        runtime_id: RuntimePluginId::Terrain,
        runtime_crate: "zircon_plugin_terrain_runtime",
        capability: "runtime.plugin.terrain",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
    BuiltinCatalogRow {
        package_id: "tilemap_2d",
        display_name: "Tilemap 2D",
        runtime_id: RuntimePluginId::Tilemap2d,
        runtime_crate: "zircon_plugin_tilemap_2d_runtime",
        capability: "runtime.plugin.tilemap_2d",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
    BuiltinCatalogRow {
        package_id: "prefab_tools",
        display_name: "Prefab Tools",
        runtime_id: RuntimePluginId::PrefabTools,
        runtime_crate: "zircon_plugin_prefab_tools_runtime",
        capability: "runtime.plugin.prefab_tools",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
];
