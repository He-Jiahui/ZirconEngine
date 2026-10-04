use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiPoint};

use super::UiSurfaceInputState;

#[test]
fn closing_parent_popup_also_closes_nested_popup_tail() {
    let mut input = UiSurfaceInputState::default();
    input.open_popup(
        "menu.file".to_string(),
        Some(UiNodeId::new(1)),
        Some(UiPoint::new(8.0, 12.0)),
    );
    input.open_popup(
        "menu.file.recent".to_string(),
        Some(UiNodeId::new(2)),
        Some(UiPoint::new(24.0, 12.0)),
    );
    input.open_popup(
        "menu.file.recent.project".to_string(),
        Some(UiNodeId::new(3)),
        Some(UiPoint::new(40.0, 12.0)),
    );

    assert!(input.close_popup("menu.file.recent"));
    assert_eq!(
        input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str())
            .collect::<Vec<_>>(),
        vec!["menu.file"]
    );
}

#[test]
fn closing_unknown_popup_preserves_stack() {
    let mut input = UiSurfaceInputState::default();
    input.open_popup("menu.file".to_string(), Some(UiNodeId::new(1)), None);

    assert!(!input.close_popup("menu.edit"));
    assert_eq!(input.popup_stack.len(), 1);
}
