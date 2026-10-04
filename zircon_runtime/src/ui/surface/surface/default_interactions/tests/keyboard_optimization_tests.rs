use std::hint::black_box;
use std::time::Instant;

use super::append_or_adopt_batch;

#[test]
fn optimization_batch_dk_keyboard_event_batch_adoption_preserves_order() {
    let mut output = None;

    append_or_adopt_batch(&mut output, Vec::<u32>::new());
    append_or_adopt_batch(&mut output, vec![1, 2]);
    append_or_adopt_batch(&mut output, vec![3, 4]);

    assert_eq!(output, Some(vec![1, 2, 3, 4]));
}

#[test]
fn optimization_batch_dk_semantic_keyboard_adopts_first_batch_source() {
    let source = include_str!("../keyboard.rs");
    let function = source
        .split("pub(crate) fn apply_default_semantic_keyboard_component_action")
        .nth(1)
        .expect("semantic keyboard action")
        .split("pub(crate) fn apply_default_semantic_keyboard_component_text")
        .next()
        .expect("semantic action body");

    assert!(function.contains("let mut component_events = None;"));
    assert!(function.contains("append_or_adopt_batch"));
    assert!(!function.contains("let mut component_events = Vec::new();"));
}

#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_dk_adopt_first_keyboard_event_batch_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const MERGES_PER_SAMPLE: usize = 32_768;
    const REPORTS_PER_BATCH: usize = 32;

    let template = (0..REPORTS_PER_BATCH as u64).collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_batch_adoption(&template, MERGES_PER_SAMPLE, true));
            optimized_samples.push(measure_batch_adoption(&template, MERGES_PER_SAMPLE, false));
        } else {
            optimized_samples.push(measure_batch_adoption(&template, MERGES_PER_SAMPLE, false));
            legacy_samples.push(measure_batch_adoption(&template, MERGES_PER_SAMPLE, true));
        }
    }

    let legacy_p95 = p95(&mut legacy_samples);
    let optimized_p95 = p95(&mut optimized_samples);
    println!(
        "RUNTIME419_ADOPT_FIRST_KEYBOARD_EVENT_BATCH_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "adopted keyboard event batch p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );
}

fn measure_batch_adoption(template: &[u64], merges: usize, legacy: bool) -> u128 {
    let started_at = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..merges {
        let batch = black_box(template).to_vec();
        let output = if legacy {
            let mut output = Vec::new();
            output.extend(batch);
            output
        } else {
            let mut output = None;
            append_or_adopt_batch(&mut output, batch);
            output.expect("non-empty batch is adopted")
        };
        checksum = checksum.wrapping_add(output.len() as u64);
        black_box(output);
    }
    black_box(checksum);
    started_at.elapsed().as_nanos()
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    let index = samples
        .len()
        .saturating_mul(95)
        .div_ceil(100)
        .saturating_sub(1);
    samples[index]
}
