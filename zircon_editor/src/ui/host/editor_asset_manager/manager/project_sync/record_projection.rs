use zircon_runtime::asset::project::{PreviewState, ProjectCatalogInputRecord};
use zircon_runtime::asset::ReferenceRepair;
use zircon_runtime::core::resource::ResourceState;

use crate::ui::host::editor_asset_manager::{AssetCatalogRecord, PreviewArtifactKey, PreviewCache};

use super::display_name_for_path::display_name_for_path;

pub(super) fn project_catalog_record(
    preview_cache: &PreviewCache,
    catalog_input: &ProjectCatalogInputRecord,
) -> Option<AssetCatalogRecord> {
    let metadata = catalog_input.resource();
    let locator = metadata.primary_locator().clone();
    if locator.label().is_some() {
        return None;
    }

    let source_path = catalog_input.source_path();
    let meta_path = catalog_input.meta_path().to_path_buf();
    let meta = catalog_input.meta().clone();
    let preview_state = meta.preview_state;
    let direct_references = if metadata.state == ResourceState::Ready {
        catalog_input.direct_references().to_vec()
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
    let mut diagnostics = metadata
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.clone())
        .collect::<Vec<_>>();
    diagnostics.extend(
        catalog_input
            .reference_repairs()
            .iter()
            .map(reference_repair_diagnostic),
    );

    Some(AssetCatalogRecord {
        asset_uuid: meta.uuid,
        asset_id: metadata.id(),
        locator,
        kind: metadata.kind,
        display_name: display_name_for_path(&source_path, metadata.primary_locator()),
        file_name,
        extension,
        meta_path,
        meta,
        source_mtime_unix_ms: catalog_input.source_mtime_unix_ms(),
        source_hash: metadata.source_hash.clone(),
        preview_state,
        preview_artifact_path,
        dirty: preview_state == PreviewState::Dirty,
        diagnostics,
        direct_references,
    })
}

fn reference_repair_diagnostic(repair: &ReferenceRepair) -> String {
    format!(
        "reference path hint requires fix-up for {}: {} -> {}",
        repair.stale.guid(),
        repair.stale.path_hint(),
        repair.resolved.path_hint(),
    )
}
