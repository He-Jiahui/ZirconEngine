use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const LOWER_HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LibraryCacheKey {
    source_hash: String,
    importer_version: u32,
    config_hash: String,
}

impl LibraryCacheKey {
    pub fn new(
        source_hash: impl Into<String>,
        importer_version: u32,
        config_hash: impl Into<String>,
    ) -> Self {
        Self {
            source_hash: source_hash.into(),
            importer_version,
            config_hash: config_hash.into(),
        }
    }

    pub fn fingerprint(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        fixed_lower_hex(hasher.finish())
    }
}

fn fixed_lower_hex(value: u64) -> String {
    let mut fingerprint = String::with_capacity(16);
    for shift in (0..=60).rev().step_by(4) {
        let nibble = ((value >> shift) & 0x0f) as usize;
        fingerprint.push(char::from(LOWER_HEX_DIGITS[nibble]));
    }
    fingerprint
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const VALUES_PER_SAMPLE: usize = 262_144;

    #[test]
    fn optimization_batch_ey_runtime457_preserves_cache_fingerprint_bytes() {
        for value in [0, 1, 0x0f, 0x10, 0x0123_4567_89ab_cdef, u64::MAX] {
            assert_eq!(fixed_lower_hex(value), format!("{value:016x}"));
        }

        let key = LibraryCacheKey::new("source", 7, "config");
        let mut legacy_hasher = DefaultHasher::new();
        key.hash(&mut legacy_hasher);
        assert_eq!(
            key.fingerprint(),
            format!("{:016x}", legacy_hasher.finish())
        );

        let production = include_str!("cache_key.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        assert!(!production.contains("format!("));
        assert!(production.contains("String::with_capacity(16)"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_ey_runtime457_direct_cache_fingerprint_hex_benchmark() {
        for _ in 0..4 {
            black_box(measure_legacy_hex());
            black_box(measure_direct_hex());
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy_hex());
                optimized_samples.push(measure_direct_hex());
            } else {
                optimized_samples.push(measure_direct_hex());
                legacy_samples.push(measure_legacy_hex());
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn measure_legacy_hex() -> u128 {
        measure_hex(|value| format!("{value:016x}"))
    }

    fn measure_direct_hex() -> u128 {
        measure_hex(fixed_lower_hex)
    }

    fn measure_hex(mut encode: impl FnMut(u64) -> String) -> u128 {
        let started = Instant::now();
        let mut total_len = 0_usize;
        for index in 0..VALUES_PER_SAMPLE {
            let value = black_box((index as u64).wrapping_mul(0xd6e8_feb8_6659_fd93));
            let fingerprint = encode(value);
            total_len += black_box(fingerprint.len());
            black_box(fingerprint);
        }
        assert_eq!(black_box(total_len), VALUES_PER_SAMPLE * 16);
        started.elapsed().as_nanos().max(1)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME457_DIRECT_CACHE_FINGERPRINT_HEX_BENCH_V1 sample_pairs={SAMPLE_PAIRS} values_per_sample={VALUES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=35",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(65) / 100,
            "direct cache fingerprint hex encoding must reduce P95 by at least 35%"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
