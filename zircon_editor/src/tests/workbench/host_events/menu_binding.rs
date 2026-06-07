use crate::core::editor_event::{MenuAction, ViewDescriptorId};
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::workbench::event::{
    dispatch_editor_host_binding, menu_action_binding, EditorHostEvent,
};
use zircon_runtime::scene::components::NodeKind;

#[test]
fn menu_action_binding_roundtrips_through_headless_dispatch() {
    let action = MenuAction::OpenView(ViewDescriptorId::new("editor.scene"));
    let binding = menu_action_binding(&action);

    assert_eq!(
        binding.native_binding(),
        r#"WorkbenchMenuBar/OpenView.editor.scene:onClick(MenuAction("workbench.view.open.editor.scene"))"#
    );
    assert_eq!(
        dispatch_editor_host_binding(&binding).unwrap(),
        EditorHostEvent::Menu(action)
    );
}

#[test]
fn debug_observatory_window_menu_binding_roundtrips_through_headless_dispatch() {
    let action = MenuAction::OpenView(ViewDescriptorId::new("editor.debug_observatory"));
    let binding = menu_action_binding(&action);

    assert_eq!(
        binding.native_binding(),
        r#"WorkbenchMenuBar/OpenView.editor.debug_observatory:onClick(MenuAction("workbench.view.open.editor.debug_observatory"))"#
    );
    assert_eq!(
        dispatch_editor_host_binding(&binding).unwrap(),
        EditorHostEvent::Menu(action)
    );
}

#[test]
fn play_mode_menu_action_bindings_roundtrip_through_headless_dispatch() {
    for (action, expected_binding) in [
        (
            MenuAction::EnterPlayMode,
            r#"WorkbenchMenuBar/EnterPlayMode:onClick(MenuAction("workbench.play_mode.enter"))"#,
        ),
        (
            MenuAction::ExitPlayMode,
            r#"WorkbenchMenuBar/ExitPlayMode:onClick(MenuAction("workbench.play_mode.exit"))"#,
        ),
    ] {
        let binding = menu_action_binding(&action);

        assert_eq!(binding.native_binding(), expected_binding);
        assert_eq!(
            dispatch_editor_host_binding(&binding).unwrap(),
            EditorHostEvent::Menu(action)
        );
    }
}

#[test]
fn dotted_menu_action_ids_roundtrip_through_headless_dispatch() {
    for (action_id, expected_action) in [
        (
            "workbench.scene.node.create.cube",
            MenuAction::CreateNode(NodeKind::Cube),
        ),
        (
            "workbench.view.open.editor.scene",
            MenuAction::OpenView(ViewDescriptorId::new("editor.scene")),
        ),
        ("CreateNode.Cube", MenuAction::CreateNode(NodeKind::Cube)),
        (
            "OpenView.editor.scene",
            MenuAction::OpenView(ViewDescriptorId::new("editor.scene")),
        ),
        (
            "menu_action.workbench.project.save",
            MenuAction::SaveProject,
        ),
        ("SaveProject", MenuAction::SaveProject),
    ] {
        let binding = EditorUiBinding::new(
            "WorkbenchMenuBar",
            action_id,
            EditorUiEventKind::Click,
            EditorUiBindingPayload::menu_action(action_id),
        );

        assert_eq!(
            dispatch_editor_host_binding(&binding).unwrap(),
            EditorHostEvent::Menu(expected_action)
        );
    }
}
