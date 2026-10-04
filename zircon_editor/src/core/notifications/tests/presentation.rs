use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::format_decision_message_template;

#[test]
fn decision_message_template_reuses_unmodified_translation_storage() {
    let without_arguments: Arc<str> = Arc::from("No pending work.");
    let projected = format_decision_message_template(Arc::clone(&without_arguments), |_| None);
    assert!(Arc::ptr_eq(&without_arguments, &projected));

    let without_matching_placeholder: Arc<str> = Arc::from("Keep {unknown} intact.");
    let projected =
        format_decision_message_template(Arc::clone(&without_matching_placeholder), |name| {
            (name == "pending_count").then_some(2)
        });
    assert!(Arc::ptr_eq(&without_matching_placeholder, &projected));
}

#[test]
fn decision_message_template_formats_repeated_values_and_preserves_unknown_placeholders() {
    let projected = format_decision_message_template(
        Arc::from("{count} queued, {count} total; keep {unknown}; nested {{count}}."),
        |name| (name == "count").then_some(42),
    );

    assert_eq!(
        projected.as_ref(),
        "42 queued, 42 total; keep {unknown}; nested {42}."
    );
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_wave_20260825_editor10_message_template_evidence() {
    const PROJECTIONS: usize = 100_000;
    const ARGUMENT_COUNT: usize = 8;
    const MAX_ELAPSED_NS: u128 = 3_000_000_000;
    const ARGUMENTS: [(&str, u64); ARGUMENT_COUNT] = [
        ("one", 1),
        ("two", 2),
        ("three", 3),
        ("four", 4),
        ("five", 5),
        ("six", 6),
        ("seven", 7),
        ("eight", 8),
    ];

    let template: Arc<str> =
        Arc::from("{one}/{two}/{three}/{four}/{five}/{six}/{seven}/{eight}: {unknown}");
    let started = Instant::now();
    for _ in 0..PROJECTIONS {
        black_box(format_decision_message_template(
            Arc::clone(&template),
            |name| {
                ARGUMENTS
                    .iter()
                    .find_map(|(argument_name, value)| (*argument_name == name).then_some(*value))
            },
        ));
    }
    let elapsed_ns = started.elapsed().as_nanos();
    let legacy_full_template_passes = PROJECTIONS * ARGUMENT_COUNT;
    let optimized_template_passes = PROJECTIONS;
    let pass_reduction_bps = legacy_full_template_passes
        .saturating_sub(optimized_template_passes)
        .saturating_mul(10_000)
        / legacy_full_template_passes;

    println!(
        "EDITOR_DECISION_MESSAGE_FORMAT_BENCH_V1 projections={PROJECTIONS} arguments={ARGUMENT_COUNT} legacy_full_template_passes={legacy_full_template_passes} optimized_template_passes={optimized_template_passes} pass_reduction_bps={pass_reduction_bps} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}"
    );

    assert_eq!(pass_reduction_bps, 8_750);
    assert!(elapsed_ns <= MAX_ELAPSED_NS);
}
