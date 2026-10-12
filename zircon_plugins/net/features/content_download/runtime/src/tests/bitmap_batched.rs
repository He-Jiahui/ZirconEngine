use std::{hint::black_box, time::Instant};

use zircon_runtime::core::framework::net::{
    NetDownloadChunk, NetDownloadId, NetDownloadManifest, NetDownloadProgress,
    NetDownloadStatus,
};

use super::NetContentDownloadRuntimeManager;

const BENCHMARK_CHUNK_COUNT: usize = 512;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn batched_resume_apply_preserves_existing_progress_and_manifest_order() {
    let manager = NetContentDownloadRuntimeManager::new();
    let download = NetDownloadId::new(91);
    manager.queue_manifest(test_manifest(download, 3));
    manager
        .mark_cache_hit(download, "chunk-00001")
        .expect("existing completed chunk should be recorded");
    manager.store_resume_bitmap(download, [true, false, true]);

    let progress = manager
        .apply_resume_bitmap(download)
        .expect("resume bitmap should apply");

    assert_eq!(progress.status, NetDownloadStatus::Complete);
    assert_eq!(progress.downloaded_bytes, 3);
    assert_eq!(
        progress.completed_chunks,
        vec![
            "chunk-00001".to_string(),
            "chunk-00000".to_string(),
            "chunk-00002".to_string(),
        ]
    );
    assert_eq!(
        manager.cache_hits(download),
        vec![
            "chunk-00001".to_string(),
            "chunk-00000".to_string(),
            "chunk-00002".to_string(),
        ]
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn batched_resume_apply_release_benchmark_evidence() {
    let legacy_equivalence = benchmark_manager();
    let optimized_equivalence = benchmark_manager();
    let download = NetDownloadId::new(92);
    assert_eq!(
        legacy_apply_resume_bitmap(&legacy_equivalence, download),
        optimized_equivalence.apply_resume_bitmap(download)
    );
    assert_eq!(
        legacy_equivalence.cache_hits(download),
        optimized_equivalence.cache_hits(download)
    );

    let legacy_lock_acquisitions = BENCHMARK_CHUNK_COUNT + 2;
    let optimized_lock_acquisitions = 1;
    let legacy_progress_clones = BENCHMARK_CHUNK_COUNT + 1;
    let optimized_progress_clones = 1;
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy(download));
            optimized_samples.push(measure_optimized(download));
        } else {
            optimized_samples.push(measure_optimized(download));
            legacy_samples.push(measure_legacy(download));
        }
    }

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT task=plugins10_batched_resume_bitmap_apply chunks={BENCHMARK_CHUNK_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_lock_acquisitions_per_sample={legacy_lock_acquisitions} optimized_lock_acquisitions_per_sample={optimized_lock_acquisitions} legacy_progress_clones_per_sample={legacy_progress_clones} optimized_progress_clones_per_sample={optimized_progress_clones} threshold_percent=50 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={} optimized_raw_ns={}",
        raw_samples(&legacy_samples),
        raw_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(50),
        "batched resume apply P95 {optimized_p95}ns did not improve legacy {legacy_p95}ns by 50%"
    );
}

fn test_manifest(download: NetDownloadId, chunk_count: usize) -> NetDownloadManifest {
    (0..chunk_count).fold(
        NetDownloadManifest::new(download, "asset://benchmark/resume"),
        |manifest, index| {
            manifest.with_chunk(NetDownloadChunk::new(
                format!("chunk-{index:05}"),
                format!("https://cdn.example/chunk-{index:05}"),
                index as u64,
                1,
                [index as u8; 32],
            ))
        },
    )
}

fn benchmark_manager() -> NetContentDownloadRuntimeManager {
    let manager = NetContentDownloadRuntimeManager::new();
    let download = NetDownloadId::new(92);
    manager.queue_manifest(test_manifest(download, BENCHMARK_CHUNK_COUNT));
    manager.store_resume_bitmap(download, std::iter::repeat_n(true, BENCHMARK_CHUNK_COUNT));
    manager
}

fn legacy_apply_resume_bitmap(
    manager: &NetContentDownloadRuntimeManager,
    download: NetDownloadId,
) -> Option<NetDownloadProgress> {
    let chunk_ids = {
        let state = manager.state();
        let manifest = state.manifests.get(&download)?;
        let bitmap = state.resume_bitmaps.get(&download)?;
        manifest
            .chunks
            .iter()
            .zip(bitmap.iter())
            .filter_map(|(chunk, completed)| completed.then(|| chunk.id.clone()))
            .collect::<Vec<_>>()
    };

    let mut progress = manager.progress(download)?;
    for chunk_id in chunk_ids {
        progress = manager.mark_cache_hit(download, &chunk_id)?;
    }
    Some(progress)
}

fn measure_legacy(download: NetDownloadId) -> u128 {
    let manager = benchmark_manager();
    let start = Instant::now();
    let progress = legacy_apply_resume_bitmap(black_box(&manager), download);
    let elapsed = start.elapsed().as_nanos();
    black_box(progress);
    elapsed
}

fn measure_optimized(download: NetDownloadId) -> u128 {
    let manager = benchmark_manager();
    let start = Instant::now();
    let progress = black_box(&manager).apply_resume_bitmap(download);
    let elapsed = start.elapsed().as_nanos();
    black_box(progress);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw_samples(samples: &[u128]) -> String {
    format!(
        "[{}]",
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )
}
