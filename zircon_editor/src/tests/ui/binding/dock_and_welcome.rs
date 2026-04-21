use crate::ui::binding::{
    DockCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind, WelcomeCommand,
};

#[test]
fn dock_command_binding_roundtrips_through_native_binding() {
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
        binding.native_binding(),
        r#"HierarchyView/AutoHideDrawer:onClick(DockCommand.SetDrawerMode("left_top","AutoHide"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn activity_rail_toggle_binding_roundtrips_through_native_binding() {
    let binding = EditorUiBinding::new(
        "ActivityRail",
        "ProjectToggle",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::dock_command(DockCommand::ActivateDrawerTab {
            slot: "left_top".to_string(),
            instance_id: "editor.project#1".to_string(),
        }),
    );

    assert_eq!(
        binding.native_binding(),
        r#"ActivityRail/ProjectToggle:onClick(DockCommand.ActivateDrawerTab("left_top","editor.project#1"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn dock_preset_command_bindings_roundtrip_through_native_binding() {
    let save_binding = EditorUiBinding::new(
        "ToolWindow",
        "SavePreset",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::dock_command(DockCommand::SavePreset {
            name: "rider".to_string(),
        }),
    );

    assert_eq!(
        save_binding.native_binding(),
        r#"ToolWindow/SavePreset:onClick(DockCommand.SavePreset("rider"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&save_binding.native_binding()).unwrap(),
        save_binding
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
        load_binding.native_binding(),
        r#"ToolWindow/LoadPreset:onClick(DockCommand.LoadPreset("rider"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&load_binding.native_binding()).unwrap(),
        load_binding
    );
}

#[test]
fn dock_attach_command_bindings_roundtrip_through_native_binding() {
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
        drawer_binding.native_binding(),
        r#"ToolWindow/DropToRight:onClick(DockCommand.AttachViewToDrawer("editor.project#1","right_top"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&drawer_binding.native_binding()).unwrap(),
        drawer_binding
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
        document_binding.native_binding(),
        r#"DocumentTabs/DropToDocument:onClick(DockCommand.AttachViewToDocument("editor.project#1","workbench"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&document_binding.native_binding()).unwrap(),
        document_binding
    );
}

#[test]
fn welcome_command_binding_roundtrips_through_native_binding() {
    let edit_binding = EditorUiBinding::new(
        "WelcomeSurface",
        "ProjectNameEdited",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::SetProjectName {
            value: "Sandbox".to_string(),
        }),
    );

    assert_eq!(
        edit_binding.native_binding(),
        r#"WelcomeSurface/ProjectNameEdited:onChange(WelcomeCommand.SetProjectName("Sandbox"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&edit_binding.native_binding()).unwrap(),
        edit_binding
    );

    let open_recent_binding = EditorUiBinding::new(
        "WelcomeSurface",
        "OpenRecentProject",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::OpenRecentProject {
            path: "E:/Projects/Sandbox".to_string(),
        }),
    );

    assert_eq!(
        open_recent_binding.native_binding(),
        r#"WelcomeSurface/OpenRecentProject:onClick(WelcomeCommand.OpenRecentProject("E:/Projects/Sandbox"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&open_recent_binding.native_binding()).unwrap(),
        open_recent_binding
    );
}
