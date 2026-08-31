use std::collections::HashSet;

use zircon_runtime::core::framework::net::{NetDownloadId, NetDownloadProgress};

use super::NetContentDownloadRuntimeManager;

const LINEAR_COMPLETION_LOOKUP_LIMIT: usize = 8;

impl NetContentDownloadRuntimeManager {
    pub fn store_resume_bitmap(
        &self,
        download: NetDownloadId,
        completed_chunks: impl IntoIterator<Item = bool>,
    ) {
        self.state()
            .resume_bitmaps
            .insert(download, completed_chunks.into_iter().collect());
    }

    pub fn resume_bitmap(&self, download: NetDownloadId) -> Vec<bool> {
        let state = self.state();
        if let Some(bitmap) = state.resume_bitmaps.get(&download) {
            return bitmap.clone();
        }
        let Some(manifest) = state.manifests.get(&download) else {
            return Vec::new();
        };
        let completed = state
            .progress
            .get(&download)
            .map(|progress| progress.completed_chunks.as_slice())
            .unwrap_or_default();
        indexed_completion_bitmap(
            manifest.chunks.iter().map(|chunk| chunk.id.as_str()),
            completed,
        )
    }

    pub fn apply_resume_bitmap(&self, download: NetDownloadId) -> Option<NetDownloadProgress> {
        let mut state = self.state();
        apply_resume_bitmap_to_state(&mut state, download)
    }
}

fn apply_resume_bitmap_to_state(
    state: &mut super::state::NetContentDownloadRuntimeState,
    download: NetDownloadId,
) -> Option<NetDownloadProgress> {
    state.progress.get(&download)?;
    let completed_chunks = {
        let manifest = state.manifests.get(&download)?;
        let bitmap = state.resume_bitmaps.get(&download)?;
        manifest
            .chunks
            .iter()
            .zip(bitmap.iter())
            .filter_map(|(chunk, completed)| completed.then(|| (chunk.id.clone(), chunk.byte_len)))
            .collect::<Vec<_>>()
    };
    if completed_chunks.is_empty() {
        return state.progress.get(&download).cloned();
    }

    let missing_cache_hits = {
        let cache_hits = state.cache_hits.get(&download);
        completed_chunks
            .iter()
            .filter(|(chunk_id, _)| {
                cache_hits
                    .is_none_or(|cache_hits| !cache_hits.iter().any(|cached| cached == chunk_id))
            })
            .map(|(chunk_id, _)| chunk_id.clone())
            .collect::<Vec<_>>()
    };
    state
        .cache_hits
        .entry(download)
        .or_default()
        .extend(missing_cache_hits);

    let progress = state.progress.get_mut(&download)?;
    let missing_progress_chunks = {
        let existing = &progress.completed_chunks;
        completed_chunks
            .iter()
            .filter(|(chunk_id, _)| !existing.iter().any(|current| current == chunk_id))
            .cloned()
            .collect::<Vec<_>>()
    };
    for (chunk_id, byte_len) in missing_progress_chunks {
        progress.completed_chunks.push(chunk_id);
        progress.downloaded_bytes += byte_len;
    }
    progress.status = if progress.downloaded_bytes >= progress.total_bytes {
        zircon_runtime::core::framework::net::NetDownloadStatus::Complete
    } else {
        zircon_runtime::core::framework::net::NetDownloadStatus::Downloading
    };
    Some(progress.clone())
}

impl super::state::NetContentDownloadRuntimeState {
    pub(in crate::manager) fn mark_resume_bitmap_chunk_complete(
        &mut self,
        download: NetDownloadId,
        chunk_id: &str,
    ) {
        let Some(manifest) = self.manifests.get(&download) else {
            return;
        };
        let Some(index) = manifest
            .chunks
            .iter()
            .position(|chunk| chunk.id == chunk_id)
        else {
            return;
        };
        let bitmap = self
            .resume_bitmaps
            .entry(download)
            .or_insert_with(|| vec![false; manifest.chunks.len()]);
        if index < bitmap.len() {
            bitmap[index] = true;
        }
    }
}

fn indexed_completion_bitmap<'a>(
    chunk_ids: impl IntoIterator<Item = &'a str>,
    completed: &[String],
) -> Vec<bool> {
    let chunk_ids = chunk_ids.into_iter();
    if completed.len() <= LINEAR_COMPLETION_LOOKUP_LIMIT {
        return chunk_ids
            .map(|chunk_id| completed.iter().any(|id| id == chunk_id))
            .collect();
    }

    let completed = completed.iter().map(String::as_str).collect::<HashSet<_>>();
    chunk_ids
        .map(|chunk_id| completed.contains(chunk_id))
        .collect()
}

#[cfg(test)]
mod completion_index_tests {
    use std::{hint::black_box, time::Instant};

    use super::indexed_completion_bitmap;

    const BENCHMARK_CHUNK_COUNT: usize = 4_096;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[test]
    fn indexed_completion_bitmap_preserves_manifest_order_and_duplicate_semantics() {
        let manifest = ["chunk-a", "chunk-b", "chunk-c", "chunk-d"];
        let completed = vec![
            "chunk-c".to_string(),
            "chunk-a".to_string(),
            "chunk-c".to_string(),
            "unrelated".to_string(),
        ];

        assert_eq!(
            indexed_completion_bitmap(manifest, &completed),
            vec![true, false, true, false]
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn indexed_completion_bitmap_release_benchmark_evidence() {
        let manifest = (0..BENCHMARK_CHUNK_COUNT)
            .map(|index| format!("chunk-{index:05}"))
            .collect::<Vec<_>>();
        let completed = manifest.iter().step_by(2).cloned().collect::<Vec<_>>();
        assert_eq!(
            legacy_completion_bitmap(&manifest, &completed),
            indexed_completion_bitmap(manifest.iter().map(String::as_str), &completed)
        );

        let legacy_string_comparisons = completed.len() * (completed.len() + 1) / 2
            + (manifest.len() - completed.len()) * completed.len();
        let optimized_hash_lookups = manifest.len();
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_legacy(&manifest, &completed));
                optimized_samples.push(measure_optimized(&manifest, &completed));
            } else {
                optimized_samples.push(measure_optimized(&manifest, &completed));
                legacy_samples.push(measure_legacy(&manifest, &completed));
            }
        }

        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        println!(
            "PERF_RESULT task=plugins10_indexed_resume_bitmap chunks={BENCHMARK_CHUNK_COUNT} completed_chunks={} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_string_comparisons_per_sample={legacy_string_comparisons} optimized_hash_lookups_per_sample={optimized_hash_lookups} threshold_percent=50 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={} optimized_raw_ns={}",
            completed.len(),
            raw_samples(&legacy_samples),
            raw_samples(&optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(50),
            "indexed resume bitmap P95 {optimized_p95}ns did not improve legacy {legacy_p95}ns by 50%"
        );
    }

    fn legacy_completion_bitmap(manifest: &[String], completed: &[String]) -> Vec<bool> {
        manifest
            .iter()
            .map(|chunk_id| completed.iter().any(|id| id == chunk_id))
            .collect()
    }

    fn measure_legacy(manifest: &[String], completed: &[String]) -> u128 {
        let start = Instant::now();
        let result = legacy_completion_bitmap(black_box(manifest), black_box(completed));
        let elapsed = start.elapsed().as_nanos();
        black_box(result);
        elapsed
    }

    fn measure_optimized(manifest: &[String], completed: &[String]) -> u128 {
        let start = Instant::now();
        let result = indexed_completion_bitmap(
            black_box(manifest).iter().map(String::as_str),
            black_box(completed),
        );
        let elapsed = start.elapsed().as_nanos();
        black_box(result);
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
}

#[cfg(test)]
mod batched_resume_apply_tests {
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
}
