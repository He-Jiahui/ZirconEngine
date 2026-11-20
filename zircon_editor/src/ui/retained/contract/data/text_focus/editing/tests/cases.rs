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

fn focus_with_text(text: &str) -> HostTextInputFocusData {
    HostTextInputFocusData {
        value_text: text.into(),
        ..focus()
    }
}

#[test]
fn private_text_arrows_and_shift_keep_combining_grapheme_atomic() {
    let mut focus = focus_with_text("Ae\u{0301}B");
    assert!(focus.move_text_caret("left", true));
    assert_eq!(focus.selected_scalar_range(), Some((3, 4)));
    assert!(focus.move_text_caret("left", true));
    assert_eq!(focus.selected_scalar_range(), Some((1, 4)));
    assert!(focus.move_text_caret("left", false));
    assert_eq!(focus.resolved_caret_scalar_offset(), 1);
    assert_eq!(focus.selected_scalar_range(), None);
    assert!(focus.move_text_caret("right", false));
    assert_eq!(focus.resolved_caret_scalar_offset(), 3);
    assert_eq!(focus.value_text.as_str(), "Ae\u{0301}B");
}

#[test]
fn private_text_both_delete_directions_remove_the_whole_grapheme() {
    for (edge, step, backward) in [("end", "left", true), ("home", "right", false)] {
        let mut focus = focus_with_text("Ae\u{0301}B");
        focus.move_text_caret(edge, false);
        focus.move_text_caret(step, false);
        assert!(focus.delete_text(backward));
        assert_eq!(focus.value_text.as_str(), "AB", "backward: {backward}");
        assert_eq!(focus.resolved_caret_scalar_offset(), 1);
        assert_eq!(focus.selected_scalar_range(), None);
    }
}

#[test]
fn private_text_shift_then_typing_replaces_the_whole_grapheme() {
    let mut focus = focus_with_text("Ae\u{0301}");
    assert!(focus.move_text_caret("left", true));
    assert!(focus.insert_text("Q"));
    assert_eq!(focus.value_text.as_str(), "AQ");
    assert_eq!(focus.resolved_caret_scalar_offset(), 2);
    assert_eq!(focus.selected_scalar_range(), None);
}

#[test]
fn private_text_resolves_interior_scalar_caret_and_anchor_before_editing() {
    let mut focus = focus_with_text("Ae\u{0301}B");
    // Offset 2 is a legal scalar boundary, not a boundary of the visible character.
    focus.caret_scalar_offset = Some(2);
    focus.selection_anchor_scalar_offset = Some(4);
    assert_eq!(focus.resolved_caret_scalar_offset(), 1);
    assert_eq!(focus.selected_scalar_range(), Some((1, 4)));

    focus.caret_scalar_offset = Some(4);
    focus.selection_anchor_scalar_offset = Some(2);
    assert_eq!(focus.selected_scalar_range(), Some((1, 4)));
    assert!(focus.move_text_caret("left", true));
    assert_eq!(focus.selected_scalar_range(), Some((1, 3)));
    assert!(focus.delete_text(false));
    assert_eq!(focus.value_text.as_str(), "AB");
}

#[test]
fn private_text_keeps_the_caret_on_a_boundary_when_a_splice_joins_clusters() {
    let mut focus = focus_with_text("\u{0301}X");
    assert!(focus.move_text_caret("home", false));
    assert!(focus.insert_text("e"));
    assert_eq!(focus.value_text.as_str(), "e\u{0301}X");
    assert_eq!(focus.resolved_caret_scalar_offset(), 2);
    assert!(focus.delete_text(true));
    assert_eq!(focus.value_text.as_str(), "X");

    let mut focus = focus_with_text("🇦x🇧");
    focus.move_text_caret("home", false);
    focus.move_text_caret("right", false);
    assert!(focus.delete_text(false));
    assert_eq!(focus.value_text.as_str(), "🇦🇧");
    assert_eq!(focus.resolved_caret_scalar_offset(), 0);
}
