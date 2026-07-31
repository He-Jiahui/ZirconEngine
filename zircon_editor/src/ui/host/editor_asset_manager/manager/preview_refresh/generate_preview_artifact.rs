use std::path::PathBuf;

use crate::core::asset::{
    builtin_asset_type_definition, ThumbnailPlaceholderPalette, ThumbnailProviderDescriptor,
};
use zircon_runtime::asset::importer::AssetImportError;
use zircon_runtime::asset::project::ProjectManager;

use crate::ui::host::editor_asset_manager::{AssetCatalogRecord, PreviewArtifactKey, PreviewCache};

pub(super) fn generate_preview_artifact(
    project: &ProjectManager,
    record: &AssetCatalogRecord,
    cache: &PreviewCache,
) -> Result<PathBuf, AssetImportError> {
    let key = PreviewArtifactKey::thumbnail(record.asset_uuid, &record.source_hash);
    let definition = builtin_asset_type_definition(record.kind).ok_or_else(|| {
        AssetImportError::Parse(format!(
            "asset type definition is missing for {:?}",
            record.kind
        ))
    })?;
    match definition.thumbnail() {
        ThumbnailProviderDescriptor::SourceImage => {
            let source_path = project.source_path_for_uri(&record.locator)?;
            let image = image::open(&source_path).map_err(|error| {
                AssetImportError::Parse(format!(
                    "failed to decode preview image {}: {error}",
                    source_path.display()
                ))
            })?;
            cache
                .write_thumbnail(&key, &image)
                .map_err(AssetImportError::from)
        }
        ThumbnailProviderDescriptor::Placeholder { palette, .. } => cache
            .write_kind_placeholder(&key, *palette)
            .map_err(AssetImportError::from),
        ThumbnailProviderDescriptor::Icon(_) => cache
            .write_kind_placeholder(&key, ThumbnailPlaceholderPalette::neutral())
            .map_err(AssetImportError::from),
        ThumbnailProviderDescriptor::Operation(operation) => Err(AssetImportError::Parse(format!(
            "thumbnail operation `{operation}` must be dispatched by the editor operation host"
        ))),
    }
}
