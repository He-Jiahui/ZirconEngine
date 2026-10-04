use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

use super::*;

#[test]
fn source_search_selection_and_rename_keep_distinct_state_domains() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("workbench bridge should build");

    assert!(bridge.control_bool("WorkbenchTagsSourceRow", "selected"));
    assert!(bridge.control_bool("WorkbenchTagsAbilityActivateRow", "selected"));

    bridge
        .dispatch_control_state("WorkbenchTagsStateStunnedRow", UiEventKind::Click)
        .expect("stunned tag should dispatch")
        .expect("stunned tag should bind");
    assert!(bridge.control_bool("WorkbenchTagsSourceRow", "selected"));
    assert!(bridge.control_bool("WorkbenchTagsStateStunnedRow", "selected"));
    assert_eq!(
        Some("Character.State.Stun".to_string()),
        bridge.control_string("WorkbenchTagsRedirectField", "value")
    );
    for (control_id, value) in [
        ("WorkbenchTagsRedirectField", "Character.State.Disabled"),
        ("WorkbenchTagsOwnerField", "CustomGameplayTags.ini"),
    ] {
        bridge
            .mutate_control_property(control_id, "value", UiValue::String(value.to_string()))
            .expect("tag property should edit");
    }

    bridge
        .dispatch_control_state("WorkbenchTagsRenameButton", UiEventKind::Click)
        .expect("rename should dispatch")
        .expect("rename should bind");
    assert_eq!(
        Some("Rename ready   CustomGameplayTags.ini".to_string()),
        bridge.control_string("WorkbenchTagsValidationRow", "value_text")
    );
    assert_eq!(
        Some("Tag Registry / Character.State.Disabled".to_string()),
        bridge.control_string("WorkbenchTagsCenterTitle", "text")
    );

    bridge
        .mutate_control_property(
            TAGS_SEARCH_CONTROL,
            "value",
            UiValue::String("ability".to_string()),
        )
        .expect("tag search value should update");
    bridge
        .dispatch_control_state(TAGS_SEARCH_CONTROL, UiEventKind::Change)
        .expect("tag search should dispatch")
        .expect("tag search should bind");
    assert!(bridge
        .control_frame("WorkbenchTagsAbilityActivateRow")
        .is_some());
    assert!(bridge
        .control_frame("WorkbenchTagsStateStunnedRow")
        .is_none());
    assert!(bridge.control_bool("WorkbenchTagsAbilityActivateRow", "selected"));
}
