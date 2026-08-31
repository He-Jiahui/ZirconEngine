#[inline]
pub(super) fn collect_fixed_packed_words<T, const WORD_COUNT: usize>(
    values: &[T],
    packed_words: impl Fn(&T) -> [u32; WORD_COUNT],
) -> Vec<u32> {
    let capacity = values
        .len()
        .checked_mul(WORD_COUNT)
        .expect("fixed-width packed word count exceeds addressable memory");
    let mut words = Vec::with_capacity(capacity);
    for value in values {
        words.extend_from_slice(&packed_words(value));
    }
    words
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::collect_fixed_packed_words;

    const BENCH_RECORD_COUNT: usize = 4_096;
    const BENCH_WORD_COUNT: usize = 16;
    const CHECKS_PER_SAMPLE: usize = 32;
    const SAMPLE_PAIRS: usize = 21;

    #[test]
    fn fixed_width_packer_preserves_record_and_word_order() {
        let records = [[1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12]];

        let packed = collect_fixed_packed_words(&records, |record| *record);

        assert_eq!(packed, (1..=12).collect::<Vec<_>>());
        assert_eq!(packed.len(), records.len() * 4);
        assert!(packed.capacity() >= packed.len());
        assert!(collect_fixed_packed_words::<[u32; 4], 4>(&[], |record| *record).is_empty());
    }

    #[test]
    #[ignore = "release-only fixed-width GPU word packing benchmark"]
    fn fixed_width_packing_release_benchmark_evidence() {
        fn legacy_pack(records: &[[u32; BENCH_WORD_COUNT]]) -> Vec<u32> {
            records.iter().flat_map(|record| *record).collect()
        }

        fn measure_legacy(records: &[[u32; BENCH_WORD_COUNT]]) -> u128 {
            let started = Instant::now();
            for _ in 0..CHECKS_PER_SAMPLE {
                black_box(legacy_pack(black_box(records)));
            }
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(records: &[[u32; BENCH_WORD_COUNT]]) -> u128 {
            let started = Instant::now();
            for _ in 0..CHECKS_PER_SAMPLE {
                black_box(collect_fixed_packed_words(black_box(records), |record| {
                    *record
                }));
            }
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

        let records = (0..BENCH_RECORD_COUNT)
            .map(|record_index| {
                std::array::from_fn(|word_index| {
                    u32::try_from(record_index * BENCH_WORD_COUNT + word_index).unwrap_or(u32::MAX)
                })
            })
            .collect::<Vec<[u32; BENCH_WORD_COUNT]>>();
        assert_eq!(
            legacy_pack(&records),
            collect_fixed_packed_words(&records, |record| *record)
        );

        for _ in 0..4 {
            black_box(measure_legacy(&records));
            black_box(measure_optimized(&records));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(&records));
                optimized_samples.push(measure_optimized(&records));
            } else {
                optimized_samples.push(measure_optimized(&records));
                legacy_samples.push(measure_legacy(&records));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);

        println!(
            "PERF_RESULT plan=Plugins17 task=fixed_width_gpu_word_packing \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
record_count={BENCH_RECORD_COUNT} words_per_record={BENCH_WORD_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_preallocated_words=0 optimized_preallocated_words={} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            BENCH_RECORD_COUNT * BENCH_WORD_COUNT,
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
            "fixed-width packing must reduce P95 by at least 25%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
