use std::hint::black_box;
use std::time::{Duration, Instant};

use super::AssetToolkitDescriptor;
use crate::core::editor_operation::EditorOperationPath;

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 96;
const CAPABILITY_COUNT: usize = 4_096;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn capability_fixture() -> Vec<String> {
    (0..CAPABILITY_COUNT)
        .rev()
        .map(|index| {
            format!(
                "editor.asset.capability.{:04}",
                index % (CAPABILITY_COUNT / 2)
            )
        })
        .collect()
}

fn operation_path() -> EditorOperationPath {
    EditorOperationPath::parse("fixture.asset.open").expect("valid fixture operation")
}

fn legacy_descriptor(capabilities: &[String]) -> AssetToolkitDescriptor {
    let mut descriptor = AssetToolkitDescriptor::new("fixture.asset.view", operation_path());
    descriptor
        .required_capabilities
        .extend(capabilities.iter().cloned());
    descriptor.required_capabilities.sort();
    descriptor.required_capabilities.dedup();
    descriptor
}

fn optimized_descriptor(capabilities: &[String]) -> AssetToolkitDescriptor {
    AssetToolkitDescriptor::new("fixture.asset.view", operation_path())
        .with_required_capabilities(capabilities.iter().cloned())
}

#[test]
fn editor04_type_registry_toolkit_capability_order_is_unchanged() {
    let capabilities = capability_fixture();
    let legacy = legacy_descriptor(&capabilities);
    let optimized = optimized_descriptor(&capabilities);
    assert_eq!(
        optimized.required_capabilities(),
        legacy.required_capabilities()
    );
    assert_eq!(
        optimized.required_capabilities().len(),
        CAPABILITY_COUNT / 2
    );
    assert!(optimized
        .required_capabilities()
        .windows(2)
        .all(|window| window[0] < window[1]));
}

#[test]
fn editor04_type_registry_toolkit_source_contract() {
    let source = include_str!("../toolkit.rs");
    assert!(source.contains("let (lower_bound, _) = capabilities.size_hint();"));
    assert!(source.contains("self.required_capabilities.reserve(lower_bound);"));
    assert!(source.contains("self.required_capabilities.sort_unstable();"));
    assert!(!source.contains("self.required_capabilities.sort();"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor04_type_registry_toolkit_capability_bench() {
    let capabilities = capability_fixture();
    let legacy = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_descriptor(&capabilities));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let optimized = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(optimized_descriptor(&capabilities));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "EDITOR04_TYPE_REGISTRY_TOOLKIT_CAPABILITY_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} capabilities={} unique_capabilities={} stable_sort=1->0 preallocation=0->{}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        CAPABILITY_COUNT,
        CAPABILITY_COUNT / 2,
        CAPABILITY_COUNT,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}
