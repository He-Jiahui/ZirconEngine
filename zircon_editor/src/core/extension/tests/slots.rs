use super::{DefaultWorkbenchPreset, WorkbenchSlot};

#[test]
fn drawer_classification_is_exhaustive_and_document_center_is_default() {
    assert_eq!(WorkbenchSlot::default(), WorkbenchSlot::DocumentCenter);
    for slot in [
        WorkbenchSlot::LeftTopDrawer,
        WorkbenchSlot::LeftBottomDrawer,
        WorkbenchSlot::RightTopDrawer,
        WorkbenchSlot::RightBottomDrawer,
        WorkbenchSlot::BottomDrawer,
    ] {
        assert!(slot.is_drawer());
    }
    for slot in [
        WorkbenchSlot::DocumentCenter,
        WorkbenchSlot::FloatingWindow,
        WorkbenchSlot::ExclusiveMainPage,
    ] {
        assert!(!slot.is_drawer());
    }
}

#[test]
fn default_preset_names_are_finite_and_stably_ordered() {
    assert!(DefaultWorkbenchPreset::Authoring < DefaultWorkbenchPreset::Review);
    assert!(DefaultWorkbenchPreset::Review < DefaultWorkbenchPreset::Focus);
    assert!(DefaultWorkbenchPreset::Focus < DefaultWorkbenchPreset::Debug);
}

#[test]
fn default_preset_declarations_are_normalized_by_the_finite_owner() {
    assert_eq!(
        DefaultWorkbenchPreset::normalize([
            DefaultWorkbenchPreset::Debug,
            DefaultWorkbenchPreset::Authoring,
            DefaultWorkbenchPreset::Focus,
            DefaultWorkbenchPreset::Debug,
        ]),
        vec![
            DefaultWorkbenchPreset::Authoring,
            DefaultWorkbenchPreset::Focus,
            DefaultWorkbenchPreset::Debug,
        ]
    );
}
