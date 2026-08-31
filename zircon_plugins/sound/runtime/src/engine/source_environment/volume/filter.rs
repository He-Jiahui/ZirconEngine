pub(in crate::engine::source_environment) fn low_pass_block(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    cutoff_hz: f32,
    amount: f32,
) {
    if channels == 0 || cutoff_hz <= 0.0 || amount <= 0.0 {
        return;
    }
    let rc = 1.0 / (cutoff_hz * std::f32::consts::TAU);
    let dt = 1.0 / sample_rate_hz.max(1) as f32;
    let alpha = (dt / (rc + dt)).clamp(0.0, 1.0);
    let frames = buffer.len() / channels;
    for channel in 0..channels {
        let mut low = 0.0;
        for frame in 0..frames {
            let index = frame * channels + channel;
            let dry_sample = buffer[index];
            low += alpha * (dry_sample - low);
            buffer[index] = dry_sample * (1.0 - amount) + low * amount;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::mem::size_of;
    use std::time::Instant;

    use super::*;

    const BENCHMARK_SAMPLE_PAIRS: usize = 21;
    const BENCHMARK_BLOCK_FRAMES: usize = 256;
    const BENCHMARK_CHANNELS: usize = 2;
    const BENCHMARK_BLOCKS_PER_SAMPLE: usize = 2_048;

    #[test]
    fn low_pass_in_place_matches_legacy_samples_and_preserves_tail() {
        let input = (0..26)
            .map(|index| ((index as f32 * 0.37).sin() * 0.8) + index as f32 * 0.001)
            .collect::<Vec<_>>();
        let trailing_samples = input[24..].to_vec();
        let mut legacy = input.clone();
        let mut optimized = input;

        legacy_low_pass_block(&mut legacy, 3, 48_000, 1_850.0, 0.65);
        low_pass_block(&mut optimized, 3, 48_000, 1_850.0, 0.65);

        assert_eq!(optimized, legacy);
        assert_eq!(&optimized[24..], trailing_samples.as_slice());
    }

    #[test]
    fn low_pass_zero_channels_remains_a_no_op() {
        let mut samples = vec![0.25, -0.5, 0.75];

        low_pass_block(&mut samples, 0, 48_000, 1_000.0, 1.0);

        assert_eq!(samples, vec![0.25, -0.5, 0.75]);
    }

    #[test]
    #[ignore = "release-only performance gate"]
    fn low_pass_in_place_release_gate() {
        black_box(legacy_benchmark_sample());
        black_box(optimized_benchmark_sample());

        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        for sample_index in 0..BENCHMARK_SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(legacy_benchmark_sample());
                optimized_samples.push(optimized_benchmark_sample());
            } else {
                optimized_samples.push(optimized_benchmark_sample());
                legacy_samples.push(legacy_benchmark_sample());
            }
        }

        let legacy_p50 = nearest_rank_percentile(&legacy_samples, 50);
        let legacy_p95 = nearest_rank_percentile(&legacy_samples, 95);
        let optimized_p50 = nearest_rank_percentile(&optimized_samples, 50);
        let optimized_p95 = nearest_rank_percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);
        let transient_bytes = BENCHMARK_BLOCKS_PER_SAMPLE
            * BENCHMARK_BLOCK_FRAMES
            * BENCHMARK_CHANNELS
            * size_of::<f32>();

        println!(
            "PERF_RESULT task=plugins11_in_place_low_pass block_frames={BENCHMARK_BLOCK_FRAMES} channels={BENCHMARK_CHANNELS} blocks_per_sample={BENCHMARK_BLOCKS_PER_SAMPLE} sample_pairs={BENCHMARK_SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_transient_allocations_per_sample={BENCHMARK_BLOCKS_PER_SAMPLE} optimized_transient_allocations_per_sample=0 legacy_transient_bytes_per_sample={transient_bytes} optimized_transient_bytes_per_sample=0 threshold_percent=15 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={legacy_ns} optimized_raw_ns={optimized_ns}"
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(85),
            "optimized P95 {optimized_p95}ns must be at least 15% faster than legacy P95 {legacy_p95}ns"
        );
    }

    fn legacy_low_pass_block(
        buffer: &mut [f32],
        channels: usize,
        sample_rate_hz: u32,
        cutoff_hz: f32,
        amount: f32,
    ) {
        if cutoff_hz <= 0.0 || amount <= 0.0 {
            return;
        }
        let dry = buffer.to_vec();
        let rc = 1.0 / (cutoff_hz * std::f32::consts::TAU);
        let dt = 1.0 / sample_rate_hz.max(1) as f32;
        let alpha = (dt / (rc + dt)).clamp(0.0, 1.0);
        for channel in 0..channels {
            let mut low = 0.0;
            for frame in 0..(buffer.len() / channels) {
                let index = frame * channels + channel;
                low += alpha * (dry[index] - low);
                buffer[index] = dry[index] * (1.0 - amount) + low * amount;
            }
        }
    }

    fn legacy_benchmark_sample() -> u128 {
        benchmark_sample(legacy_low_pass_block)
    }

    fn optimized_benchmark_sample() -> u128 {
        benchmark_sample(low_pass_block)
    }

    fn benchmark_sample(operation: fn(&mut [f32], usize, u32, f32, f32)) -> u128 {
        let mut buffer = (0..BENCHMARK_BLOCK_FRAMES * BENCHMARK_CHANNELS)
            .map(|index| (index as f32 * 0.013).sin())
            .collect::<Vec<_>>();
        let started = Instant::now();
        for _ in 0..BENCHMARK_BLOCKS_PER_SAMPLE {
            operation(
                black_box(buffer.as_mut_slice()),
                BENCHMARK_CHANNELS,
                48_000,
                1_850.0,
                0.65,
            );
        }
        let elapsed = started.elapsed().as_nanos();
        black_box(buffer);
        elapsed
    }

    fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        sorted[(sorted.len() * percentile).div_ceil(100) - 1]
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
