use super::*;

fn focus() -> HostTextInputFocusData {
    HostTextInputFocusData {
        control_id: "Name".into(),
        value_text: "A🙂界B".into(),
        ..HostTextInputFocusData::default()
    }
}

#[test]
fn private_text_selection_replaces_unicode_scalars_and_preserves_identity() {
    let mut focus = focus();
    focus.move_text_caret("all", false);
    assert_eq!(focus.selected_scalar_range(), Some((0, 4)));
    assert!(focus.insert_text("é🙂"));
    assert_eq!(focus.value_text.as_str(), "é🙂");
    assert_eq!(focus.resolved_caret_scalar_offset(), 2);
    assert_eq!(focus.selected_scalar_range(), None);
    assert_eq!(focus.control_id.as_str(), "Name");
}

#[test]
fn private_text_shift_navigation_and_deletion_are_scalar_safe() {
    let mut focus = focus();
    focus.move_text_caret("left", true);
    focus.move_text_caret("left", true);
    assert_eq!(focus.selected_scalar_range(), Some((2, 4)));
    focus.move_text_caret("left", false);
    assert_eq!(focus.resolved_caret_scalar_offset(), 2);
    assert_eq!(focus.selected_scalar_range(), None);
    assert!(focus.delete_text(true));
    assert_eq!(focus.value_text.as_str(), "A界B");
    assert!(focus.delete_text(false));
    assert_eq!(focus.value_text.as_str(), "AB");
    focus.move_text_caret("home", false);
    assert!(!focus.delete_text(true));
    focus.move_text_caret("end", true);
    assert_eq!(focus.selected_scalar_range(), Some((0, 2)));
    assert!(focus.delete_text(false));
    assert!(focus.value_text.is_empty());
}

#[test]
fn private_text_offsets_clamp_after_value_refresh_and_control_text_is_ignored() {
    let mut focus = focus();
    focus.caret_scalar_offset = Some(99);
    focus.selection_anchor_scalar_offset = Some(99);
    assert_eq!(focus.resolved_caret_scalar_offset(), 4);
    assert_eq!(focus.selected_scalar_range(), None);
    assert!(!focus.insert_text("\n\t"));
    focus.value_text = "é".into();
    assert_eq!(focus.resolved_caret_scalar_offset(), 1);
    assert!(focus.delete_text(true));
    assert!(focus.value_text.is_empty());
}
