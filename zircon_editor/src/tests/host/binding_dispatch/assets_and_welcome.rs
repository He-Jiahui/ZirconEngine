use crate::core::editor_event::{EditorAssetSurface, EditorAssetUtilityTab, EditorAssetViewMode};
use crate::ui::binding::{
    AssetCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind, WelcomeCommand,
};
use crate::ui::binding_dispatch::{
    dispatch_asset_binding, dispatch_welcome_binding, AssetHostEvent, EditorBindingDispatchError,
    WelcomeHostEvent,
};

#[test]
fn asset_binding_dispatches_into_host_event() {
    let binding = EditorUiBinding::new(
        "ProjectView",
        "OpenAsset",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::asset_command(AssetCommand::OpenAsset {
            asset_locator: "crate://prefabs/player.prefab".to_string(),
        }),
    );

    assert_eq!(
        dispatch_asset_binding(&binding).unwrap(),
        AssetHostEvent::OpenAsset {
            asset_locator: "crate://prefabs/player.prefab".to_string(),
        }
    );
}

#[test]
fn asset_view_mode_binding_dispatches_into_typed_host_event() {
    let binding = EditorUiBinding::new(
        "AssetSurface",
        "SetViewMode",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::asset_command(AssetCommand::SetViewMode {
            surface: "browser".to_string(),
            view_mode: "thumbnail".to_string(),
        }),
    );

    assert_eq!(
        dispatch_asset_binding(&binding).unwrap(),
        AssetHostEvent::SetViewMode {
            surface: EditorAssetSurface::Browser,
            view_mode: EditorAssetViewMode::Thumbnail,
        }
    );
}

#[test]
fn asset_utility_tab_binding_dispatches_into_typed_host_event() {
    let binding = EditorUiBinding::new(
        "AssetSurface",
        "SetUtilityTab",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::asset_command(AssetCommand::SetUtilityTab {
            surface: "browser".to_string(),
            tab: "metadata".to_string(),
        }),
    );

    assert_eq!(
        dispatch_asset_binding(&binding).unwrap(),
        AssetHostEvent::SetUtilityTab {
            surface: EditorAssetSurface::Browser,
            tab: EditorAssetUtilityTab::Metadata,
        }
    );
}

#[test]
fn asset_relocation_binding_preserves_uuid_and_target_locator() {
    let binding = EditorUiBinding::new(
        "AssetTree",
        "RelocateAsset",
        EditorUiEventKind::Drop,
        EditorUiBindingPayload::asset_command(AssetCommand::RelocateAsset {
            asset_uuid: "00112233-4455-6677-8899-aabbccddeeff".to_string(),
            target_locator: "res://environment/cube.zmodel".to_string(),
        }),
    );

    assert_eq!(
        dispatch_asset_binding(&binding).unwrap(),
        AssetHostEvent::RelocateAsset {
            asset_uuid: "00112233-4455-6677-8899-aabbccddeeff".to_string(),
            target_locator: "res://environment/cube.zmodel".to_string(),
        }
    );
}

#[test]
fn asset_deletion_binding_preserves_the_target_uuid() {
    let binding = EditorUiBinding::new(
        "AssetContextMenu",
        "DeleteAsset",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::asset_command(AssetCommand::DeleteAsset {
            asset_uuid: "00112233-4455-6677-8899-aabbccddeeff".to_string(),
        }),
    );

    assert_eq!(
        dispatch_asset_binding(&binding).unwrap(),
        AssetHostEvent::DeleteAsset {
            asset_uuid: "00112233-4455-6677-8899-aabbccddeeff".to_string(),
        }
    );
}

#[test]
fn asset_binding_rejects_unknown_typed_tokens_without_state_mutation_errors() {
    let unknown_surface = EditorUiBinding::new(
        "AssetSurface",
        "SetViewMode",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::asset_command(AssetCommand::SetViewMode {
            surface: "library".to_string(),
            view_mode: "list".to_string(),
        }),
    );
    let unknown_view_mode = EditorUiBinding::new(
        "AssetSurface",
        "SetViewMode",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::asset_command(AssetCommand::SetViewMode {
            surface: "browser".to_string(),
            view_mode: "grid".to_string(),
        }),
    );
    let unknown_utility_tab = EditorUiBinding::new(
        "AssetSurface",
        "SetUtilityTab",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::asset_command(AssetCommand::SetUtilityTab {
            surface: "browser".to_string(),
            tab: "summary".to_string(),
        }),
    );

    assert!(matches!(
        dispatch_asset_binding(&unknown_surface),
        Err(EditorBindingDispatchError::UnknownAssetSurface(surface)) if surface == "library"
    ));
    assert!(matches!(
        dispatch_asset_binding(&unknown_view_mode),
        Err(EditorBindingDispatchError::UnknownAssetViewMode(mode)) if mode == "grid"
    ));
    assert!(matches!(
        dispatch_asset_binding(&unknown_utility_tab),
        Err(EditorBindingDispatchError::UnknownAssetUtilityTab(tab)) if tab == "summary"
    ));
}

#[test]
fn welcome_project_name_binding_dispatches_into_typed_host_event() {
    let binding = EditorUiBinding::new(
        "WelcomeSurface",
        "ProjectNameEdited",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::SetProjectName {
            value: "Sandbox".to_string(),
        }),
    );

    assert_eq!(
        dispatch_welcome_binding(&binding).unwrap(),
        WelcomeHostEvent::SetProjectName {
            value: "Sandbox".to_string(),
        }
    );
}

#[test]
fn welcome_open_recent_binding_dispatches_into_typed_host_event() {
    let binding = EditorUiBinding::new(
        "WelcomeSurface",
        "OpenRecentProject",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::OpenRecentProject {
            path: "E:/Projects/Sandbox".to_string(),
        }),
    );

    assert_eq!(
        dispatch_welcome_binding(&binding).unwrap(),
        WelcomeHostEvent::OpenRecentProject {
            path: "E:/Projects/Sandbox".to_string(),
        }
    );
}

#[test]
fn welcome_recover_recent_binding_dispatches_into_typed_host_event() {
    let binding = EditorUiBinding::new(
        "WelcomeSurface",
        "RecoverRecentProject",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::RecoverRecentProject {
            path: "E:/Projects/Sandbox".to_string(),
        }),
    );

    assert_eq!(
        dispatch_welcome_binding(&binding).unwrap(),
        WelcomeHostEvent::RecoverRecentProject {
            path: "E:/Projects/Sandbox".to_string(),
        }
    );
}

#[test]
fn welcome_safe_recent_binding_dispatches_into_typed_host_event() {
    let binding = EditorUiBinding::new(
        "WelcomeSurface",
        "SafeRecentProject",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::SafeRecentProject {
            path: "E:/Projects/Sandbox".to_string(),
        }),
    );

    assert_eq!(
        dispatch_welcome_binding(&binding).unwrap(),
        WelcomeHostEvent::SafeRecentProject {
            path: "E:/Projects/Sandbox".to_string(),
        }
    );
}

#[test]
fn welcome_startup_chooser_bindings_dispatch_into_typed_host_events() {
    let workbench = EditorUiBinding::new(
        "WelcomeSurface",
        "OpenStartupWorkbench",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::OpenStartupWorkbench),
    );
    let demo = EditorUiBinding::new(
        "WelcomeSurface",
        "OpenStartupDemo",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::OpenStartupDemo),
    );
    let asset = EditorUiBinding::new(
        "WelcomeSurface",
        "OpenStartupAssetWindow",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::OpenStartupAssetWindow),
    );
    let ui_layout = EditorUiBinding::new(
        "WelcomeSurface",
        "OpenStartupUILayoutEditor",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::OpenStartupUILayoutEditor),
    );

    assert_eq!(
        dispatch_welcome_binding(&workbench).unwrap(),
        WelcomeHostEvent::OpenStartupWorkbench
    );
    assert_eq!(
        dispatch_welcome_binding(&demo).unwrap(),
        WelcomeHostEvent::OpenStartupDemo
    );
    assert_eq!(
        dispatch_welcome_binding(&asset).unwrap(),
        WelcomeHostEvent::OpenStartupAssetWindow
    );
    assert_eq!(
        dispatch_welcome_binding(&ui_layout).unwrap(),
        WelcomeHostEvent::OpenStartupUILayoutEditor
    );
}
