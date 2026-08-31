use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::accessibility::UiAccessibilityActionStatus;

use super::action_note;

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 150_000;

#[test]
fn optimization_batch_20260829ai_runtime308_action_notes_preserve_exact_text() {
    assert_eq!(
        action_note(UiAccessibilityActionStatus::Accepted, None, None),
        "status=accepted"
    );
    assert_eq!(
        action_note(
            UiAccessibilityActionStatus::Rejected,
            Some("invalid_target"),
            Some("target is stale"),
        ),
        "status=rejected code=invalid_target reason=target is stale"
    );
}

#[test]
fn optimization_batch_20260829ai_runtime308_action_notes_write_one_buffer() {
    let source = include_str!("../result.rs");
    let builder = source
        .split("pub(super) fn action_note")
        .nth(1)
        .expect("action note builder")
        .split("fn status_label")
        .next()
        .expect("action note builder body");

    assert!(builder.contains("String::with_capacity"));
    assert_eq!(builder.matches("write!(note").count(), 3);
    assert!(!builder.contains("let mut note = format!"));
    assert!(!builder.contains("push_str(code)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ai_runtime308_single_buffer_accessibility_action_note_bench() {
    let code = "focus_target_missing_after_workspace_restore";
    let reason = "the requested accessibility target is no longer present in the active scene";
    assert_eq!(
        action_note(
            UiAccessibilityActionStatus::Rejected,
            Some(code),
            Some(reason),
        ),
        legacy_action_note(
            UiAccessibilityActionStatus::Rejected,
            Some(code),
            Some(reason)
        )
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, code, reason));
            optimized_samples.push(measure(true, code, reason));
        } else {
            optimized_samples.push(measure(true, code, reason));
            legacy_samples.push(measure(false, code, reason));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME308_SINGLE_BUFFER_ACCESSIBILITY_ACTION_NOTE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} legacy_string_buffers_per_build=2 \
optimized_string_buffers_per_build=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_action_note(
    status: UiAccessibilityActionStatus,
    code: Option<&str>,
    reason: Option<&str>,
) -> String {
    let mut note = format!("status={}", status_label(status));
    if let Some(code) = code {
        note.push_str(" code=");
        note.push_str(code);
    }
    if let Some(reason) = reason {
        note.push_str(" reason=");
        note.push_str(reason);
    }
    note
}

fn status_label(status: UiAccessibilityActionStatus) -> &'static str {
    match status {
        UiAccessibilityActionStatus::Accepted => "accepted",
        UiAccessibilityActionStatus::Rejected => "rejected",
        UiAccessibilityActionStatus::Unsupported => "unsupported",
        UiAccessibilityActionStatus::StaleTarget => "stale_target",
    }
}

fn measure(optimized: bool, code: &str, reason: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let note = if optimized {
            action_note(
                UiAccessibilityActionStatus::Rejected,
                Some(black_box(code)),
                Some(black_box(reason)),
            )
        } else {
            legacy_action_note(
                UiAccessibilityActionStatus::Rejected,
                Some(black_box(code)),
                Some(black_box(reason)),
            )
        };
        checksum = checksum.wrapping_add(black_box(note).len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
