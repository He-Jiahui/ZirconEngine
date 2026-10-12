use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use crate::core::editing::operation::{
    EditOperationTarget, OperationCommand, OperationCommandFactory, OperationCommandFactoryError,
    OperationCommandFactoryRegistration, PendingEditRetention,
};
use crate::core::editor_operation::EditorOperationInvocation;
use crate::core::notifications::ToastNotification;
use crate::core::play::{PendingEditApplyBudget, PendingEditQueue};
use crate::ui::host::EditorOperationDispatchError;

use super::{
    pending_play_failure_toast_message, PlayPendingEditApplyFailure,
    MAX_PLAY_PENDING_FAILURE_DETAILS,
};

const RENDERS_PER_SAMPLE: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn editor887_play_pending_failure_single_buffer_preserves_exact_text() {
    let cases = [
        Vec::new(),
        vec![" concise failure ".to_string()],
        (0..6)
            .map(|index| format!("ordered failure {index}"))
            .collect::<Vec<_>>(),
        vec!["中".repeat(512), " trailing whitespace   ".to_string()],
    ];

    for reasons in cases {
        let failures = fixture_failures(&reasons);
        assert_eq!(
            pending_play_failure_toast_message(&failures),
            legacy_failure_toast_message(&failures),
            "single-buffer aggregation must preserve legacy text for {reasons:?}"
        );
    }

    assert_eq!(
        pending_play_failure_toast_message(&[]),
        "Queued edits could not be applied."
    );
    let unicode = fixture_failures(&["中".repeat(512)]);
    let message = pending_play_failure_toast_message(&unicode);
    assert!(message.len() <= 256);
    assert!(message.is_char_boundary(message.len()));
    assert!(message.ends_with("..."));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor887_play_pending_failure_single_buffer_benchmark() {
    let reasons = (0..MAX_PLAY_PENDING_FAILURE_DETAILS)
        .map(|index| format!("pending edit failure {index}"))
        .collect::<Vec<_>>();
    let failures = fixture_failures(&reasons);
    assert_eq!(
        pending_play_failure_toast_message(&failures),
        legacy_failure_toast_message(&failures)
    );

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&failures, legacy_failure_toast_message));
            optimized.push(measure(&failures, pending_play_failure_toast_message));
        } else {
            optimized.push(measure(&failures, pending_play_failure_toast_message));
            legacy.push(measure(&failures, legacy_failure_toast_message));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "EDITOR887_PLAY_PENDING_FAILURE_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} renders_per_sample={RENDERS_PER_SAMPLE} details_per_render={MAX_PLAY_PENDING_FAILURE_DETAILS} legacy_staged_child_strings_per_sample=49152 legacy_temporary_vector_slots_per_sample=16384 optimized_staged_child_strings_per_sample=0 optimized_temporary_vector_slots_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(110),
        "single-buffer failure detail P95 {optimized_p95}ns must stay within 10% of staged aggregation P95 {legacy_p95}ns"
    );
}

fn fixture_failures(reasons: &[String]) -> Vec<PlayPendingEditApplyFailure> {
    let queue = PendingEditQueue::default();
    for index in 0..reasons.len() {
        let invocation =
            EditorOperationInvocation::parse(format!("editor.test.pending_failure_{index}"))
                .expect("fixture operation path should be valid");
        let deferred = OperationCommandFactoryRegistration::new(
            invocation.operation_id.clone(),
            "pending failure message fixture",
            EditOperationTarget::EditWorkspace,
            Arc::new(PendingFailureFixtureFactory),
        )
        .with_pending_edit_retention(PendingEditRetention::Lossless)
        .defer(invocation)
        .expect("fixture registration should bind its invocation");
        queue
            .enqueue(EditOperationTarget::EditWorkspace, deferred)
            .expect("fixture failure should enqueue");
    }

    let mut reason_index = 0;
    queue
        .apply_with_budget(PendingEditApplyBudget::unlimited(), |intent| {
            let reason = reasons[reason_index].clone();
            reason_index += 1;
            Err(EditorOperationDispatchError::from(
                OperationCommandFactoryError::Factory {
                    operation: intent.invocation.operation_id.clone(),
                    reason,
                },
            ))
        })
        .failures
        .into_iter()
        .map(|failure| PlayPendingEditApplyFailure::new(failure.intent, failure.error))
        .collect()
}

fn legacy_failure_toast_message(failures: &[PlayPendingEditApplyFailure]) -> String {
    let diagnostics = failures
        .iter()
        .take(MAX_PLAY_PENDING_FAILURE_DETAILS)
        .map(|failure| {
            let error = ToastNotification::bounded_message(
                &failure.error().to_string(),
                "pending edit operation failed",
            );
            ToastNotification::bounded_message(
                &format!("pending edit intent {:?} failed: {error}", failure.intent()),
                "Queued edits could not be applied.",
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    ToastNotification::bounded_message(&diagnostics, "Queued edits could not be applied.")
}

fn measure(
    failures: &[PlayPendingEditApplyFailure],
    render: fn(&[PlayPendingEditApplyFailure]) -> String,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..RENDERS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(failures))).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

struct PendingFailureFixtureFactory;

impl OperationCommandFactory for PendingFailureFixtureFactory {
    fn create(
        &self,
        _invocation: &EditorOperationInvocation,
    ) -> Result<OperationCommand, OperationCommandFactoryError> {
        unreachable!("failure message fixtures never execute the deferred operation")
    }
}
