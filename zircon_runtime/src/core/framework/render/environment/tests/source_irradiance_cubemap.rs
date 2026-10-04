use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::framework::render::environment::build_source_cubemap_from_equirect;
use crate::core::framework::tasks::ParallelSliceExecutor;

#[derive(Default)]
struct CountingParallelSliceExecutor {
    dispatches: std::sync::atomic::AtomicUsize,
    work_items: std::sync::atomic::AtomicUsize,
}

impl ParallelSliceExecutor for CountingParallelSliceExecutor {
    fn parallel_for<T, F>(&self, items: &mut [T], chunk_size: usize, task: F)
    where
        T: Send,
        F: Fn(&mut [T]) + Send + Sync,
    {
        self.dispatches
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.work_items.fetch_add(
            items.len().div_ceil(chunk_size.max(1)),
            std::sync::atomic::Ordering::Relaxed,
        );
        for chunk in items.chunks_mut(chunk_size.max(1)) {
            task(chunk);
        }
    }
}

#[test]
fn cloned_irradiance_cube_shares_immutable_texel_storage() {
    let cube = SourceCubemapIrradianceCube::new(1, vec![[0.25, 0.5, 0.75]; 6]);
    let cloned = cube.clone();

    assert!(std::sync::Arc::ptr_eq(&cube.texels, &cloned.texels));
}

#[test]
fn optimization_batch_20260830eu_runtime557_batches_irradiance_hash_bytes() {
    let production = include_str!("../source_irradiance_cubemap.rs");
    let hash_source = production
        .split("fn source_cubemap_irradiance_cube_content_hash")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub fn build_source_cubemap_irradiance_cube")
                .next()
        })
        .expect("irradiance hash source");

    assert!(production.contains("IRRADIANCE_HASH_TEXELS_PER_BATCH"));
    assert!(hash_source.contains("update_irradiance_texel_hash"));
    assert!(!hash_source.contains("hasher.update(&channel.to_bits().to_le_bytes())"));

    let texels = (0..257)
        .map(|index| {
            let value = index as Real * 0.03125 - 3.0;
            [value, -value, value * value]
        })
        .collect::<Vec<_>>();
    assert_eq!(
        source_cubemap_irradiance_cube_content_hash(19, &texels),
        legacy_content_hash(19, &texels)
    );
}

fn legacy_content_hash(face_size: u32, texels: &[[Real; 3]]) -> [u32; 4] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&face_size.to_le_bytes());
    for texel in texels {
        for channel in texel {
            hasher.update(&channel.to_bits().to_le_bytes());
        }
    }
    let bytes = hasher.finalize();
    let bytes = bytes.as_bytes();
    [
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
    ]
}

#[test]
#[ignore = "deterministic performance marker"]
fn optimization_batch_20260830eu_runtime557_irradiance_hash_batch_benchmark() {
    const TEXEL_COUNT: usize = 32 * 32 * 6;
    const SAMPLES: usize = 9;
    let texels = (0..TEXEL_COUNT)
        .map(|index| {
            let value = index as Real * 0.000_976_562_5;
            [value, value * 0.5, 1.0 - value]
        })
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for _ in 0..SAMPLES {
        let started = Instant::now();
        black_box(legacy_content_hash(32, &texels));
        legacy_samples.push(started.elapsed());

        let started = Instant::now();
        black_box(source_cubemap_irradiance_cube_content_hash(32, &texels));
        optimized_samples.push(started.elapsed());
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    println!(
        "RUNTIME557_IRRADIANCE_HASH_BATCH_BENCH_V1 legacy={:?} optimized={:?}",
        legacy_samples[SAMPLES / 2],
        optimized_samples[SAMPLES / 2]
    );
}

#[test]
fn parallel_irradiance_cube_matches_serial_output_and_tiles_face_rows_for_executor() {
    let source =
        build_source_cubemap_from_equirect(4, |u, v| [u, v, (u * 0.75 + v * 0.25).fract(), 1.0]);
    let serial = build_source_cubemap_irradiance_cube(&source);
    let executor = CountingParallelSliceExecutor::default();
    let parallel = build_source_cubemap_irradiance_cube_with_parallel_executor(&source, &executor);

    assert_eq!(parallel, serial);
    assert_eq!(
        executor
            .dispatches
            .load(std::sync::atomic::Ordering::Relaxed),
        1,
        "IEM convolution must submit all independent output rows through one caller-owned executor dispatch"
    );
    assert_eq!(
        executor
            .work_items
            .load(std::sync::atomic::Ordering::Relaxed),
        48,
        "32x32 IEM output must use four-row tiles so a direct convolution can use more than six worker tasks"
    );
}
