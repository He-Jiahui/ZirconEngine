use zircon_runtime_interface::ui::{
    dispatch::{UiKeyboardInputEvent, UiKeyboardInputState},
    surface::{UiEditableTextState, UiTextEditAction},
};

use crate::ui::text::{
    line_end_boundary, line_start_boundary, next_grapheme_boundary, next_line_same_column_boundary,
    next_word_boundary, previous_grapheme_boundary, previous_line_same_column_boundary,
    previous_word_boundary,
};

pub(in crate::ui::surface::input) struct KeyboardTextEditActions {
    first: UiTextEditAction,
    second: Option<UiTextEditAction>,
}

impl IntoIterator for KeyboardTextEditActions {
    type Item = UiTextEditAction;
    type IntoIter = std::iter::Chain<
        std::iter::Once<UiTextEditAction>,
        std::option::IntoIter<UiTextEditAction>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self.first).chain(self.second)
    }
}

pub(in crate::ui::surface::input) fn keyboard_text_edit_actions(
    keyboard: &UiKeyboardInputEvent,
    state: &UiEditableTextState,
    secure: bool,
) -> Option<KeyboardTextEditActions> {
    if !matches!(
        keyboard.state,
        UiKeyboardInputState::Pressed | UiKeyboardInputState::Repeated
    ) {
        return None;
    }

    let extend_selection = keyboard.metadata.modifiers.shift;
    let secure_line_navigation = secure && keyboard.metadata.modifiers.control;
    let secure_line_deletion = secure_line_navigation
        && !keyboard.metadata.modifiers.alt
        && !keyboard.metadata.modifiers.shift;
    let word_navigation = !secure && keyboard.metadata.modifiers.control;
    let document_navigation =
        keyboard.metadata.modifiers.control || keyboard.metadata.modifiers.super_key;
    let hard_line_navigation = secure_line_navigation
        || (keyboard.metadata.modifiers.super_key
            && !keyboard.metadata.modifiers.control
            && !keyboard.metadata.modifiers.alt);
    match keyboard.logical_key.as_str() {
        key if keyboard_requests_select_all(keyboard, key) => {
            Some(single_action(UiTextEditAction::SetSelection {
                anchor: 0,
                focus: state.text.len(),
            }))
        }
        "Backspace" if secure_line_deletion => Some(delete_to_line_start_actions(state)),
        "Delete" if secure_line_deletion => Some(delete_to_line_end_actions(state)),
        "Backspace" if word_navigation => Some(delete_previous_word_actions(state)),
        "Delete" if word_navigation => Some(delete_next_word_actions(state)),
        "Backspace" => Some(single_action(UiTextEditAction::Backspace)),
        "Delete" => Some(single_action(UiTextEditAction::Delete)),
        "Escape" => Some(escape_actions(state)),
        "ArrowLeft" if hard_line_navigation => Some(single_action(UiTextEditAction::MoveCaret {
            offset: line_start_boundary(&state.text, state.caret.offset),
            extend_selection,
        })),
        "ArrowRight" if hard_line_navigation => Some(single_action(UiTextEditAction::MoveCaret {
            offset: line_end_boundary(&state.text, state.caret.offset),
            extend_selection,
        })),
        "ArrowLeft" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: previous_text_boundary(&state.text, state.caret.offset, word_navigation),
            extend_selection,
        })),
        "ArrowRight" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: next_text_boundary(&state.text, state.caret.offset, word_navigation),
            extend_selection,
        })),
        "ArrowUp" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: previous_line_offset(state, document_navigation),
            extend_selection,
        })),
        "ArrowDown" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: next_line_offset(state, document_navigation),
            extend_selection,
        })),
        "Home" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: home_offset(state, document_navigation),
            extend_selection,
        })),
        "End" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: end_offset(state, document_navigation),
            extend_selection,
        })),
        _ => keyboard_text_edit_actions_from_key_code(
            keyboard,
            state,
            extend_selection,
            word_navigation,
            secure_line_navigation,
            secure_line_deletion,
        ),
    }
}

fn keyboard_text_edit_actions_from_key_code(
    keyboard: &UiKeyboardInputEvent,
    state: &UiEditableTextState,
    extend_selection: bool,
    word_navigation: bool,
    secure_line_navigation: bool,
    secure_line_deletion: bool,
) -> Option<KeyboardTextEditActions> {
    let document_navigation =
        keyboard.metadata.modifiers.control || keyboard.metadata.modifiers.super_key;
    let hard_line_navigation = secure_line_navigation
        || (keyboard.metadata.modifiers.super_key
            && !keyboard.metadata.modifiers.control
            && !keyboard.metadata.modifiers.alt);
    match keyboard.key_code {
        65 | 97 if keyboard_requests_select_all(keyboard, "") => {
            Some(single_action(UiTextEditAction::SetSelection {
                anchor: 0,
                focus: state.text.len(),
            }))
        }
        8 if secure_line_deletion => Some(delete_to_line_start_actions(state)),
        46 if secure_line_deletion => Some(delete_to_line_end_actions(state)),
        8 if word_navigation => Some(delete_previous_word_actions(state)),
        46 if word_navigation => Some(delete_next_word_actions(state)),
        8 => Some(single_action(UiTextEditAction::Backspace)),
        46 => Some(single_action(UiTextEditAction::Delete)),
        27 => Some(escape_actions(state)),
        37 if hard_line_navigation => Some(single_action(UiTextEditAction::MoveCaret {
            offset: line_start_boundary(&state.text, state.caret.offset),
            extend_selection,
        })),
        39 if hard_line_navigation => Some(single_action(UiTextEditAction::MoveCaret {
            offset: line_end_boundary(&state.text, state.caret.offset),
            extend_selection,
        })),
        37 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: previous_text_boundary(&state.text, state.caret.offset, word_navigation),
            extend_selection,
        })),
        39 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: next_text_boundary(&state.text, state.caret.offset, word_navigation),
            extend_selection,
        })),
        38 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: previous_line_offset(state, document_navigation),
            extend_selection,
        })),
        40 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: next_line_offset(state, document_navigation),
            extend_selection,
        })),
        36 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: home_offset(state, document_navigation),
            extend_selection,
        })),
        35 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: end_offset(state, document_navigation),
            extend_selection,
        })),
        _ => None,
    }
}

fn delete_previous_word_actions(state: &UiEditableTextState) -> KeyboardTextEditActions {
    if has_active_selection(state) {
        return single_action(UiTextEditAction::Backspace);
    }
    let caret = state.caret.offset.min(state.text.len());
    let start = previous_text_boundary(&state.text, caret, true);
    if start == caret {
        single_action(UiTextEditAction::Backspace)
    } else {
        double_action(
            UiTextEditAction::SetSelection {
                anchor: start,
                focus: caret,
            },
            UiTextEditAction::Backspace,
        )
    }
}

fn delete_next_word_actions(state: &UiEditableTextState) -> KeyboardTextEditActions {
    if has_active_selection(state) {
        return single_action(UiTextEditAction::Delete);
    }
    let caret = state.caret.offset.min(state.text.len());
    let end = next_text_boundary(&state.text, caret, true);
    if end == caret {
        single_action(UiTextEditAction::Delete)
    } else {
        double_action(
            UiTextEditAction::SetSelection {
                anchor: caret,
                focus: end,
            },
            UiTextEditAction::Delete,
        )
    }
}

fn delete_to_line_start_actions(state: &UiEditableTextState) -> KeyboardTextEditActions {
    if has_active_selection(state) {
        return single_action(UiTextEditAction::Backspace);
    }
    let caret = state.caret.offset.min(state.text.len());
    let start = line_start_boundary(&state.text, caret);
    if start == caret {
        single_action(UiTextEditAction::Backspace)
    } else {
        double_action(
            UiTextEditAction::SetSelection {
                anchor: start,
                focus: caret,
            },
            UiTextEditAction::Backspace,
        )
    }
}

fn delete_to_line_end_actions(state: &UiEditableTextState) -> KeyboardTextEditActions {
    if has_active_selection(state) {
        return single_action(UiTextEditAction::Delete);
    }
    let caret = state.caret.offset.min(state.text.len());
    let end = line_end_boundary(&state.text, caret);
    if end == caret {
        single_action(UiTextEditAction::Delete)
    } else {
        double_action(
            UiTextEditAction::SetSelection {
                anchor: caret,
                focus: end,
            },
            UiTextEditAction::Delete,
        )
    }
}

fn escape_actions(state: &UiEditableTextState) -> KeyboardTextEditActions {
    if state.composition.is_some() {
        single_action(UiTextEditAction::CancelComposition)
    } else {
        single_action(UiTextEditAction::MoveCaret {
            offset: state.caret.offset,
            extend_selection: false,
        })
    }
}

fn single_action(action: UiTextEditAction) -> KeyboardTextEditActions {
    KeyboardTextEditActions {
        first: action,
        second: None,
    }
}

fn double_action(first: UiTextEditAction, second: UiTextEditAction) -> KeyboardTextEditActions {
    KeyboardTextEditActions {
        first,
        second: Some(second),
    }
}

fn has_active_selection(state: &UiEditableTextState) -> bool {
    state
        .selection
        .as_ref()
        .is_some_and(|selection| selection.anchor != selection.focus)
}

fn home_offset(state: &UiEditableTextState, document_navigation: bool) -> usize {
    if document_navigation {
        0
    } else {
        line_start_boundary(&state.text, state.caret.offset)
    }
}

fn end_offset(state: &UiEditableTextState, document_navigation: bool) -> usize {
    if document_navigation {
        state.text.len()
    } else {
        line_end_boundary(&state.text, state.caret.offset)
    }
}

fn previous_line_offset(state: &UiEditableTextState, document_navigation: bool) -> usize {
    if document_navigation {
        0
    } else {
        previous_line_same_column_boundary(&state.text, state.caret.offset).unwrap_or(0)
    }
}

fn next_line_offset(state: &UiEditableTextState, document_navigation: bool) -> usize {
    if document_navigation {
        state.text.len()
    } else {
        next_line_same_column_boundary(&state.text, state.caret.offset).unwrap_or(state.text.len())
    }
}

fn keyboard_requests_select_all(keyboard: &UiKeyboardInputEvent, logical_key: &str) -> bool {
    (keyboard.metadata.modifiers.control || keyboard.metadata.modifiers.super_key)
        && !keyboard.metadata.modifiers.alt
        && matches!(logical_key, "a" | "A" | "")
}

fn previous_text_boundary(text: &str, offset: usize, word_navigation: bool) -> usize {
    if word_navigation {
        previous_word_boundary(text, offset).unwrap_or(0)
    } else {
        previous_grapheme_boundary(text, offset).unwrap_or(0)
    }
}

fn next_text_boundary(text: &str, offset: usize, word_navigation: bool) -> usize {
    if word_navigation {
        next_word_boundary(text, offset).unwrap_or(text.len())
    } else {
        next_grapheme_boundary(text, offset).unwrap_or(text.len())
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{double_action, single_action};
    use zircon_runtime_interface::ui::surface::UiTextEditAction;

    const ACTION_SEQUENCES_PER_SAMPLE: usize = 262_144;
    const SAMPLE_PAIRS: usize = 17;

    fn consume(actions: impl IntoIterator<Item = UiTextEditAction>) -> u64 {
        actions.into_iter().fold(0_u64, |checksum, action| {
            checksum.wrapping_add(match action {
                UiTextEditAction::MoveCaret {
                    offset,
                    extend_selection,
                } => offset as u64 + u64::from(extend_selection),
                UiTextEditAction::Delete => 7,
                _ => 0,
            })
        })
    }

    fn legacy_actions(index: usize) -> Vec<UiTextEditAction> {
        let move_caret = UiTextEditAction::MoveCaret {
            offset: index,
            extend_selection: index % 8 != 0 && index % 2 == 0,
        };
        if index % 8 == 0 {
            vec![move_caret, UiTextEditAction::Delete]
        } else {
            vec![move_caret]
        }
    }

    fn inline_actions(index: usize) -> super::KeyboardTextEditActions {
        let move_caret = UiTextEditAction::MoveCaret {
            offset: index,
            extend_selection: index % 8 != 0 && index % 2 == 0,
        };
        if index % 8 == 0 {
            double_action(move_caret, UiTextEditAction::Delete)
        } else {
            single_action(move_caret)
        }
    }

    fn measure(optimized: bool) -> (u128, u64) {
        let started = Instant::now();
        let mut checksum = 0_u64;
        for index in 0..ACTION_SEQUENCES_PER_SAMPLE {
            checksum = checksum.wrapping_add(if optimized {
                consume(black_box(inline_actions(index)))
            } else {
                consume(black_box(legacy_actions(index)))
            });
        }
        (started.elapsed().as_nanos().max(1), black_box(checksum))
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    #[test]
    fn runtime82_batch_inline_single_action_preserves_order() {
        let mut actions = single_action(UiTextEditAction::Backspace).into_iter();

        assert_eq!(actions.next(), Some(UiTextEditAction::Backspace));
        assert_eq!(actions.next(), None);
    }

    #[test]
    fn runtime82_batch_inline_two_action_word_delete_preserves_order() {
        let mut actions = double_action(
            UiTextEditAction::SetSelection {
                anchor: 3,
                focus: 8,
            },
            UiTextEditAction::Delete,
        )
        .into_iter();

        assert_eq!(
            actions.next(),
            Some(UiTextEditAction::SetSelection {
                anchor: 3,
                focus: 8,
            })
        );
        assert_eq!(actions.next(), Some(UiTextEditAction::Delete));
        assert_eq!(actions.next(), None);
    }

    #[test]
    fn runtime82_batch_inline_action_iterator_stays_exhausted() {
        let mut actions = single_action(UiTextEditAction::CancelComposition).into_iter();

        assert_eq!(actions.next(), Some(UiTextEditAction::CancelComposition));
        assert_eq!(actions.next(), None);
        assert_eq!(actions.next(), None);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn runtime82_batch_inline_keyboard_edit_actions_p95() {
        for _ in 0..3 {
            black_box(measure(false));
            black_box(measure(true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut inline_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut legacy_checksum = 0_u64;
        let mut inline_checksum = 0_u64;
        for pair in 0..SAMPLE_PAIRS {
            let (legacy, inline) = if pair % 2 == 0 {
                (measure(false), measure(true))
            } else {
                let inline = measure(true);
                let legacy = measure(false);
                (legacy, inline)
            };
            legacy_samples.push(legacy.0);
            inline_samples.push(inline.0);
            legacy_checksum = legacy.1;
            inline_checksum = inline.1;
        }

        assert_eq!(legacy_checksum, inline_checksum);
        let legacy_p50_ns = nearest_rank(&legacy_samples, 50);
        let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
        let inline_p50_ns = nearest_rank(&inline_samples, 50);
        let inline_p95_ns = nearest_rank(&inline_samples, 95);
        let action_size_bytes = std::mem::size_of::<UiTextEditAction>();
        let legacy_sequence_payload_bytes = action_size_bytes
            .saturating_mul(ACTION_SEQUENCES_PER_SAMPLE + ACTION_SEQUENCES_PER_SAMPLE / 8);
        println!(
            "RUNTIME82_INLINE_KEYBOARD_EDIT_ACTIONS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             action_sequences={ACTION_SEQUENCES_PER_SAMPLE} pair_order=alternating_legacy_even \
             legacy_first_pairs=9 inline_first_pairs=8 action_size_bytes={action_size_bytes} \
             legacy_sequence_container_allocations={ACTION_SEQUENCES_PER_SAMPLE} \
             inline_sequence_container_allocations=0 \
             legacy_sequence_payload_bytes={legacy_sequence_payload_bytes} \
             inline_sequence_payload_bytes=0 legacy_p50_ns={legacy_p50_ns} \
             legacy_p95_ns={legacy_p95_ns} inline_p50_ns={inline_p50_ns} \
             inline_p95_ns={inline_p95_ns} checksum={legacy_checksum}"
        );
        assert!(inline_p95_ns.saturating_mul(2) <= legacy_p95_ns);
    }
}
