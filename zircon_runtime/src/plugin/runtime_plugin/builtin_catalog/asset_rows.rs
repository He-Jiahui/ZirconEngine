mod media;
mod model;
mod pipeline;

use super::rows::BuiltinCatalogRow;
use media::MEDIA_BUILTIN_CATALOG_ROWS;
use model::MODEL_BUILTIN_CATALOG_ROWS;
use pipeline::PIPELINE_BUILTIN_CATALOG_ROWS;

pub(super) fn asset_builtin_catalog_rows() -> impl Iterator<Item = &'static BuiltinCatalogRow> {
    MODEL_BUILTIN_CATALOG_ROWS
        .iter()
        .chain(MEDIA_BUILTIN_CATALOG_ROWS.iter())
        .chain(PIPELINE_BUILTIN_CATALOG_ROWS.iter())
}
