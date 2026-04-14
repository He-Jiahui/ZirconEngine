use crate::{
    dispatch_workbench_binding, menu_action_binding, MenuAction, ViewDescriptorId,
    WorkbenchHostEvent,
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
        dispatch_workbench_binding(&binding).unwrap(),
        WorkbenchHostEvent::Menu(action)
    );
}
