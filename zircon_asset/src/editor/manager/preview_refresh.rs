use std::path::{Path, PathBuf};

use crate::{
    AssetCatalogRecord, AssetImportError, AssetUri, AssetUuid, PreviewArtifactKey, PreviewCache,
    PreviewState, ProjectManager,
};

use super::default_editor_asset_manager::DefaultEditorAssetManager;
use super::super::{preview::PreviewPalette, EditorAssetChangeKind, EditorAssetChangeRecord};

impl DefaultEditorAssetManager {
    pub fn mark_preview_dirty(
        &self,
        asset_uuid: AssetUuid,
    ) -> Result<Option<AssetCatalogRecord>, AssetImportError> {
        let change = {
            let mut state = self
                .state
                .write()
                .expect("editor asset state lock poisoned");
            let updated = {
                let Some(record) = state.catalog_by_uuid.get_mut(&asset_uuid) else {
                    return Ok(None);
                };
                record.preview_state = PreviewState::Dirty;
                record.dirty = true;
                record.meta.preview_state = PreviewState::Dirty;
                record.meta.save(&record.meta_path)?;
                record.clone()
            };
            state.preview_scheduler.mark_dirty(asset_uuid);
            Some(EditorAssetChangeRecord {
                kind: EditorAssetChangeKind::PreviewChanged,
                catalog_revision: state.catalog_revision,
                uuid: Some(updated.asset_uuid.to_string()),
                locator: Some(updated.locator.to_string()),
            })
        };
        if let Some(change) = change {
            self.broadcast(change);
        }
        Ok(self.record_by_uuid(asset_uuid))
    }

    pub fn request_preview_refresh(
        &self,
        asset_uuid: AssetUuid,
        visible: bool,
    ) -> Result<Option<AssetCatalogRecord>, AssetImportError> {
        let change = {
            let mut state = self
                .state
                .write()
                .expect("editor asset state lock poisoned");
            let should_refresh = state.preview_scheduler.request_refresh(asset_uuid, visible);
            let catalog_revision = state.catalog_revision;
            let cache = state.preview_cache.as_ref().cloned().ok_or_else(|| {
                AssetImportError::Parse("preview cache is not initialized".to_string())
            })?;
            let project = state.project.as_ref().cloned().ok_or_else(|| {
                AssetImportError::Parse("editor project is not initialized".to_string())
            })?;
            let Some(record) = state.catalog_by_uuid.get_mut(&asset_uuid) else {
                return Ok(None);
            };
            if !should_refresh {
                return Ok(Some(record.clone()));
            }

            match generate_preview_artifact(&project, record, &cache) {
                Ok(path) => {
                    record.preview_artifact_path = path;
                    record.preview_state = PreviewState::Ready;
                    record.dirty = false;
                    record.meta.preview_state = PreviewState::Ready;
                    record.meta.save(&record.meta_path)?;
                }
                Err(error) => {
                    record.preview_state = PreviewState::Error;
                    record.dirty = false;
                    record.meta.preview_state = PreviewState::Error;
                    record.meta.save(&record.meta_path)?;
                    return Err(error);
                }
            }

            Some(EditorAssetChangeRecord {
                kind: EditorAssetChangeKind::PreviewChanged,
                catalog_revision,
                uuid: Some(record.asset_uuid.to_string()),
                locator: Some(record.locator.to_string()),
            })
        };
        if let Some(change) = change {
            self.broadcast(change);
        }
        Ok(self.record_by_uuid(asset_uuid))
    }
}

fn generate_preview_artifact(
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

fn preview_palette(kind: crate::AssetKind) -> PreviewPalette {
    match kind {
        crate::AssetKind::Texture => PreviewPalette {
            primary: [74, 127, 173, 255],
            secondary: [35, 55, 82, 255],
            accent: [182, 220, 255, 255],
            banner: [212, 236, 255, 255],
        },
        crate::AssetKind::Material => PreviewPalette {
            primary: [156, 112, 66, 255],
            secondary: [74, 49, 28, 255],
            accent: [237, 204, 158, 255],
            banner: [247, 231, 199, 255],
        },
        crate::AssetKind::Scene => PreviewPalette {
            primary: [67, 118, 91, 255],
            secondary: [31, 60, 48, 255],
            accent: [180, 228, 200, 255],
            banner: [220, 245, 228, 255],
        },
        crate::AssetKind::Model => PreviewPalette {
            primary: [102, 97, 145, 255],
            secondary: [46, 43, 73, 255],
            accent: [210, 204, 250, 255],
            banner: [229, 225, 255, 255],
        },
        crate::AssetKind::Shader => PreviewPalette {
            primary: [170, 80, 97, 255],
            secondary: [78, 31, 43, 255],
            accent: [255, 208, 219, 255],
            banner: [255, 231, 236, 255],
        },
        crate::AssetKind::UiLayout => PreviewPalette {
            primary: [65, 112, 148, 255],
            secondary: [29, 54, 71, 255],
            accent: [190, 228, 250, 255],
            banner: [226, 243, 255, 255],
        },
        crate::AssetKind::UiWidget => PreviewPalette {
            primary: [116, 98, 169, 255],
            secondary: [52, 44, 81, 255],
            accent: [221, 210, 255, 255],
            banner: [238, 232, 255, 255],
        },
        crate::AssetKind::UiStyle => PreviewPalette {
            primary: [164, 113, 55, 255],
            secondary: [79, 53, 24, 255],
            accent: [246, 217, 173, 255],
            banner: [255, 239, 214, 255],
        },
    }
}

pub(super) fn display_name_for_locator(locator: &AssetUri) -> String {
    locator
        .label()
        .map(str::to_string)
        .or_else(|| {
            Path::new(locator.path())
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| locator.to_string())
}
