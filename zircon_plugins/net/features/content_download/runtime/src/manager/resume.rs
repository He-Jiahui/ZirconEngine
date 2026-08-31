use zircon_runtime::core::framework::net::{NetDownloadAttemptDescriptor, NetDownloadId};

use super::NetContentDownloadRuntimeManager;

impl NetContentDownloadRuntimeManager {
    pub fn store_partial_chunk(
        &self,
        download: NetDownloadId,
        chunk_id: impl Into<String>,
        bytes: Vec<u8>,
    ) {
        self.state()
            .partial_chunks
            .entry(download)
            .or_default()
            .insert(chunk_id.into(), bytes);
    }

    pub fn partial_chunk_bytes(&self, download: NetDownloadId, chunk_id: &str) -> Vec<u8> {
        self.state()
            .partial_chunks
            .get(&download)
            .and_then(|chunks| chunks.get(chunk_id))
            .cloned()
            .unwrap_or_default()
    }

    pub(in crate::manager) fn partial_prefix_for_attempt(
        &self,
        attempt: &NetDownloadAttemptDescriptor,
    ) -> Option<Vec<u8>> {
        let Some(range_start) = attempt.range_start else {
            return Some(Vec::new());
        };
        let expected_prefix_len = range_start.checked_sub(attempt.byte_offset)? as usize;
        let prefix = self
            .state()
            .partial_chunks
            .get(&attempt.download)
            .and_then(|chunks| chunks.get(attempt.chunk_id.as_str()))
            .cloned()
            .unwrap_or_default();
        if prefix.len() == expected_prefix_len {
            Some(prefix)
        } else {
            self.fail_progress(
                attempt.download,
                format!(
                    "chunk resume requires existing partial bytes: {}",
                    attempt.chunk_id
                ),
            )?;
            None
        }
    }
}

#[cfg(test)]
mod nested_partial_chunk_tests {
    use std::{collections::HashMap, hint::black_box, time::Instant};

    use super::{NetContentDownloadRuntimeManager, NetDownloadId};

    const BENCHMARK_LOOKUP_COUNT: usize = 8_192;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[test]
    fn nested_partial_chunk_index_preserves_store_overwrite_and_lookup() {
        let manager = NetContentDownloadRuntimeManager::new();
        let download = NetDownloadId::new(41);

        manager.store_partial_chunk(download, "chunk-a", vec![1, 2]);
        manager.store_partial_chunk(download, "chunk-a", vec![3, 4, 5]);
        manager.store_partial_chunk(download, "chunk-b", vec![6]);

        assert_eq!(
            manager.partial_chunk_bytes(download, "chunk-a"),
            vec![3, 4, 5]
        );
        assert_eq!(manager.partial_chunk_bytes(download, "chunk-b"), vec![6]);
        assert!(manager.partial_chunk_bytes(download, "missing").is_empty());
        let state = manager.state();
        let chunks = state
            .partial_chunks
            .get(&download)
            .expect("download should own one nested partial-chunk table");
        assert_eq!(chunks.len(), 2);
        assert_eq!(
            chunks.get("chunk-a").map(Vec::as_slice),
            Some(&[3, 4, 5][..])
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn nested_partial_chunk_lookup_release_benchmark_evidence() {
        let download = NetDownloadId::new(73);
        let chunk_id = format!("chunk-{}", "x".repeat(1_024));
        let payload = vec![7u8; 32];
        let legacy = HashMap::from([((download, chunk_id.clone()), payload.clone())]);
        let optimized = HashMap::from([(
            download,
            HashMap::from([(chunk_id.clone(), payload.clone())]),
        )]);
        assert_eq!(
            legacy_partial_chunk(&legacy, download, &chunk_id),
            nested_partial_chunk(&optimized, download, &chunk_id)
        );

        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_legacy(&legacy, download, &chunk_id));
                optimized_samples.push(measure_optimized(&optimized, download, &chunk_id));
            } else {
                optimized_samples.push(measure_optimized(&optimized, download, &chunk_id));
                legacy_samples.push(measure_legacy(&legacy, download, &chunk_id));
            }
        }

        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        println!(
            "PERF_RESULT task=plugins10_nested_partial_chunk_lookup lookups={BENCHMARK_LOOKUP_COUNT} chunk_id_bytes={} payload_bytes={} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_key_string_allocations_per_sample={BENCHMARK_LOOKUP_COUNT} optimized_key_string_allocations_per_sample=0 threshold_percent=25 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={} optimized_raw_ns={}",
            chunk_id.len(),
            payload.len(),
            raw_samples(&legacy_samples),
            raw_samples(&optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
            "nested partial chunk lookup P95 {optimized_p95}ns did not improve legacy {legacy_p95}ns by 25%"
        );
    }

    fn legacy_partial_chunk<'a>(
        partial_chunks: &'a HashMap<(NetDownloadId, String), Vec<u8>>,
        download: NetDownloadId,
        chunk_id: &str,
    ) -> Option<&'a [u8]> {
        partial_chunks
            .get(&(download, chunk_id.to_string()))
            .map(Vec::as_slice)
    }

    fn nested_partial_chunk<'a>(
        partial_chunks: &'a HashMap<NetDownloadId, HashMap<String, Vec<u8>>>,
        download: NetDownloadId,
        chunk_id: &str,
    ) -> Option<&'a [u8]> {
        partial_chunks
            .get(&download)
            .and_then(|chunks| chunks.get(chunk_id))
            .map(Vec::as_slice)
    }

    fn measure_legacy(
        partial_chunks: &HashMap<(NetDownloadId, String), Vec<u8>>,
        download: NetDownloadId,
        chunk_id: &str,
    ) -> u128 {
        let start = Instant::now();
        let mut matched_bytes = 0usize;
        for _ in 0..BENCHMARK_LOOKUP_COUNT {
            matched_bytes +=
                legacy_partial_chunk(black_box(partial_chunks), download, black_box(chunk_id))
                    .map_or(0, <[u8]>::len);
        }
        let elapsed = start.elapsed().as_nanos();
        black_box(matched_bytes);
        elapsed
    }

    fn measure_optimized(
        partial_chunks: &HashMap<NetDownloadId, HashMap<String, Vec<u8>>>,
        download: NetDownloadId,
        chunk_id: &str,
    ) -> u128 {
        let start = Instant::now();
        let mut matched_bytes = 0usize;
        for _ in 0..BENCHMARK_LOOKUP_COUNT {
            matched_bytes +=
                nested_partial_chunk(black_box(partial_chunks), download, black_box(chunk_id))
                    .map_or(0, <[u8]>::len);
        }
        let elapsed = start.elapsed().as_nanos();
        black_box(matched_bytes);
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
