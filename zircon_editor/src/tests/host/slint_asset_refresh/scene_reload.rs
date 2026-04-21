use crate::ui::slint_host::{plan_asset_backend_refresh, AssetBackendRefreshPlan};
use zircon_runtime::asset::watch::{AssetChange, AssetChangeKind};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::resource::{
    ResourceEvent, ResourceEventKind, ResourceId, ResourceLocator,
};

#[test]
fn default_scene_resource_change_requests_reload_and_runtime_sync() {
    let plan = plan_asset_backend_refresh(
        None,
        Some("res://scenes/main.scene.toml"),
        &[AssetChange {
            kind: AssetChangeKind::Modified,
            uri: AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            previous_uri: None,
        }],
        &[],
        &[ResourceEvent {
            kind: ResourceEventKind::Updated,
            id: ResourceId::from_locator(
                &ResourceLocator::parse("res://scenes/main.scene.toml").unwrap(),
            ),
            locator: Some(ResourceLocator::parse("res://scenes/main.scene.toml").unwrap()),
            previous_locator: None,
            revision: 9,
        }],
    );

    assert_eq!(
        plan,
        AssetBackendRefreshPlan {
            sync_resources: true,
            reload_default_scene: true,
            mark_render_dirty: true,
            mark_presentation_dirty: true,
            ..AssetBackendRefreshPlan::default()
        }
    );
}
