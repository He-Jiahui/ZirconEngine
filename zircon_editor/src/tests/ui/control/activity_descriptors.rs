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

#[test]
fn activity_drawer_slot_preference_exposes_single_bottom_dock_and_migrates_legacy_bottom_slots() {
    let descriptor = ActivityViewDescriptor::new("editor.console", "Console", "terminal")
        .with_default_drawer(ActivityDrawerSlotPreference::Bottom);

    assert_eq!(
        serde_json::to_string(&descriptor.default_drawer).unwrap(),
        r#""Bottom""#,
        "external activity descriptors should expose one Bottom drawer position"
    );

    for legacy in [r#""BottomLeft""#, r#""BottomRight""#] {
        let decoded: ActivityDrawerSlotPreference = serde_json::from_str(legacy).unwrap();
        assert_eq!(
            decoded,
            ActivityDrawerSlotPreference::Bottom,
            "legacy serialized bottom drawer slots should migrate to Bottom"
        );
    }
}
