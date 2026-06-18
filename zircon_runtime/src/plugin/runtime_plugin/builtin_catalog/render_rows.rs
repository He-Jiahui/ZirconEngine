use crate::builtin::{RuntimePluginId, RuntimeTargetMode};

use super::rows::BuiltinCatalogRow;

pub(super) const RENDER_BUILTIN_CATALOG_ROWS: &[BuiltinCatalogRow] = &[
    BuiltinCatalogRow {
        package_id: "rendering",
        display_name: "Rendering",
        runtime_id: RuntimePluginId::Rendering,
        runtime_crate: "zircon_plugin_rendering_runtime",
        capability: "runtime.plugin.rendering",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
    BuiltinCatalogRow {
        package_id: "virtual_geometry",
        display_name: "Virtual Geometry",
        runtime_id: RuntimePluginId::VirtualGeometry,
        runtime_crate: "zircon_plugin_virtual_geometry_runtime",
        capability: "runtime.plugin.virtual_geometry",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
    BuiltinCatalogRow {
        package_id: "hybrid_gi",
        display_name: "Hybrid GI",
        runtime_id: RuntimePluginId::HybridGi,
        runtime_crate: "zircon_plugin_hybrid_gi_runtime",
        capability: "runtime.plugin.hybrid_gi",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
    BuiltinCatalogRow {
        package_id: "solari",
        display_name: "Solari",
        runtime_id: RuntimePluginId::Solari,
        runtime_crate: "zircon_plugin_solari_runtime",
        capability: "runtime.plugin.solari",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
];
