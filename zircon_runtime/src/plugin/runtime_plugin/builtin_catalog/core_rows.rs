mod content;
mod runtime;

use super::rows::BuiltinCatalogRow;
use content::CONTENT_BUILTIN_CATALOG_ROWS;
use runtime::runtime_builtin_catalog_rows;

pub(super) fn core_builtin_catalog_rows() -> impl Iterator<Item = &'static BuiltinCatalogRow> {
    runtime_builtin_catalog_rows().chain(CONTENT_BUILTIN_CATALOG_ROWS.iter())
}
