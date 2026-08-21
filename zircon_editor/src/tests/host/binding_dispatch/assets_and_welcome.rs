use crate::core::editor_event::{EditorAssetSurface, EditorAssetUtilityTab, EditorAssetViewMode};
use crate::ui::binding::{
    AssetCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind, WelcomeCommand,
};
use crate::ui::binding_dispatch::{
    dispatch_asset_binding, dispatch_welcome_binding, AssetHostEvent, WelcomeHostEvent,
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
