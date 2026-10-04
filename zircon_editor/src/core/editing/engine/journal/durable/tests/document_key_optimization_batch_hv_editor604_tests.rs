use std::hint::black_box;
use std::path::Path;
use std::time::{Duration, Instant};

use super::{normalize_project_relative_path, JournalDocumentKey};

#[test]
fn optimization_batch_hv_editor604_streaming_normalization_preserves_identity() {
    let forward = JournalDocumentKey::from_project_relative_path(Path::new(
        "assets/scenes/world/main.zscene",
    ))
    .unwrap();
    let mixed = JournalDocumentKey::from_project_relative_path(Path::new(
        r"assets\\scenes//world\main.zscene",
    ))
    .unwrap();

    assert_eq!(mixed, forward);
    assert_eq!(
        mixed.source_path(),
        Path::new("assets/scenes/world/main.zscene")
    );
    assert_eq!(
        normalize_project_relative_path("assets//ui/"),
        Some("assets/ui".to_owned())
    );
    assert_eq!(normalize_project_relative_path("assets/./ui"), None);
    assert_eq!(normalize_project_relative_path("assets/../ui"), None);
}

#[test]
fn optimization_batch_hv_editor604_normalization_avoids_component_collection() {
    let source = include_str!("../document_key.rs");
    let normalization = source
        .split("fn normalize_project_relative_path")
        .nth(1)
        .expect("path normalization")
        .split("#[derive(Debug, thiserror::Error)]")
        .next()
        .expect("path normalization body");

    assert!(!normalization.contains("collect::<Vec"));
    assert!(!normalization.contains("normalized.join"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hv_editor604_streaming_normalization_p95() {
    const MARKER: &str = "EDITOR604_JOURNAL_DOCUMENT_KEY_STREAMING_NORMALIZATION_BENCH_V1";
    const SAMPLE_PAIRS: usize = 21;
    const ITERATIONS: usize = 4_096;
    let source = (0..96)
        .map(|index| format!("component-{index:03}"))
        .collect::<Vec<_>>()
        .join(r"\\//");
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&source, ITERATIONS, false));
            optimized.push(measure(&source, ITERATIONS, true));
        } else {
            optimized.push(measure(&source, ITERATIONS, true));
            legacy.push(measure(&source, ITERATIONS, false));
        }
    }

    let legacy_p95_ns = percentile_ns(&legacy, 95);
    let optimized_p95_ns = percentile_ns(&optimized, 95);
    let ratio = optimized_p95_ns as f64 / legacy_p95_ns.max(1) as f64;
    eprintln!(
        "{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} ratio={ratio:.4} samples={SAMPLE_PAIRS} iterations={ITERATIONS} components=96 allocations=2->1"
    );
    assert!(
        ratio <= 0.70,
        "{MARKER} expected streaming normalization ratio <= 0.70, got {ratio:.4}"
    );
}

fn measure(source: &str, iterations: usize, optimized: bool) -> Duration {
    let started = Instant::now();
    for _ in 0..iterations {
        let normalized = if optimized {
            normalize_project_relative_path(black_box(source))
        } else {
            legacy_normalize(black_box(source))
        };
        black_box(normalized);
    }
    started.elapsed()
}

fn legacy_normalize(source: &str) -> Option<String> {
    let bytes = source.as_bytes();
    let has_drive_prefix = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    if source.is_empty() || source.starts_with(['/', '\\']) || has_drive_prefix {
        return None;
    }
    let components = source
        .split(['/', '\\'])
        .filter(|component| !component.is_empty())
        .collect::<Vec<_>>();
    if components.is_empty()
        || components
            .iter()
            .any(|component| matches!(*component, "." | ".."))
    {
        return None;
    }
    Some(components.join("/"))
}

fn percentile_ns(samples: &[Duration], percentile: usize) -> u128 {
    let mut values = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    values.sort_unstable();
    values[(values.len() - 1) * percentile / 100]
}
