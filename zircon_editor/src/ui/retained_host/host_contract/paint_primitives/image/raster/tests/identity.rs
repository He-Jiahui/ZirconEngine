use std::hint::black_box;
use std::time::Instant;

#[test]
fn optimization_batch_20260830et_editor555_advances_identity_rows_by_stride() {
    let production = include_str!("../identity.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("identity image production source");

    assert!(production.contains("source_start += source_stride"));
    assert!(production.contains("destination_start += destination_stride"));
    assert!(!production.contains("source_y0 + row"));
    assert!(!production.contains("target.y0 as usize + row"));
}

#[test]
#[ignore = "deterministic performance marker"]
fn optimization_batch_20260830et_editor555_identity_row_stride_benchmark() {
    const HEIGHT: usize = 65_536;
    const SAMPLES: usize = 9;
    let image_width = black_box(257_usize);
    let frame_width = black_box(521_usize);
    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for _ in 0..SAMPLES {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for row in 0..HEIGHT {
            let source = ((17 + row) * image_width + 3) * 4;
            let destination = ((29 + row) * frame_width + 7) * 4;
            checksum ^= source ^ destination;
        }
        black_box(checksum);
        legacy_samples.push(started.elapsed());

        let started = Instant::now();
        let source_stride = image_width * 4;
        let destination_stride = frame_width * 4;
        let mut source = (17 * image_width + 3) * 4;
        let mut destination = (29 * frame_width + 7) * 4;
        let mut checksum = 0_usize;
        for _ in 0..HEIGHT {
            checksum ^= source ^ destination;
            source += source_stride;
            destination += destination_stride;
        }
        black_box(checksum);
        optimized_samples.push(started.elapsed());
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    println!(
        "EDITOR555_IDENTITY_ROW_STRIDE_BENCH_V1 legacy={:?} optimized={:?}",
        legacy_samples[SAMPLES / 2],
        optimized_samples[SAMPLES / 2]
    );
}
