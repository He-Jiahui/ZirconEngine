use super::*;

fn summary_growing(metadata: &UiDragSourceMetadata) -> Option<String> {
    match (&metadata.asset_kind, &metadata.display_name) {
        (Some(kind), Some(name)) if !kind.is_empty() && !name.is_empty() => {
            Some(format!("{kind}: {name}"))
        }
        (_, Some(name)) if !name.is_empty() => Some(name.clone()),
        (_, _) => metadata
            .locator
            .clone()
            .filter(|locator| !locator.is_empty()),
    }
}

#[test]
fn runtime_interface03_batch51_52_presized_drag_summary_preserves_growing_results() {
    for metadata in [
        UiDragSourceMetadata {
            asset_kind: Some("Texture".to_string()),
            display_name: Some("Albedo".to_string()),
            locator: Some("res://textures/albedo.png".to_string()),
            ..UiDragSourceMetadata::default()
        },
        UiDragSourceMetadata {
            asset_kind: Some(String::new()),
            display_name: Some("Fallback name".to_string()),
            locator: Some("res://fallback".to_string()),
            ..UiDragSourceMetadata::default()
        },
        UiDragSourceMetadata {
            display_name: Some(String::new()),
            locator: Some("res://locator".to_string()),
            ..UiDragSourceMetadata::default()
        },
        UiDragSourceMetadata::default(),
    ] {
        assert_eq!(metadata.summary(), summary_growing(&metadata));
    }
}

fn p95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[test]
#[ignore = "release-only presized drag summary benchmark"]
fn runtime_interface03_batch51_52_presized_drag_summary_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const SUMMARY_COUNT: usize = 200_000;
    const SAMPLE_COUNT: usize = 11;
    let metadata = UiDragSourceMetadata {
        asset_kind: Some("VirtualTextureLayer".repeat(8)),
        display_name: Some("EnvironmentAlbedoVariant".repeat(12)),
        ..UiDragSourceMetadata::default()
    };
    let mut growing_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut presized_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_growing = || {
            let started = Instant::now();
            for _ in 0..SUMMARY_COUNT {
                black_box(summary_growing(black_box(&metadata)));
            }
            started.elapsed().as_nanos()
        };
        let measure_presized = || {
            let started = Instant::now();
            for _ in 0..SUMMARY_COUNT {
                black_box(metadata.summary());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            growing_samples.push(measure_growing());
            presized_samples.push(measure_presized());
        } else {
            presized_samples.push(measure_presized());
            growing_samples.push(measure_growing());
        }
    }

    let growing_p95_ns = p95(growing_samples);
    let presized_p95_ns = p95(presized_samples);
    eprintln!(
        "RUNTIME_INTERFACE03_PRESIZED_DRAG_SUMMARY_BENCH_V1 summaries={SUMMARY_COUNT} kind_bytes={} name_bytes={} samples={SAMPLE_COUNT} growing_p95_ns={growing_p95_ns} presized_p95_ns={presized_p95_ns}",
        metadata.asset_kind.as_deref().map_or(0, str::len),
        metadata.display_name.as_deref().map_or(0, str::len),
    );
    assert!(
        presized_p95_ns.saturating_mul(5) <= growing_p95_ns.saturating_mul(4),
        "presized drag summary must improve P95 by at least 20%: growing={growing_p95_ns}ns presized={presized_p95_ns}ns",
    );
}
