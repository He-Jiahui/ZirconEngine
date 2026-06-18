use crate::builtin::{RuntimePluginId, RuntimeTargetMode};

use super::super::rows::BuiltinCatalogRow;

const LANGUAGE_BUILTIN_CATALOG_ROWS: &[BuiltinCatalogRow] = &[BuiltinCatalogRow {
    package_id: "zr_vm_language",
    display_name: "ZrVM Language",
    runtime_id: RuntimePluginId::ZrVmLanguage,
    runtime_crate: "zircon_plugin_zr_vm_language_runtime",
    capability: "runtime.plugin.zr_vm_language",
    target_modes: &[
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ServerRuntime,
        RuntimeTargetMode::EditorHost,
    ],
}];

pub(in crate::plugin::runtime_plugin::builtin_catalog) fn language_builtin_catalog_rows(
) -> impl Iterator<Item = &'static BuiltinCatalogRow> {
    LANGUAGE_BUILTIN_CATALOG_ROWS.iter()
}
