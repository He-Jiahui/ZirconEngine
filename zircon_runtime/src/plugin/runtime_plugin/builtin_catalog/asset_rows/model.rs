use crate::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

use super::super::rows::BuiltinCatalogRow;

pub(super) const MODEL_BUILTIN_CATALOG_ROWS: &[BuiltinCatalogRow] = &[
    BuiltinCatalogRow {
        package_id: "gltf_importer",
        display_name: "glTF Importer",
        runtime_id: RuntimePluginId::GltfImporter,
        runtime_crate: "zircon_plugin_gltf_importer_runtime",
        capability: "runtime.plugin.gltf_importer",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
    BuiltinCatalogRow {
        package_id: "obj_importer",
        display_name: "OBJ Importer",
        runtime_id: RuntimePluginId::ObjImporter,
        runtime_crate: "zircon_plugin_obj_importer_runtime",
        capability: "runtime.plugin.obj_importer",
        target_modes: &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    },
];
