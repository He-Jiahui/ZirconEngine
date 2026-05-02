use crate::core::editor_event::{MenuAction, ViewDescriptorId};
use crate::ui::workbench::event::{
    dispatch_editor_host_binding, menu_action_binding, EditorHostEvent,
};

#[test]
fn menu_action_binding_roundtrips_through_headless_dispatch() {
    let action = MenuAction::OpenView(ViewDescriptorId::new("editor.scene"));
    let binding = menu_action_binding(&action);

    assert_eq!(
        binding.native_binding(),
        r#"WorkbenchMenuBar/OpenView.editor.scene:onClick(MenuAction("OpenView.editor.scene"))"#
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
            r#"WorkbenchMenuBar/EnterPlayMode:onClick(MenuAction("EnterPlayMode"))"#,
        ),
        (
            MenuAction::ExitPlayMode,
            r#"WorkbenchMenuBar/ExitPlayMode:onClick(MenuAction("ExitPlayMode"))"#,
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
