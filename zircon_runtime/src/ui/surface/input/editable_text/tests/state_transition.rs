use super::{delete_surrounding_text_state, retained_document_replaced_range};
use zircon_runtime_interface::ui::{
    dispatch::UiImeDeleteSurrounding,
    surface::{UiEditableTextState, UiTextCaret, UiTextComposition, UiTextRange},
};

#[test]
fn delete_surrounding_uses_the_visible_paint_only_composition_text() {
    let state = UiEditableTextState {
        text: "aXb".to_owned(),
        caret: UiTextCaret {
            offset: 2,
            ..Default::default()
        },
        composition: Some(UiTextComposition {
            range: UiTextRange { start: 1, end: 2 },
            preedit_clauses: Vec::new(),
            text: "X".to_owned(),
            restore_text: None,
        }),
        ..Default::default()
    };

    let next = delete_surrounding_text_state(state, UiImeDeleteSurrounding::new(1, 0))
        .expect("visible text has a byte before the mapped caret");

    assert_eq!(next.state.text, "Xb");
    assert!(next.state.composition.is_none());
    let intent = next.committed_edit.expect("surrounding delete intent");
    assert_eq!(intent.old, 0..1);
    assert_eq!(intent.new, 0..0);
}

#[test]
fn retained_document_range_uses_composition_restore_length_not_visible_preedit_length() {
    let state = UiEditableTextState {
        text: "aXYf".to_owned(),
        composition: Some(UiTextComposition {
            range: UiTextRange { start: 1, end: 3 },
            preedit_clauses: Vec::new(),
            text: "XY".to_owned(),
            restore_text: Some("bcde".to_owned()),
        }),
        ..Default::default()
    };

    assert_eq!(
        retained_document_replaced_range(&state),
        UiTextRange { start: 1, end: 5 }
    );
}
