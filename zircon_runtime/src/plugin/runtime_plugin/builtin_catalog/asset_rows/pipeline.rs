use crate::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

use super::super::rows::BuiltinCatalogRow;

pub(super) const PIPELINE_BUILTIN_CATALOG_ROWS: &[BuiltinCatalogRow] = &[
    BuiltinCatalogRow {
        package_id: "shader_wgsl_importer",
        display_name: "WGSL Shader Importer",
        runtime_id: RuntimePluginId::ShaderWgslImporter,
        runtime_crate: "zircon_plugin_shader_wgsl_importer_runtime",
        capability: "runtime.plugin.shader_wgsl_importer",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
    BuiltinCatalogRow {
        package_id: "ui_document_importer",
        display_name: "UI Document Importer",
        runtime_id: RuntimePluginId::UiDocumentImporter,
        runtime_crate: "zircon_plugin_ui_document_importer_runtime",
        capability: "runtime.plugin.ui_document_importer",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
];
