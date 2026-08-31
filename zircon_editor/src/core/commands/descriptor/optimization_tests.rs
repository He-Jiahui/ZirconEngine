use std::hint::black_box;
use std::time::{Duration, Instant};

use super::EditorCommandDescriptor;
use crate::core::commands::CommandEvalCtx;
use crate::core::editor_operation::EditorOperationPath;

const SAMPLE_COUNT: usize = 17;
const KEYWORD_ITERATIONS: usize = 96;
const KEYWORD_COUNT: usize = 4_096;
const CAPABILITY_ITERATIONS: usize = 256;
const CAPABILITY_COUNT: usize = 1_024;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn keyword_fixture() -> Vec<String> {
    (0..KEYWORD_COUNT)
        .rev()
        .map(|index| format!("keyword.{:04}", index % (KEYWORD_COUNT / 2)))
        .collect()
}

fn command_descriptor() -> EditorCommandDescriptor {
    EditorCommandDescriptor::operation(
        EditorOperationPath::parse("fixture.command.normalize").expect("valid fixture route"),
    )
}

fn legacy_keyword_descriptor(keywords: &[String]) -> EditorCommandDescriptor {
    let mut descriptor = command_descriptor();
    descriptor.keywords = keywords.iter().cloned().collect();
    descriptor.keywords.sort();
    descriptor.keywords.dedup();
    descriptor
}

fn optimized_keyword_descriptor(keywords: &[String]) -> EditorCommandDescriptor {
    command_descriptor().with_keywords(keywords.iter().cloned())
}

fn capability_fixture() -> Vec<String> {
    (0..CAPABILITY_COUNT)
        .map(|index| format!("editor.capability.{index:04}"))
        .collect()
}

fn legacy_missing_capabilities(
    descriptor: &EditorCommandDescriptor,
    context: &CommandEvalCtx,
) -> Vec<String> {
    let mut missing = Vec::new();
    for capability in descriptor.required_capabilities() {
        if !context.has_capability(capability) {
            missing.push(capability.clone());
        }
    }
    missing
}

#[test]
fn editor08_command_descriptor_keyword_normalization_preserves_order_and_uniqueness() {
    let keywords = keyword_fixture();
    let legacy = legacy_keyword_descriptor(&keywords);
    let optimized = optimized_keyword_descriptor(&keywords);

    assert_eq!(optimized.keywords(), legacy.keywords());
    assert_eq!(optimized.keywords().len(), KEYWORD_COUNT / 2);
    assert!(optimized
        .keywords()
        .windows(2)
        .all(|window| window[0] < window[1]));
}

#[test]
fn editor08_command_descriptor_missing_capability_diagnostics_preserve_order() {
    let descriptor = command_descriptor().with_required_capabilities(capability_fixture());
    let context = CommandEvalCtx::headless(std::iter::empty::<String>());

    let legacy = legacy_missing_capabilities(&descriptor, &context);
    let optimized = descriptor.missing_required_capabilities(&context);

    assert_eq!(optimized, legacy);
    assert_eq!(optimized.len(), CAPABILITY_COUNT);
}

#[test]
fn editor08_command_descriptor_source_contracts() {
    let source = include_str!("../descriptor.rs");
    let keyword_body = source
        .split_once("pub fn with_keywords")
        .expect("keyword normalizer should exist")
        .1
        .split_once("pub fn with_payload_schema_id")
        .expect("keyword normalizer should precede payload metadata")
        .0;
    assert!(keyword_body.contains("size_hint()"));
    assert!(keyword_body.contains("Vec::with_capacity(lower_bound)"));
    assert!(keyword_body.contains("sort_unstable()"));
    assert!(!keyword_body.contains("self.keywords.sort();"));

    let capability_body = source
        .split_once("pub(crate) fn missing_required_capabilities")
        .expect("missing capability helper should exist")
        .1
        .split_once("pub fn keywords")
        .expect("missing capability helper should precede keyword accessors")
        .0;
    assert!(capability_body.contains("Vec::with_capacity(self.required_capabilities.len())"));
    assert!(capability_body.contains("for capability in &self.required_capabilities"));
    assert!(!capability_body.contains(".filter(") && !capability_body.contains(".collect()"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor08_command_descriptor_keyword_normalization_bench() {
    let keywords = keyword_fixture();
    let legacy = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..KEYWORD_ITERATIONS {
                black_box(legacy_keyword_descriptor(&keywords));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let optimized = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..KEYWORD_ITERATIONS {
                black_box(optimized_keyword_descriptor(&keywords));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "EDITOR08_COMMAND_DESCRIPTOR_KEYWORD_NORMALIZATION_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} keywords={} unique_keywords={} stable_sort=1->0 preallocation=0->{}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        KEYWORD_ITERATIONS,
        KEYWORD_COUNT,
        KEYWORD_COUNT / 2,
        KEYWORD_COUNT,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor08_command_descriptor_missing_capability_diagnostics_bench() {
    let descriptor = command_descriptor().with_required_capabilities(capability_fixture());
    let context = CommandEvalCtx::headless(std::iter::empty::<String>());
    let legacy = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..CAPABILITY_ITERATIONS {
                black_box(legacy_missing_capabilities(&descriptor, &context));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let optimized = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..CAPABILITY_ITERATIONS {
                black_box(descriptor.missing_required_capabilities(&context));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "EDITOR08_COMMAND_DESCRIPTOR_MISSING_CAPABILITY_DIAGNOSTICS_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} capabilities={} missing_capabilities={} allocations=geometric->single",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        CAPABILITY_ITERATIONS,
        CAPABILITY_COUNT,
        CAPABILITY_COUNT,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}
