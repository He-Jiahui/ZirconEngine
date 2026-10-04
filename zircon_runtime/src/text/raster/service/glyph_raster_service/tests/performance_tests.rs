use std::hint::black_box;
use std::sync::Arc;
use std::time::{Duration, Instant};

use swash::FontRef;

use super::{GlyphRasterService, RuntimeGlyphRasterFace};
use crate::core::framework::text::{
    TextFontCollectionHandle, TextFontFaceHandle, TextGlyphRasterHinting, TextGlyphRasterMode,
    TextGlyphRasterRequest,
};

const SAMPLE_COUNT: usize = 31;
const ITERATIONS_PER_SAMPLE: usize = 1_024;
const TEST_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/FiraSans-Regular.ttf"
));
const TEST_COLLECTION: TextFontCollectionHandle = TextFontCollectionHandle::new(73);
const TEST_FACE: TextFontFaceHandle = TextFontFaceHandle::new(TEST_COLLECTION, 11, 23);

#[test]
#[ignore = "managed Windows release performance evidence"]
fn text_runtime_raster_authority_sync_service_repeated_request_profile() {
    const MARKER: &str = "TEXT04_SYNC_RASTER_REPEATED_REQUEST_PROFILE_V1";
    let font = FontRef::from_index(TEST_FONT_BYTES, 0).expect("test font should parse");
    let glyph_id = u32::from(font.charmap().map('P'));
    assert_ne!(glyph_id, 0, "test glyph should be present in Fira Sans");
    let request = TextGlyphRasterRequest::new(glyph_id, 18, TextGlyphRasterMode::Outline)
        .with_hinting(TextGlyphRasterHinting::Full)
        .with_subpixel_position(20.8, 4.6);
    let service = GlyphRasterService::new();

    let cold_started = Instant::now();
    let reference = service
        .rasterize(test_face(), request)
        .expect("profile glyph should rasterize");
    let cold_nanos = cold_started.elapsed().as_nanos();
    let mut raster_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut receipt_clone_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            raster_samples.push(measure_raster_batch(&service, request));
            receipt_clone_samples.push(measure_receipt_clone_batch(&reference));
        } else {
            receipt_clone_samples.push(measure_receipt_clone_batch(&reference));
            raster_samples.push(measure_raster_batch(&service, request));
        }
    }

    let raster = percentiles(&raster_samples);
    let receipt_clone = percentiles(&receipt_clone_samples);
    let p95_ratio = raster.p95 as f64 / receipt_clone.p95.max(1) as f64;
    eprintln!(
        "{MARKER} cold_ns={cold_nanos} raster_p50_ns={} raster_p95_ns={} raster_p99_ns={} receipt_clone_p50_ns={} receipt_clone_p95_ns={} receipt_clone_p99_ns={} p95_ratio={p95_ratio:.3} bitmap_bytes={} samples={SAMPLE_COUNT} iterations_per_sample={ITERATIONS_PER_SAMPLE}",
        raster.p50,
        raster.p95,
        raster.p99,
        receipt_clone.p50,
        receipt_clone.p95,
        receipt_clone.p99,
        reference.bitmap.len(),
    );

    assert_eq!(reference.request, request);
    assert!(!reference.bitmap.is_empty());
    assert_eq!(raster.sample_count, SAMPLE_COUNT);
    assert_eq!(receipt_clone.sample_count, SAMPLE_COUNT);
}

fn test_face() -> RuntimeGlyphRasterFace<'static> {
    RuntimeGlyphRasterFace {
        font_collection: TEST_COLLECTION,
        font_face: TEST_FACE,
        font_instance: None,
        font_generation: 23,
        source_identity: [0x73; 16],
        bytes: TEST_FONT_BYTES,
        collection_index: 0,
        variations: None,
    }
}

fn measure_raster_batch(service: &GlyphRasterService, request: TextGlyphRasterRequest) -> Duration {
    let started = Instant::now();
    for _ in 0..ITERATIONS_PER_SAMPLE {
        let receipt = service
            .rasterize(test_face(), request)
            .expect("profile glyph should rasterize");
        black_box(receipt.bitmap.len());
    }
    per_iteration(started.elapsed())
}

fn measure_receipt_clone_batch(
    receipt: &crate::core::framework::text::TextGlyphRasterReceipt,
) -> Duration {
    let started = Instant::now();
    for _ in 0..ITERATIONS_PER_SAMPLE {
        black_box(Arc::clone(&receipt.bitmap));
    }
    per_iteration(started.elapsed())
}

fn per_iteration(elapsed: Duration) -> Duration {
    let nanos = elapsed.as_nanos() / ITERATIONS_PER_SAMPLE as u128;
    Duration::from_nanos(u64::try_from(nanos).unwrap_or(u64::MAX))
}

#[derive(Clone, Copy)]
struct Percentiles {
    sample_count: usize,
    p50: u128,
    p95: u128,
    p99: u128,
}

fn percentiles(samples: &[Duration]) -> Percentiles {
    let mut nanos = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    nanos.sort_unstable();
    Percentiles {
        sample_count: nanos.len(),
        p50: percentile(&nanos, 50),
        p95: percentile(&nanos, 95),
        p99: percentile(&nanos, 99),
    }
}

fn percentile(sorted: &[u128], percentile: usize) -> u128 {
    sorted[(sorted.len() - 1) * percentile / 100]
}
