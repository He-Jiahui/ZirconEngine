use std::path::PathBuf;

use zircon_runtime::asset::importer::AssetImportError;
use zircon_runtime::asset::project::ProjectManager;

use crate::ui::host::editor_asset_manager::{AssetCatalogRecord, PreviewArtifactKey, PreviewCache};

use super::preview_palette::preview_palette;

pub(super) fn generate_preview_artifact(
    project: &ProjectManager,
    record: &AssetCatalogRecord,
    cache: &PreviewCache,
) -> Result<PathBuf, AssetImportError> {
    let key = PreviewArtifactKey::thumbnail(record.asset_uuid);
    match record.kind {
        zircon_runtime::asset::AssetKind::Texture => {
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
        zircon_runtime::asset::AssetKind::Data
        | zircon_runtime::asset::AssetKind::MaterialGraph
        | zircon_runtime::asset::AssetKind::Material
        | zircon_runtime::asset::AssetKind::PhysicsMaterial
        | zircon_runtime::asset::AssetKind::NavMesh
        | zircon_runtime::asset::AssetKind::NavigationSettings
        | zircon_runtime::asset::AssetKind::Terrain
        | zircon_runtime::asset::AssetKind::TerrainLayerStack
        | zircon_runtime::asset::AssetKind::TileSet
        | zircon_runtime::asset::AssetKind::TileMap
        | zircon_runtime::asset::AssetKind::Prefab
        | zircon_runtime::asset::AssetKind::Scene
        | zircon_runtime::asset::AssetKind::Model
        | zircon_runtime::asset::AssetKind::Sound
        | zircon_runtime::asset::AssetKind::Font
        | zircon_runtime::asset::AssetKind::AnimationSkeleton
        | zircon_runtime::asset::AssetKind::AnimationClip
        | zircon_runtime::asset::AssetKind::AnimationSequence
        | zircon_runtime::asset::AssetKind::AnimationGraph
        | zircon_runtime::asset::AssetKind::AnimationStateMachine
        | zircon_runtime::asset::AssetKind::Shader
        | zircon_runtime::asset::AssetKind::UiLayout
        | zircon_runtime::asset::AssetKind::UiWidget
        | zircon_runtime::asset::AssetKind::UiStyle => cache
            .write_kind_placeholder(&key, preview_palette(record.kind))
            .map_err(AssetImportError::from),
    }
}
