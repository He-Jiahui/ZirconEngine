use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use super::*;

fn benchmark_request() -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(IblBakeKey::source_cubemap(17, [9, 8, 7, 6]), 1_024, 11)
        .with_pmrem_layout(256, 9)
}

#[test]
fn runtime_cache_writer_uses_runtime_atomic_publication() {
    let source = include_str!("../ibl_bake_artifact_cache.rs");
    let writer = source
        .split("pub fn write_runtime_cache(")
        .nth(1)
        .and_then(|writer| writer.split("pub fn read_runtime_cache(").next())
        .expect("runtime cache store must retain its writer");

    assert!(source.contains("core::resource::io::atomic_write"));
    assert!(writer.contains("atomic_write("));
    assert!(!writer.contains("fs::write("));
}

#[test]
fn optimization_batch_ek_preallocated_cache_path_preserves_layout() {
    let root = PathBuf::from("project-cache-root");
    let store = IblBakeArtifactCacheStore::new(&root);
    let request = benchmark_request();
    let source_hash = ibl_bake_artifact_request_identity_hash(&request);

    assert_eq!(
        store.runtime_cache_path(&request),
        root.join(IBL_BAKE_RUNTIME_CACHE_DIRECTORY)
            .join(format!("v{:016x}", IBL_BAKE_ALGORITHM_VERSION))
            .join(source_hash)
            .join(format!(
                "face_{:04}_mips_{:02}.{}",
                request.pmrem_face_size(),
                request.pmrem_mip_count(),
                IBL_BAKE_RUNTIME_CACHE_EXTENSION
            ))
    );
}

#[test]
fn optimization_batch_ek_cache_path_uses_one_preallocated_buffer() {
    let source = include_str!("../ibl_bake_artifact_cache.rs");
    let implementation = source
        .split("pub fn runtime_cache_path(")
        .nth(1)
        .expect("runtime cache path implementation")
        .split("pub fn runtime_cache_path_for_descriptor(")
        .next()
        .expect("bounded runtime cache path implementation");

    assert!(implementation.contains("let mut path = self.cache_root.clone()"));
    assert!(implementation.contains("path.reserve("));
    assert!(implementation.contains("path.push(IBL_BAKE_RUNTIME_CACHE_DIRECTORY)"));
    assert!(!implementation.contains("self.cache_root\n            .join("));
}

#[test]
#[ignore = "release-only preallocated IBL cache path benchmark"]
fn optimization_batch_ek_preallocated_ibl_cache_path_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    const PATHS_PER_SAMPLE: usize = 2_048;

    fn legacy_path(store: &IblBakeArtifactCacheStore, request: &IblBakeArtifactRequest) -> PathBuf {
        let source_hash = ibl_bake_artifact_request_identity_hash(request);
        store
            .cache_root()
            .join(IBL_BAKE_RUNTIME_CACHE_DIRECTORY)
            .join(format!("v{:016x}", IBL_BAKE_ALGORITHM_VERSION))
            .join(source_hash)
            .join(format!(
                "face_{:04}_mips_{:02}.{}",
                request.pmrem_face_size(),
                request.pmrem_mip_count(),
                IBL_BAKE_RUNTIME_CACHE_EXTENSION
            ))
    }

    fn measure_legacy(store: &IblBakeArtifactCacheStore, request: &IblBakeArtifactRequest) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..PATHS_PER_SAMPLE {
            let path = black_box(legacy_path(black_box(store), black_box(request)));
            checksum = checksum.wrapping_add(path.as_os_str().len());
            black_box(path);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(
        store: &IblBakeArtifactCacheStore,
        request: &IblBakeArtifactRequest,
    ) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..PATHS_PER_SAMPLE {
            let path = black_box(store.runtime_cache_path(black_box(request)));
            checksum = checksum.wrapping_add(path.as_os_str().len());
            black_box(path);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    let root = PathBuf::from(format!("C:\\{}cache", "ibl-cache-segment\\".repeat(256)));
    let store = IblBakeArtifactCacheStore::new(root);
    let request = benchmark_request();
    for _ in 0..4 {
        black_box(measure_legacy(&store, &request));
        black_box(measure_optimized(&store, &request));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy(&store, &request));
            optimized_samples.push(measure_optimized(&store, &request));
        } else {
            optimized_samples.push(measure_optimized(&store, &request));
            legacy_samples.push(measure_legacy(&store, &request));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME445_PREALLOCATED_IBL_CACHE_PATH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             paths_per_sample={PATHS_PER_SAMPLE} root_bytes={} \
             pair_order=alternating_legacy_even legacy_path_buffers_per_path=4 \
             optimized_path_buffers_per_path=1 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        store.cache_root().as_os_str().len(),
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(80),
        "preallocated IBL cache path construction must reduce P95 by at least 20%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
