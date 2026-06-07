use zircon_runtime_interface::ui::{
    dispatch::UiTextByteRange,
    surface::{UiEditableTextState, UiTextEditAction, UiTextRange, UiTextSelection},
};

use crate::ui::text::apply_text_edit_action;

use super::super::{text_constraints::TextInputConstraints, text_state::clamp_text_boundary};

pub(super) fn committed_text_state(
    editable: UiEditableTextState,
    text: String,
    constraints: TextInputConstraints,
) -> UiEditableTextState {
    if editable.composition.is_some() {
        let range = editable
            .composition
            .as_ref()
            .map(|composition| composition.range)
            .unwrap_or(UiTextRange {
                start: editable.caret.offset,
                end: editable.caret.offset,
            });
        let text = constraints.sanitize_replacement(&editable.text, range, &text);
        let composed =
            apply_text_edit_action(editable, UiTextEditAction::SetComposition { range, text });
        apply_text_edit_action(composed, UiTextEditAction::CommitComposition)
    } else {
        let range = editable
            .selection
            .as_ref()
            .map(UiTextSelection::range)
            .unwrap_or(UiTextRange {
                start: editable.caret.offset,
                end: editable.caret.offset,
            });
        let text = constraints.sanitize_replacement(&editable.text, range, &text);
        apply_text_edit_action(editable, UiTextEditAction::Insert { text })
    }
}

pub(super) fn preedit_text_state(
    editable: UiEditableTextState,
    preedit: &str,
    cursor_range: Option<UiTextByteRange>,
    constraints: TextInputConstraints,
) -> UiEditableTextState {
    let range = editable
        .composition
        .as_ref()
        .map(|composition| composition.range)
        .or_else(|| editable.selection.as_ref().map(UiTextSelection::range))
        .unwrap_or(UiTextRange {
            start: editable.caret.offset,
            end: editable.caret.offset,
        });
    let preedit = constraints.sanitize_replacement(&editable.text, range, preedit);
    let mut next = apply_text_edit_action(
        editable,
        UiTextEditAction::SetComposition {
            range,
            text: preedit,
        },
    );

    if let Some(cursor_range) = cursor_range {
        if let Some(composition) = next.composition.as_ref() {
            let anchor = composition.range.start
                + clamp_text_boundary(&composition.text, cursor_range.start_byte as usize);
            let focus = composition.range.start
                + clamp_text_boundary(&composition.text, cursor_range.end_byte as usize);
            next = if anchor == focus {
                apply_text_edit_action(
                    next,
                    UiTextEditAction::MoveCaret {
                        offset: focus,
                        extend_selection: false,
                    },
                )
            } else {
                apply_text_edit_action(next, UiTextEditAction::SetSelection { anchor, focus })
            };
        }
    }

    next
}
