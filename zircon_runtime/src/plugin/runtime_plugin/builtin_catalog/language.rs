mod classification;
mod rows;

use super::super::RuntimePluginDescriptor;
use super::rows::BuiltinCatalogRow;

pub(super) fn is_language_descriptor(package_id: &str) -> bool {
    classification::is_language_descriptor(package_id)
}

pub(super) fn classify_language_descriptor(
    package_id: &str,
    descriptor: RuntimePluginDescriptor,
) -> RuntimePluginDescriptor {
    classification::classify_language_descriptor(package_id, descriptor)
}

pub(super) fn language_builtin_catalog_rows() -> impl Iterator<Item = &'static BuiltinCatalogRow> {
    rows::language_builtin_catalog_rows()
}
