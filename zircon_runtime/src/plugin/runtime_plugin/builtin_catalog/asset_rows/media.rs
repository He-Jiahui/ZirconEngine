use crate::{RuntimePluginId, RuntimeTargetMode};

use super::super::rows::BuiltinCatalogRow;

pub(super) const MEDIA_BUILTIN_CATALOG_ROWS: &[BuiltinCatalogRow] = &[
    BuiltinCatalogRow {
        package_id: "texture_importer",
        display_name: "Texture Importer",
        runtime_id: RuntimePluginId::TextureImporter,
        runtime_crate: "zircon_plugin_texture_importer_runtime",
        capability: "runtime.plugin.texture_importer",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
    BuiltinCatalogRow {
        package_id: "audio_importer",
        display_name: "Audio Importer",
        runtime_id: RuntimePluginId::AudioImporter,
        runtime_crate: "zircon_plugin_audio_importer_runtime",
        capability: "runtime.plugin.audio_importer",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
];
