use super::support;
use crate::core::editor_event::{
    ActivityDrawerMode, ActivityDrawerSlot, LayoutCommand, MainPageId, ViewHost, ViewInstanceId,
};
use crate::ui::binding::{DockCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::binding_dispatch::dispatch_docking_binding;

#[test]
fn docking_binding_dispatches_into_layout_command() {
    let binding = EditorUiBinding::new(
        "HierarchyView",
        "AutoHideDrawer",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::dock_command(DockCommand::SetDrawerMode {
            slot: "left_top".to_string(),
            mode: "AutoHide".to_string(),
        }),
    );

    assert_eq!(
        dispatch_docking_binding(&binding).unwrap(),
        LayoutCommand::SetDrawerMode {
            slot: ActivityDrawerSlot::LeftTop,
            mode: ActivityDrawerMode::AutoHide,
        }
    );
}

#[test]
fn docking_preset_binding_dispatches_into_layout_command() {
    let save_binding = EditorUiBinding::new(
        "ToolWindow",
        "SavePreset",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::dock_command(DockCommand::SavePreset {
            name: "rider".to_string(),
        }),
    );

    assert_eq!(
        dispatch_docking_binding(&save_binding).unwrap(),
        LayoutCommand::SavePreset {
            name: "rider".to_string(),
        }
    );

    let load_binding = EditorUiBinding::new(
        "ToolWindow",
        "LoadPreset",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::dock_command(DockCommand::LoadPreset {
            name: "rider".to_string(),
        }),
    );

    assert_eq!(
        dispatch_docking_binding(&load_binding).unwrap(),
        LayoutCommand::LoadPreset {
            name: "rider".to_string(),
        }
    );
}

#[test]
fn docking_attach_binding_dispatches_into_layout_command() {
    let drawer_binding = EditorUiBinding::new(
        "ToolWindow",
        "DropToRight",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::dock_command(DockCommand::AttachViewToDrawer {
            instance_id: "editor.project#1".to_string(),
            slot: "right_top".to_string(),
        }),
    );

    assert_eq!(
        dispatch_docking_binding(&drawer_binding).unwrap(),
        LayoutCommand::AttachView {
            instance_id: ViewInstanceId::new("editor.project#1"),
            target: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
            anchor: None,
        }
    );

    let document_binding = EditorUiBinding::new(
        "DocumentTabs",
        "DropToDocument",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::dock_command(DockCommand::AttachViewToDocument {
            instance_id: "editor.project#1".to_string(),
            page_id: "workbench".to_string(),
        }),
    );

    assert_eq!(
        dispatch_docking_binding(&document_binding).unwrap(),
        LayoutCommand::AttachView {
            instance_id: ViewInstanceId::new("editor.project#1"),
            target: ViewHost::Document(MainPageId::workbench(), Vec::new()),
            anchor: None,
        }
    );
}
