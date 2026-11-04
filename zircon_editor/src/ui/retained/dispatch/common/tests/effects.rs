use super::merge_effects;
use crate::ui::retained_host::event_bridge::{
    AssetDeletionRequest, AssetFolderCreateRequest, AssetFolderRequest, AssetIdentityRequest,
    AssetRelocationRequest, HostShellContentScope, UiHostEventEffects,
};
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::ViewInstanceId;

#[test]
fn u09_merged_context_asset_requests_keep_payloads_until_host_dispatch() {
    let mut requests = UiHostEventEffects::default();
    requests.asset_deletion_requested = Some(AssetDeletionRequest {
        asset_uuid: "asset-to-delete".to_owned(),
    });
    requests.asset_relocation_requested = Some(AssetRelocationRequest {
        asset_uuid: "asset-to-move".to_owned(),
        target_locator: "res://levels/moved.scene.toml".to_owned(),
    });
    requests.asset_folder_create_requested = Some(AssetFolderCreateRequest {
        parent_folder_id: "res://".to_owned(),
    });
    requests.asset_folder_rename_requested = Some(AssetFolderRequest {
        folder_id: "res://levels".to_owned(),
    });
    requests.asset_folder_deletion_requested = Some(AssetFolderRequest {
        folder_id: "res://empty".to_owned(),
    });
    requests.asset_rename_requested = Some(AssetIdentityRequest {
        asset_uuid: "asset-to-rename".to_owned(),
    });
    requests.asset_reimport_requested = Some(AssetIdentityRequest {
        asset_uuid: "asset-to-reimport".to_owned(),
    });
    let mut effects = UiHostEventEffects::default();

    merge_effects(&mut effects, requests.clone());
    assert_eq!(effects, requests);
    assert!(!effects.is_viewport_resize_recompute_compatible());

    // Later paint or popup effects must not erase an already queued host action.
    merge_effects(&mut effects, UiHostEventEffects::default());
    assert_eq!(effects, requests);
}

#[test]
fn merged_drawer_toggles_preserve_parity_and_require_host_dispatch() {
    let mut target = UiHostEventEffects::default();
    let mut toggle = UiHostEventEffects::default();
    toggle.toggle_content_drawer_requested = true;
    toggle.request_paint_only();
    merge_effects(&mut target, toggle.clone());
    assert!(target.toggle_content_drawer_requested);
    assert!(!target.is_viewport_resize_recompute_compatible());
    merge_effects(&mut target, toggle);
    assert!(!target.toggle_content_drawer_requested);
}

#[test]
fn merging_opened_view_visibility_keeps_each_actual_instance_request() {
    let mut target = UiHostEventEffects::default();
    target
        .opened_view_visibility_requests
        .push(ViewInstanceId::new("editor.inspector#1"));
    let mut source = UiHostEventEffects::default();
    source
        .opened_view_visibility_requests
        .push(ViewInstanceId::new("editor.hierarchy#1"));
    merge_effects(&mut target, source);
    assert_eq!(
        target.opened_view_visibility_requests,
        vec![
            ViewInstanceId::new("editor.inspector#1"),
            ViewInstanceId::new("editor.hierarchy#1"),
        ]
    );
    assert!(!target.is_viewport_resize_recompute_compatible());
}

#[test]
fn merging_distinct_shell_content_scopes_disables_the_single_target_scope() {
    let mut first = UiHostEventEffects::default();
    first.reuse_layout_for_shell_content(HostShellContentScope::new(
        ActivityDrawerSlot::LeftTop,
        ViewInstanceId::new("editor.hierarchy#main"),
    ));
    let mut second = UiHostEventEffects::default();
    second.reuse_layout_for_shell_content(HostShellContentScope::new(
        ActivityDrawerSlot::RightTop,
        ViewInstanceId::new("editor.inspector#main"),
    ));

    merge_effects(&mut first, second);

    assert_eq!(first.shell_content_scope(), None);
}

#[test]
fn merging_unscoped_presentation_disables_the_shell_content_scope() {
    let mut scoped = UiHostEventEffects::default();
    scoped.reuse_layout_for_shell_content(HostShellContentScope::new(
        ActivityDrawerSlot::LeftTop,
        ViewInstanceId::new("editor.hierarchy#main"),
    ));
    let mut global = UiHostEventEffects::default();
    global.request_presentation();

    merge_effects(&mut scoped, global);

    assert_eq!(scoped.shell_content_scope(), None);
}

#[test]
fn merging_paint_only_preserves_the_shell_content_scope() {
    let scope = HostShellContentScope::new(
        ActivityDrawerSlot::LeftTop,
        ViewInstanceId::new("editor.hierarchy#main"),
    );
    let mut scoped = UiHostEventEffects::default();
    scoped.reuse_layout_for_shell_content(scope.clone());
    let mut paint_only = UiHostEventEffects::default();
    paint_only.request_paint_only();

    merge_effects(&mut scoped, paint_only);

    assert_eq!(scoped.shell_content_scope(), Some(scope));
}
