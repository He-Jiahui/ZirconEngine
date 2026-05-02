use crate::ui::activity::{
    ActivityDrawerSlotPreference, ActivityViewDescriptor, ActivityWindowDescriptor,
};
use crate::ui::control::EditorUiControlService;
use zircon_runtime_interface::ui::event_ui::UiNodePath;

#[test]
fn editor_ui_control_service_registers_activity_descriptors() {
    let mut service = EditorUiControlService::default();
    service
        .register_activity_view(
            ActivityViewDescriptor::new("editor.hierarchy", "Hierarchy", "hierarchy")
                .with_multi_instance(false)
                .with_default_drawer(ActivityDrawerSlotPreference::LeftTop)
                .with_reflection_root(UiNodePath::new("editor/views/hierarchy")),
        )
        .unwrap();
    service
        .register_activity_window(
            ActivityWindowDescriptor::new("editor.prefab", "Prefab Editor", "prefab")
                .with_multi_instance(true)
                .with_supports_exclusive_page(true)
                .with_supports_floating_window(true)
                .with_reflection_root(UiNodePath::new("editor/windows/prefab")),
        )
        .unwrap();

    assert!(service.activity_view("editor.hierarchy").is_some());
    assert!(service.activity_window("editor.prefab").is_some());
}
