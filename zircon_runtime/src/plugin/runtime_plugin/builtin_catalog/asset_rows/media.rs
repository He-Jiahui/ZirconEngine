use crate::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

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
    BuiltinCatalogRow {
        package_id: "opus_importer",
        display_name: "Opus Audio Importer",
        runtime_id: RuntimePluginId::OpusImporter,
        runtime_crate: "zircon_plugin_opus_importer_runtime",
        capability: "runtime.plugin.opus_importer",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
];
