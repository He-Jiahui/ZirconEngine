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
