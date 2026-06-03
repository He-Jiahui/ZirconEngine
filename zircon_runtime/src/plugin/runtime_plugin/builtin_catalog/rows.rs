use crate::{RuntimePluginId, RuntimeTargetMode};

use super::asset_rows::asset_builtin_catalog_rows;
use super::core_rows::core_builtin_catalog_rows;
use super::language::language_builtin_catalog_rows;
use super::render_rows::RENDER_BUILTIN_CATALOG_ROWS;

pub(super) struct BuiltinCatalogRow {
    pub package_id: &'static str,
    pub display_name: &'static str,
    pub runtime_id: RuntimePluginId,
    pub runtime_crate: &'static str,
    pub capability: &'static str,
    pub target_modes: &'static [RuntimeTargetMode],
}

pub(super) fn builtin_catalog_rows() -> impl Iterator<Item = &'static BuiltinCatalogRow> {
    core_builtin_catalog_rows()
        .chain(asset_builtin_catalog_rows())
        .chain(RENDER_BUILTIN_CATALOG_ROWS.iter())
        .chain(language_builtin_catalog_rows())
}
