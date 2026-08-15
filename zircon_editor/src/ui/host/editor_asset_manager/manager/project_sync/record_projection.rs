use zircon_runtime::asset::importer::AssetImportError;
use zircon_runtime::asset::project::{AssetMetaDocument, PreviewState, ProjectManager};
use zircon_runtime::core::resource::{ResourceRecord, ResourceState};

use crate::ui::host::editor_asset_manager::{AssetCatalogRecord, PreviewArtifactKey, PreviewCache};

use super::display_name_for_path::display_name_for_path;
use super::meta_path_for_source::meta_path_for_source;
use super::preview_source_mtime::preview_source_mtime;
use crate::ui::host::editor_asset_manager::manager::reference_analysis::direct_references;

pub(super) fn project_catalog_record(
    project: &ProjectManager,
    preview_cache: &PreviewCache,
    metadata: &ResourceRecord,
) -> Result<Option<AssetCatalogRecord>, AssetImportError> {
    let locator = metadata.primary_locator().clone();
    if locator.label().is_some() {
        return Ok(None);
    }

    let source_path = project.source_path_for_uri(&locator)?;
    let meta_path = meta_path_for_source(&source_path);
    let meta = AssetMetaDocument::load(&meta_path)?;
    let preview_state = meta.preview_state;
    let direct_references = if metadata.state == ResourceState::Ready {
        let imported = project.load_artifact_by_id(metadata.id())?;
        direct_references(&imported)
    } else {
        Vec::new()
    };
    let preview_artifact_path = preview_cache.path_for(&PreviewArtifactKey::thumbnail(
        meta.uuid,
        &metadata.source_hash,
    ));
    let file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();
    let extension = source_path
        .extension()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let diagnostics = metadata
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.clone())
        .collect::<Vec<_>>();

    Ok(Some(AssetCatalogRecord {
        asset_uuid: meta.uuid,
        asset_id: metadata.id(),
        locator,
        kind: metadata.kind,
        display_name: display_name_for_path(&source_path, metadata.primary_locator()),
        file_name,
        extension,
        meta_path,
        meta,
        source_mtime_unix_ms: preview_source_mtime(&source_path),
        source_hash: metadata.source_hash.clone(),
        preview_state,
        preview_artifact_path,
        dirty: preview_state == PreviewState::Dirty,
        diagnostics,
        direct_references,
    }))
}
