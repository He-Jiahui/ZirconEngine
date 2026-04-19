use zircon_asset::editor::{EditorAssetChange, EditorAssetChangeKind};
use zircon_asset::watch::AssetChange;
use zircon_resource::ResourceEvent;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AssetBackendRefreshPlan {
    pub sync_catalog: bool,
    pub sync_resources: bool,
    pub refresh_selected_asset_details: bool,
    pub refresh_visible_asset_previews: bool,
    pub reload_default_scene: bool,
    pub mark_render_dirty: bool,
    pub mark_presentation_dirty: bool,
}

pub(crate) fn plan_asset_backend_refresh(
    selected_asset_uuid: Option<&str>,
    default_scene_uri: Option<&str>,
    asset_changes: &[AssetChange],
    editor_changes: &[EditorAssetChange],
    resource_changes: &[ResourceEvent],
) -> AssetBackendRefreshPlan {
    let mut plan = AssetBackendRefreshPlan::default();

    for change in editor_changes {
        match change.kind {
            EditorAssetChangeKind::CatalogChanged => {
                plan.sync_catalog = true;
                plan.refresh_selected_asset_details = true;
                plan.refresh_visible_asset_previews = true;
                plan.mark_presentation_dirty = true;
            }
            EditorAssetChangeKind::PreviewChanged => {
                plan.sync_catalog = true;
                plan.mark_presentation_dirty = true;
            }
            EditorAssetChangeKind::ReferenceChanged => {
                plan.sync_catalog = true;
                plan.refresh_selected_asset_details = true;
                plan.mark_presentation_dirty = true;
            }
        }

        if change.uuid.as_deref() == selected_asset_uuid
            && matches!(
                change.kind,
                EditorAssetChangeKind::CatalogChanged | EditorAssetChangeKind::ReferenceChanged
            )
        {
            plan.refresh_selected_asset_details = true;
        }
    }

    if !resource_changes.is_empty() {
        plan.sync_resources = true;
        plan.mark_render_dirty = true;
        plan.mark_presentation_dirty = true;
    }

    if let Some(default_scene_uri) = default_scene_uri {
        let default_scene_changed = asset_changes
            .iter()
            .any(|change| change.uri.to_string() == default_scene_uri)
            || resource_changes.iter().any(|change| {
                change
                    .locator
                    .as_ref()
                    .is_some_and(|locator| locator.to_string() == default_scene_uri)
                    || change
                        .previous_locator
                        .as_ref()
                        .is_some_and(|locator| locator.to_string() == default_scene_uri)
            });
        if default_scene_changed {
            plan.reload_default_scene = true;
            plan.mark_render_dirty = true;
            plan.mark_presentation_dirty = true;
        }
    }

    plan
}
