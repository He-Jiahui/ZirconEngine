use crate::ui::host::editor_asset_manager::{EditorAssetChangeKind, EditorAssetChangeRecord};
use zircon_runtime_interface::resource::{
    ResourceEvent, ResourceEventKind, ResourceId, ResourceKind, ResourceLocator,
};

use super::plan_asset_backend_refresh;

#[test]
fn active_scene_refresh_parses_the_locator_once() {
    let source = include_str!("../backend_refresh.rs");
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
