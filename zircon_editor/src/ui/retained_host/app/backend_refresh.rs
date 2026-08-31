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
    selected_asset_uuid: Option<&str>,
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
mod performance_tests {
    use crate::ui::host::editor_asset_manager::{EditorAssetChangeKind, EditorAssetChangeRecord};
    use zircon_runtime_interface::resource::{
        ResourceEvent, ResourceEventKind, ResourceId, ResourceKind, ResourceLocator,
    };

    use super::plan_asset_backend_refresh;

    #[test]
    fn active_scene_refresh_parses_the_locator_once() {
        let source = include_str!("backend_refresh.rs");
        let production = source.split("#[cfg(test)]").next().expect("implementation");
        let formatting_comparison = [".to_", "string() == active_scene_uri"].concat();

        assert!(!production.contains(&formatting_comparison));
        assert_eq!(
            production
                .matches("ResourceLocator::parse(active_scene_uri)")
                .count(),
            1
        );
    }

    #[test]
    fn preview_completion_refills_bounded_visible_preview_admission() {
        let plan = plan_asset_backend_refresh(
            None,
            None,
            &[],
            &[EditorAssetChangeRecord {
                kind: EditorAssetChangeKind::PreviewChanged,
                catalog_revision: 7,
                uuid: Some("asset-a".to_string()),
                locator: Some("res://asset-a.png".to_string()),
            }],
            &[],
        );

        assert!(plan.sync_catalog);
        assert!(plan.refresh_visible_asset_previews);
        assert!(plan.mark_paint_only_dirty);
        assert!(!plan.mark_presentation_dirty);
    }

    #[test]
    fn retry_or_cancel_admission_release_refills_without_catalog_sync() {
        let plan = plan_asset_backend_refresh(
            None,
            None,
            &[],
            &[EditorAssetChangeRecord {
                kind: EditorAssetChangeKind::PreviewAdmissionAvailable,
                catalog_revision: 7,
                uuid: Some("asset-a".to_string()),
                locator: Some("res://asset-a.png".to_string()),
            }],
            &[],
        );

        assert!(plan.refresh_visible_asset_previews);
        assert!(!plan.sync_catalog);
        assert!(!plan.mark_paint_only_dirty);
        assert!(!plan.mark_presentation_dirty);
    }

    #[test]
    fn non_ui_resource_churn_does_not_rebuild_editor_presentation() {
        let plan = plan_asset_backend_refresh(
            None,
            None,
            &[],
            &[],
            &[resource_event(ResourceKind::Mesh, "res://models/cube.mesh")],
        );

        assert!(plan.sync_resources);
        assert!(plan.mark_render_dirty);
        assert!(!plan.mark_presentation_dirty);
        assert!(!plan.mark_paint_only_dirty);
    }

    #[test]
    fn texture_resource_change_repaints_without_rebuilding_presentation() {
        let plan = plan_asset_backend_refresh(
            None,
            None,
            &[],
            &[],
            &[resource_event(
                ResourceKind::Texture,
                "res://icons/save.png",
            )],
        );

        assert!(plan.mark_render_dirty);
        assert!(plan.mark_paint_only_dirty);
        assert!(!plan.mark_presentation_dirty);
    }

    #[test]
    fn ui_resource_change_keeps_the_structural_rebuild_fallback() {
        let plan = plan_asset_backend_refresh(
            None,
            None,
            &[],
            &[],
            &[resource_event(
                ResourceKind::UiLayout,
                "res://ui/workbench.ui",
            )],
        );

        assert!(plan.mark_presentation_dirty);
    }

    fn resource_event(resource_kind: ResourceKind, locator: &str) -> ResourceEvent {
        let locator = ResourceLocator::parse(locator).expect("resource locator");
        ResourceEvent {
            kind: ResourceEventKind::Updated,
            resource_kind,
            id: ResourceId::from_locator(&locator),
            locator: Some(locator),
            previous_locator: None,
            revision: 1,
        }
    }
}
