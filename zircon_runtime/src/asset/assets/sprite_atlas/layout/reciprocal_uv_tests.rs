use std::hint::black_box;
use std::time::Instant;

use super::{SpriteAtlasRect, SpriteAtlasUvRect, SpriteAtlasValidationError};

const CHECKS_PER_SAMPLE: usize = 1_000_000;
const SAMPLE_PAIRS: usize = 31;

fn legacy_from_pixel_rect(
    rect: SpriteAtlasRect,
    atlas_width: u32,
    atlas_height: u32,
) -> Result<SpriteAtlasUvRect, SpriteAtlasValidationError> {
    if atlas_width == 0 || atlas_height == 0 {
        return Err(SpriteAtlasValidationError::ZeroAtlasDimensions {
            width: atlas_width,
            height: atlas_height,
        });
    }
    if rect.width == 0 || rect.height == 0 {
        return Err(SpriteAtlasValidationError::ZeroEntryDimensions {
            name: None,
            width: rect.width,
            height: rect.height,
        });
    }
    let rect_max_x =
        rect.x
            .checked_add(rect.width)
            .ok_or(SpriteAtlasValidationError::PixelRectOutOfBounds {
                name: None,
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
                atlas_width,
                atlas_height,
            })?;
    let rect_max_y = rect.y.checked_add(rect.height).ok_or(
        SpriteAtlasValidationError::PixelRectOutOfBounds {
            name: None,
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            atlas_width,
            atlas_height,
        },
    )?;
    if rect_max_x > atlas_width || rect_max_y > atlas_height {
        return Err(SpriteAtlasValidationError::PixelRectOutOfBounds {
            name: None,
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            atlas_width,
            atlas_height,
        });
    }
    let atlas_width = atlas_width as f32;
    let atlas_height = atlas_height as f32;
    Ok(SpriteAtlasUvRect {
        min: [rect.x as f32 / atlas_width, rect.y as f32 / atlas_height],
        max: [
            rect_max_x as f32 / atlas_width,
            rect_max_y as f32 / atlas_height,
        ],
    })
}

fn measure(rect: SpriteAtlasRect, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut evidence = 0.0_f32;
    for _ in 0..CHECKS_PER_SAMPLE {
        let uv = if optimized {
            SpriteAtlasUvRect::from_pixel_rect(black_box(rect), 4096, 2048)
        } else {
            legacy_from_pixel_rect(black_box(rect), 4096, 2048)
        }
        .expect("valid benchmark rect");
        evidence += uv.max[0];
        black_box(uv);
    }
    black_box(evidence);
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

#[test]
fn optimization_batch_20260829by_runtime352_reciprocal_uv_preserves_results() {
    for rect in [
        SpriteAtlasRect {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        },
        SpriteAtlasRect {
            x: 16,
            y: 8,
            width: 32,
            height: 16,
        },
        SpriteAtlasRect {
            x: 96,
            y: 48,
            width: 32,
            height: 16,
        },
    ] {
        assert_eq!(
            SpriteAtlasUvRect::from_pixel_rect(rect, 128, 64),
            legacy_from_pixel_rect(rect, 128, 64)
        );
    }
}

#[test]
fn optimization_batch_20260829by_runtime352_uv_uses_two_reciprocals() {
    let source = include_str!("../layout.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    let function = production
        .split_once("pub fn from_pixel_rect")
        .expect("UV constructor")
        .1;
    assert_eq!(function.matches("1.0 /").count(), 2);
    assert!(function.contains("* inverse_atlas_width"));
    assert!(function.contains("* inverse_atlas_height"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829by_runtime352_reciprocal_uv_bench() {
    let rect = SpriteAtlasRect {
        x: 512,
        y: 256,
        width: 1024,
        height: 512,
    };
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(rect, false));
            candidate.push(measure(rect, true));
        } else {
            candidate.push(measure(rect, true));
            baseline.push(measure(rect, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME352_RECIPROCAL_SPRITE_UV_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} baseline_divisions=4 candidate_divisions=2 candidate_multiplications=4 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
