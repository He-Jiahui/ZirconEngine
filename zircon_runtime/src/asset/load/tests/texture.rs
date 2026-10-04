use std::hint::black_box;
use std::time::Instant;

use super::{generate_checker_texture, generate_grid_texture, CpuTexturePayload};

#[test]
fn optimization_batch_20260830eu_runtime556_reuses_builtin_texture_row_templates() {
    let production = include_str!("../texture.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("texture production source");

    assert!(production.contains("checker_row_templates"));
    assert!(production.contains("grid_row_templates"));
    assert!(!production.contains("rgba[offset..offset + 4].copy_from_slice"));
}

#[test]
fn optimization_batch_20260830eu_runtime556_preserves_builtin_texture_patterns() {
    let checker = generate_checker_texture();
    assert_eq!(pixel(&checker, 0, 0), [220, 220, 220, 255]);
    assert_eq!(pixel(&checker, 15, 15), [220, 220, 220, 255]);
    assert_eq!(pixel(&checker, 16, 0), [40, 40, 40, 255]);
    assert_eq!(pixel(&checker, 16, 16), [220, 220, 220, 255]);

    let grid = generate_grid_texture();
    assert_eq!(pixel(&grid, 0, 7), [110, 150, 255, 255]);
    assert_eq!(pixel(&grid, 7, 0), [110, 150, 255, 255]);
    assert_eq!(pixel(&grid, 16, 7), [55, 65, 85, 255]);
    assert_eq!(pixel(&grid, 7, 16), [55, 65, 85, 255]);
    assert_eq!(pixel(&grid, 7, 7), [26, 30, 38, 255]);
}

#[test]
#[ignore = "deterministic performance marker"]
fn optimization_batch_20260830eu_runtime556_builtin_row_template_benchmark() {
    const SAMPLES: usize = 9;
    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for _ in 0..SAMPLES {
        let started = Instant::now();
        black_box(legacy_grid_texture());
        legacy_samples.push(started.elapsed());

        let started = Instant::now();
        black_box(generate_grid_texture());
        optimized_samples.push(started.elapsed());
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    println!(
        "RUNTIME556_BUILTIN_ROW_TEMPLATE_BENCH_V1 legacy={:?} optimized={:?}",
        legacy_samples[SAMPLES / 2],
        optimized_samples[SAMPLES / 2]
    );
}

fn legacy_grid_texture() -> Vec<u8> {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 256;
    let mut rgba = vec![0_u8; WIDTH * HEIGHT * 4];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let color = if x % 64 == 0 || y % 64 == 0 {
                [110, 150, 255, 255]
            } else if x % 16 == 0 || y % 16 == 0 {
                [55, 65, 85, 255]
            } else {
                [26, 30, 38, 255]
            };
            let offset = (y * WIDTH + x) * 4;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }
    rgba
}

fn pixel(payload: &CpuTexturePayload, x: usize, y: usize) -> [u8; 4] {
    let offset = (y * payload.width as usize + x) * 4;
    payload.rgba[offset..offset + 4]
        .try_into()
        .expect("RGBA pixel")
}
