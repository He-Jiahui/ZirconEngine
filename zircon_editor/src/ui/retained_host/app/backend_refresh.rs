use zircon_runtime::asset::watch::AssetChange;
use zircon_runtime_interface::resource::{ResourceEvent, ResourceKind, ResourceLocator};

use crate::ui::host::editor_asset_manager::{EditorAssetChange, EditorAssetChangeKind};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AssetBackendRefreshPlan {
    pub sync_catalog: bool,
    pub sync_resources: bool,
    pub refresh_selected_asset_details: bool,
    pub refresh_visible_asset_previews: bool,
    pub reload_active_scene: bool,
    pub mark_render_dirty: bool,
    pub mark_presentation_dirty: bool,
    pub mark_paint_only_dirty: bool,
}

pub(crate) fn plan_asset_backend_refresh(
    active_scene_uri: Option<&str>,
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
            EditorAssetChangeKind::AssetStateChanged => {
                plan.sync_catalog = true;
                plan.mark_presentation_dirty = true;
            }
            EditorAssetChangeKind::PreviewChanged => {
                plan.sync_catalog = true;
                plan.refresh_visible_asset_previews = true;
                plan.mark_paint_only_dirty = true;
            }
            EditorAssetChangeKind::PreviewAdmissionAvailable => {
                plan.refresh_visible_asset_previews = true;
            }
            EditorAssetChangeKind::ReferenceChanged => {
                plan.sync_catalog = true;
                plan.refresh_selected_asset_details = true;
                plan.mark_presentation_dirty = true;
            }
        }
    }

    if !resource_changes.is_empty() {
        plan.sync_resources = true;
        plan.mark_render_dirty = true;
        plan.mark_presentation_dirty |= resource_changes.iter().any(|change| {
            matches!(
                change.resource_kind,
                ResourceKind::UiLayout | ResourceKind::UiWidget | ResourceKind::UiStyle
            )
        });
        plan.mark_paint_only_dirty |= resource_changes
            .iter()
            .any(|change| change.resource_kind == ResourceKind::Texture);
    }

    if let Some(active_scene_uri) = active_scene_uri {
        let active_scene_locator = ResourceLocator::parse(active_scene_uri).ok();
        let active_scene_changed = asset_changes
            .iter()
            .any(|change| active_scene_locator.as_ref() == Some(&change.uri))
            || resource_changes.iter().any(|change| {
                change
                    .locator
                    .as_ref()
                    .is_some_and(|locator| active_scene_locator.as_ref() == Some(locator))
                    || change
                        .previous_locator
                        .as_ref()
                        .is_some_and(|locator| active_scene_locator.as_ref() == Some(locator))
            });
        if active_scene_changed {
            plan.reload_active_scene = true;
            plan.mark_render_dirty = true;
            plan.mark_presentation_dirty = true;
        }
    }

    plan
}

#[cfg(test)]
#[path = "tests/backend_refresh_performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "backend_refresh/tests/optimization_tests.rs"]
mod optimization_tests;
