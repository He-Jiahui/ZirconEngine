#[path = "runtime/services.rs"]
mod services;
#[path = "runtime/systems.rs"]
mod systems;

use super::super::rows::BuiltinCatalogRow;
use services::RUNTIME_SERVICE_ROWS;
use systems::RUNTIME_SYSTEM_ROWS;

pub(super) fn runtime_builtin_catalog_rows() -> impl Iterator<Item = &'static BuiltinCatalogRow> {
    RUNTIME_SERVICE_ROWS
        .iter()
        .chain(RUNTIME_SYSTEM_ROWS.iter())
}
