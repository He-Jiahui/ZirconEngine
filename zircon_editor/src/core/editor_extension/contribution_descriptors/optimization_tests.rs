use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{push_normalized_extension, AssetImporterDescriptor, EditorMenuItemDescriptor};
use crate::core::commands::EditorCommandMenuPath;
use crate::core::editor_operation::EditorOperationPath;

const SAMPLE_COUNT: usize = 17;
const CAPABILITY_ITERATIONS: usize = 96;
const CAPABILITY_COUNT: usize = 4_096;
const EXTENSION_ITERATIONS: usize = 256;
const EXTENSION_COUNT: usize = 2_048;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn operation_path() -> EditorOperationPath {
    EditorOperationPath::parse("fixture.extension.command").expect("valid fixture operation")
}

fn capability_fixture() -> Vec<String> {
    (0..CAPABILITY_COUNT)
        .rev()
        .map(|index| format!("editor.capability.{:04}", index % (CAPABILITY_COUNT / 2)))
        .collect()
}

fn legacy_menu_descriptor(capabilities: &[String]) -> EditorMenuItemDescriptor {
    let operation = operation_path();
    let mut descriptor = EditorMenuItemDescriptor::new(
        EditorCommandMenuPath::builtin(&operation, "tools", &["fixture"]),
        operation,
    );
    descriptor
        .required_capabilities
        .extend(capabilities.iter().cloned());
    descriptor.required_capabilities.sort();
    descriptor.required_capabilities.dedup();
    descriptor
}

fn legacy_importer_descriptor(capabilities: &[String]) -> AssetImporterDescriptor {
    let mut descriptor =
        AssetImporterDescriptor::new("fixture.importer", "Fixture importer", operation_path());
    descriptor
        .required_capabilities
        .extend(capabilities.iter().cloned());
    descriptor.required_capabilities.sort();
    descriptor.required_capabilities.dedup();
    descriptor
}

fn optimized_menu_descriptor(capabilities: &[String]) -> EditorMenuItemDescriptor {
    EditorMenuItemDescriptor::for_operation(operation_path())
        .with_required_capabilities(capabilities.iter().cloned())
}

fn optimized_importer_descriptor(capabilities: &[String]) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new("fixture.importer", "Fixture importer", operation_path())
        .with_required_capabilities(capabilities.iter().cloned())
}

fn extension_fixture() -> Vec<String> {
    (0..EXTENSION_COUNT)
        .rev()
        .map(|index| format!("extension.{:04}", index % (EXTENSION_COUNT / 2)))
        .collect()
}

fn legacy_repair_extension(extensions: &mut Vec<String>, extension: &str) {
    let extension = extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    if extension.is_empty() {
        return;
    }
    if !extensions.windows(2).all(|pair| pair[0] <= pair[1]) {
        extensions.sort();
        extensions.dedup();
    }
    if let Err(index) = extensions.binary_search(&extension) {
        extensions.insert(index, extension);
    }
}

#[test]
fn editor06_contribution_descriptors_capability_normalization_preserves_order() {
    let capabilities = capability_fixture();
    let legacy_menu = legacy_menu_descriptor(&capabilities);
    let optimized_menu = optimized_menu_descriptor(&capabilities);
    let legacy_importer = legacy_importer_descriptor(&capabilities);
    let optimized_importer = optimized_importer_descriptor(&capabilities);

    assert_eq!(
        optimized_menu.required_capabilities(),
        legacy_menu.required_capabilities()
    );
    assert_eq!(
        optimized_importer.required_capabilities(),
        legacy_importer.required_capabilities()
    );
    assert_eq!(
        optimized_menu.required_capabilities().len(),
        CAPABILITY_COUNT / 2
    );
    assert!(optimized_menu
        .required_capabilities()
        .windows(2)
        .all(|window| window[0] < window[1]));
}

#[test]
fn editor06_contribution_descriptors_extension_repair_preserves_order() {
    let mut legacy = extension_fixture();
    let mut optimized = legacy.clone();
    legacy_repair_extension(&mut legacy, "extension.new");
    push_normalized_extension(&mut optimized, "extension.new");

    assert_eq!(optimized, legacy);
    assert!(optimized.windows(2).all(|window| window[0] < window[1]));
}

#[test]
fn editor06_contribution_descriptors_source_contracts() {
    let source = include_str!("../contribution_descriptors.rs");
    assert_eq!(source.matches("sort_unstable();").count(), 3);
    assert!(!source.contains("extensions.sort();"));
    assert!(!source.contains("self.required_capabilities.sort();"));
    assert!(source.matches("size_hint()").count() >= 2);
    assert!(source.matches("reserve(lower_bound)").count() >= 2);
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor06_contribution_descriptors_capability_normalization_bench() {
    let capabilities = capability_fixture();
    let legacy = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..CAPABILITY_ITERATIONS {
                black_box((
                    legacy_menu_descriptor(&capabilities),
                    legacy_importer_descriptor(&capabilities),
                ));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let optimized = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..CAPABILITY_ITERATIONS {
                black_box((
                    optimized_menu_descriptor(&capabilities),
                    optimized_importer_descriptor(&capabilities),
                ));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "EDITOR06_UNSTABLE_CONTRIBUTION_CAPABILITY_SORT_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} capabilities={} unique_capabilities={} stable_sorts=2->0 preallocation=0->{}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        CAPABILITY_ITERATIONS,
        CAPABILITY_COUNT,
        CAPABILITY_COUNT / 2,
        CAPABILITY_COUNT,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor06_contribution_descriptors_extension_repair_bench() {
    let legacy = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..EXTENSION_ITERATIONS {
                let mut extensions = extension_fixture();
                legacy_repair_extension(&mut extensions, "extension.new");
                black_box(extensions);
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let optimized = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..EXTENSION_ITERATIONS {
                let mut extensions = extension_fixture();
                push_normalized_extension(&mut extensions, "extension.new");
                black_box(extensions);
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "EDITOR06_UNSTABLE_EXTENSION_NORMALIZATION_SORT_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} extensions={} unique_extensions={} stable_sorts=1->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        EXTENSION_ITERATIONS,
        EXTENSION_COUNT,
        EXTENSION_COUNT / 2,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}
