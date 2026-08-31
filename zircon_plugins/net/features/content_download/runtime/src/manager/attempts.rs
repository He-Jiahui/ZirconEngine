use zircon_runtime::core::framework::net::{
    NetDownloadAttemptDescriptor, NetDownloadChunk, NetDownloadId, NetDownloadManifest,
    NetDownloadProgress, NetDownloadStatus,
};

use super::NetContentDownloadRuntimeManager;

impl NetContentDownloadRuntimeManager {
    pub fn candidate_urls(&self, download: NetDownloadId, chunk_id: &str) -> Option<Vec<String>> {
        let state = self.state();
        let manifest = state.manifests.get(&download)?;
        let chunk = manifest.chunks.iter().find(|chunk| chunk.id == chunk_id)?;
        Some(candidate_urls_for_chunk(manifest, chunk))
    }

    pub fn next_attempt(
        &self,
        download: NetDownloadId,
        chunk_id: &str,
    ) -> Option<NetDownloadAttemptDescriptor> {
        let state = self.state();
        let manifest = state.manifests.get(&download)?;
        let chunk = manifest.chunks.iter().find(|chunk| chunk.id == chunk_id)?;
        let key = (download, chunk_id.to_string());
        let attempt_index = state.attempt_indices.get(&key).copied().unwrap_or_default();
        let url = candidate_url_for_attempt(manifest, chunk, attempt_index)?;
        Some(attempt_descriptor_for_chunk(
            download,
            chunk,
            url,
            attempt_index,
        ))
    }

    pub fn mark_attempt_failed(
        &self,
        download: NetDownloadId,
        chunk_id: &str,
        diagnostic: impl Into<String>,
    ) -> Option<NetDownloadProgress> {
        let mut state = self.state();
        let manifest = state.manifests.get(&download)?;
        manifest.chunks.iter().find(|chunk| chunk.id == chunk_id)?;
        let candidate_count = 1 + manifest.mirror_urls.len();
        let key = (download, chunk_id.to_string());
        let attempt_index = state.attempt_indices.get(&key).copied().unwrap_or_default();
        state
            .failed_attempts
            .entry(key.clone())
            .or_default()
            .push(diagnostic.into());
        let next_attempt_index = attempt_index.saturating_add(1);
        let exhausted = next_attempt_index >= candidate_count;
        let next_attempt_index = if exhausted {
            candidate_count
        } else {
            next_attempt_index
        };
        state.attempt_indices.insert(key, next_attempt_index);
        let progress = state.progress.get_mut(&download)?;
        if exhausted {
            progress.status = NetDownloadStatus::Failed;
            progress.diagnostic = Some(format!("chunk attempts exhausted: {chunk_id}"));
        } else {
            progress.status = NetDownloadStatus::Downloading;
            progress.diagnostic = Some(format!(
                "chunk attempt failed, switching mirror: {chunk_id}"
            ));
        }
        Some(progress.clone())
    }

    pub fn failed_attempts(&self, download: NetDownloadId, chunk_id: &str) -> Vec<String> {
        self.state()
            .failed_attempts
            .get(&(download, chunk_id.to_string()))
            .cloned()
            .unwrap_or_default()
    }
}

pub(in crate::manager) fn candidate_urls_for_chunk(
    manifest: &NetDownloadManifest,
    chunk: &NetDownloadChunk,
) -> Vec<String> {
    let mut urls = Vec::with_capacity(1 + manifest.mirror_urls.len());
    urls.push(chunk.url.clone());
    urls.extend(
        manifest
            .mirror_urls
            .iter()
            .map(|mirror| format!("{}/{}", mirror.trim_end_matches('/'), chunk.id)),
    );
    urls
}

fn candidate_url_for_attempt(
    manifest: &NetDownloadManifest,
    chunk: &NetDownloadChunk,
    attempt_index: usize,
) -> Option<String> {
    if attempt_index == 0 {
        return Some(chunk.url.clone());
    }
    manifest
        .mirror_urls
        .get(attempt_index - 1)
        .map(|mirror| format!("{}/{}", mirror.trim_end_matches('/'), chunk.id))
}

fn attempt_descriptor_for_chunk(
    download: NetDownloadId,
    chunk: &NetDownloadChunk,
    url: String,
    attempt_index: usize,
) -> NetDownloadAttemptDescriptor {
    NetDownloadAttemptDescriptor {
        download,
        chunk_id: chunk.id.clone(),
        url,
        byte_offset: chunk.byte_offset,
        byte_len: chunk.byte_len,
        range_start: chunk
            .allow_range_resume
            .then_some(chunk.resume_from_byte.unwrap_or(chunk.byte_offset)),
        attempt_index,
    }
}

#[cfg(test)]
mod selected_attempt_url_tests {
    use std::{hint::black_box, time::Instant};

    use zircon_runtime::core::framework::net::{
        NetDownloadChunk, NetDownloadId, NetDownloadManifest,
    };

    use super::{candidate_url_for_attempt, candidate_urls_for_chunk};

    const BENCHMARK_MIRROR_COUNT: usize = 256;
    const BENCHMARK_LOOKUPS_PER_SAMPLE: usize = 128;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[test]
    fn selected_attempt_url_matches_full_candidate_projection() {
        let manifest = NetDownloadManifest::new(NetDownloadId::new(9), "asset://bundle")
            .with_chunk(NetDownloadChunk::new(
                "chunk-main",
                "https://primary.example/chunk-main",
                0,
                64,
                [7; 32],
            ))
            .with_mirror_url("https://mirror-a.example/root/")
            .with_mirror_url("https://mirror-b.example/root");
        let chunk = &manifest.chunks[0];
        let legacy = candidate_urls_for_chunk(&manifest, chunk);

        for attempt_index in 0..=legacy.len() {
            assert_eq!(
                candidate_url_for_attempt(&manifest, chunk, attempt_index),
                legacy.get(attempt_index).cloned()
            );
        }
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn selected_attempt_url_release_benchmark_evidence() {
        let manifest = benchmark_manifest();
        let chunk = &manifest.chunks[0];
        let selected_attempt = BENCHMARK_MIRROR_COUNT;
        assert_eq!(
            legacy_selected_url(&manifest, chunk, selected_attempt),
            candidate_url_for_attempt(&manifest, chunk, selected_attempt)
                .expect("last mirror is available")
        );

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || legacy_lookup_checksum(&manifest, chunk, selected_attempt),
            || optimized_lookup_checksum(&manifest, chunk, selected_attempt),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_raw_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_raw_ns = benchmark_samples_csv(&optimized_samples);
        let legacy_url_allocations = (BENCHMARK_MIRROR_COUNT + 2) * BENCHMARK_LOOKUPS_PER_SAMPLE;

        println!(
            "PERF_RESULT task=plugins10_selected_attempt_url mirrors={BENCHMARK_MIRROR_COUNT} selected_attempt={selected_attempt} lookups_per_sample={BENCHMARK_LOOKUPS_PER_SAMPLE} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_url_string_allocations_per_sample={legacy_url_allocations} optimized_url_string_allocations_per_sample={BENCHMARK_LOOKUPS_PER_SAMPLE} legacy_url_vector_allocations_per_sample={BENCHMARK_LOOKUPS_PER_SAMPLE} optimized_url_vector_allocations_per_sample=0 threshold_percent=50 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={legacy_raw_ns} optimized_raw_ns={optimized_raw_ns}"
        );
        assert!(
            optimized_p95 * 2 <= legacy_p95,
            "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
        );
    }

    fn benchmark_manifest() -> NetDownloadManifest {
        let mut manifest = NetDownloadManifest::new(NetDownloadId::new(77), "asset://benchmark")
            .with_chunk(NetDownloadChunk::new(
                "chunk-benchmark-with-a-long-stable-identifier",
                "https://primary.example/content/chunk-benchmark-with-a-long-stable-identifier",
                0,
                4_096,
                [11; 32],
            ));
        manifest.mirror_urls = (0..BENCHMARK_MIRROR_COUNT)
            .map(|index| {
                format!("https://mirror-{index:04}.example/content/root/with/a/long/stable/path")
            })
            .collect();
        manifest
    }

    fn legacy_selected_url(
        manifest: &NetDownloadManifest,
        chunk: &NetDownloadChunk,
        attempt_index: usize,
    ) -> String {
        candidate_urls_for_chunk(manifest, chunk)
            .get(attempt_index)
            .cloned()
            .expect("benchmark attempt exists")
    }

    fn legacy_lookup_checksum(
        manifest: &NetDownloadManifest,
        chunk: &NetDownloadChunk,
        attempt_index: usize,
    ) -> usize {
        let mut checksum = 0;
        for _ in 0..BENCHMARK_LOOKUPS_PER_SAMPLE {
            checksum += black_box(legacy_selected_url(
                black_box(manifest),
                black_box(chunk),
                black_box(attempt_index),
            ))
            .len();
        }
        black_box(checksum)
    }

    fn optimized_lookup_checksum(
        manifest: &NetDownloadManifest,
        chunk: &NetDownloadChunk,
        attempt_index: usize,
    ) -> usize {
        let mut checksum = 0;
        for _ in 0..BENCHMARK_LOOKUPS_PER_SAMPLE {
            checksum += black_box(candidate_url_for_attempt(
                black_box(manifest),
                black_box(chunk),
                black_box(attempt_index),
            ))
            .expect("benchmark attempt exists")
            .len();
        }
        black_box(checksum)
    }

    fn benchmark_paired_samples(
        mut legacy: impl FnMut() -> usize,
        mut optimized: impl FnMut() -> usize,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(legacy());
        black_box(optimized());
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(benchmark_sample(&mut legacy));
                optimized_samples.push(benchmark_sample(&mut optimized));
            } else {
                optimized_samples.push(benchmark_sample(&mut optimized));
                legacy_samples.push(benchmark_sample(&mut legacy));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
        let started = Instant::now();
        let checksum = black_box(operation());
        let elapsed = started.elapsed().as_nanos();
        black_box(checksum);
        elapsed
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        let index = (sorted.len() * percentile).div_ceil(100) - 1;
        sorted[index]
    }
}
