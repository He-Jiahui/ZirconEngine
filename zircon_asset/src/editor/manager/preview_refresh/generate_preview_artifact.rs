use std::path::PathBuf;

use crate::project::ProjectManager;
use crate::{AssetCatalogRecord, AssetImportError, PreviewArtifactKey, PreviewCache};

use super::preview_palette::preview_palette;

pub(super) fn generate_preview_artifact(
    project: &ProjectManager,
    record: &AssetCatalogRecord,
    cache: &PreviewCache,
) -> Result<PathBuf, AssetImportError> {
    let key = PreviewArtifactKey::thumbnail(record.asset_uuid);
    match record.kind {
        crate::AssetKind::Texture => {
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
        crate::AssetKind::Material
        | crate::AssetKind::Scene
        | crate::AssetKind::Model
        | crate::AssetKind::Shader
        | crate::AssetKind::UiLayout
        | crate::AssetKind::UiWidget
        | crate::AssetKind::UiStyle => cache
            .write_kind_placeholder(&key, preview_palette(record.kind))
            .map_err(AssetImportError::from),
    }
}
