use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::input::{
    InputEvent, InputEventRecord, InputEventRecordingConfig, InputEventRecordingStatus,
};

#[derive(Debug, Default)]
pub(in crate::input::runtime) struct InputEventRecorder {
    config: InputEventRecordingConfig,
    records: VecDeque<InputEventRecord>,
    discarded_records: u64,
    next_sequence: u64,
}

impl InputEventRecorder {
    pub(in crate::input::runtime) fn configure(&mut self, config: InputEventRecordingConfig) {
        if !config.enabled {
            self.config = config;
            self.records.clear();
            self.discarded_records = 0;
            self.next_sequence = 0;
            return;
        }

        if !self.config.enabled {
            self.records.clear();
            self.discarded_records = 0;
            self.next_sequence = 0;
        }
        self.config = config;
        trim_records_to_capacity(
            &mut self.records,
            config.capacity as usize,
            &mut self.discarded_records,
        );
    }

    pub(in crate::input::runtime) fn record(&mut self, event: &InputEvent) {
        if !self.config.enabled {
            return;
        }

        self.next_sequence = self.next_sequence.saturating_add(1);
        if self.config.capacity == 0 {
            self.discarded_records = self.discarded_records.saturating_add(1);
            return;
        }
        if self.records.len() >= self.config.capacity as usize {
            self.records.pop_front();
            self.discarded_records = self.discarded_records.saturating_add(1);
        }
        self.records.push_back(InputEventRecord {
            sequence: self.next_sequence,
            timestamp_millis: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            event: event.clone(),
        });
    }

    pub(in crate::input::runtime) fn status(&self) -> InputEventRecordingStatus {
        InputEventRecordingStatus {
            enabled: self.config.enabled,
            capacity: self.config.capacity,
            retained_records: self.records.len() as u32,
            discarded_records: self.discarded_records,
        }
    }

    pub(in crate::input::runtime) fn drain(&mut self) -> Vec<InputEventRecord> {
        self.records.drain(..).collect()
    }
}

fn trim_records_to_capacity(
    records: &mut VecDeque<InputEventRecord>,
    capacity: usize,
    discarded_records: &mut u64,
) {
    let excess = records.len().saturating_sub(capacity);
    if excess == 0 {
        return;
    }

    drop(records.drain(..excess));
    *discarded_records = discarded_records.saturating_add(excess as u64);
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const BENCHMARK_SAMPLES: usize = 11;
    const BENCHMARK_ITERATIONS: usize = 16;
    const BENCHMARK_RECORD_COUNT: usize = 16_384;
    const BENCHMARK_CAPACITY: usize = 256;

    #[test]
    fn runtime56_recovery_batch_bulk_recorder_capacity_trim_preserves_retired_records_and_counter()
    {
        let records = (1..=8).map(event_record).collect::<VecDeque<_>>();
        let mut retired_records = records.clone();
        let mut optimized_records = records;
        let mut retired_discarded = u64::MAX - 2;
        let mut optimized_discarded = retired_discarded;

        retired_trim_records_to_capacity(&mut retired_records, 3, &mut retired_discarded);
        trim_records_to_capacity(&mut optimized_records, 3, &mut optimized_discarded);

        assert_eq!(optimized_records, retired_records);
        assert_eq!(optimized_discarded, retired_discarded);
        assert_eq!(
            optimized_records
                .iter()
                .map(|record| record.sequence)
                .collect::<Vec<_>>(),
            vec![6, 7, 8]
        );
        assert_eq!(optimized_discarded, u64::MAX);
    }

    #[test]
    fn runtime56_recovery_batch_bulk_recorder_capacity_trim_source_contract() {
        let source = include_str!("recorder.rs");
        let production = source
            .split_once("\n#[cfg(test)]\nmod tests")
            .expect("production module end")
            .0;
        let trim = production
            .split_once("fn trim_records_to_capacity")
            .expect("bulk recorder trim helper")
            .1;

        assert!(production.contains("trim_records_to_capacity("));
        assert!(trim.contains("saturating_sub(capacity)"));
        assert!(trim.contains("drop(records.drain(..excess))"));
        assert_eq!(trim.matches("saturating_add").count(), 1);
        assert!(!trim.contains("while records.len() > capacity"));
        assert!(!trim.contains("records.pop_front()"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn runtime56_recovery_batch_bulk_recorder_capacity_trim_release_benchmark() {
        let base = (0..BENCHMARK_RECORD_COUNT as u64)
            .map(event_record)
            .collect::<VecDeque<_>>();
        let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

        for sample in 0..BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_trim(&base, retired_trim_records_to_capacity));
                optimized_samples.push(measure_trim(&base, trim_records_to_capacity));
            } else {
                optimized_samples.push(measure_trim(&base, trim_records_to_capacity));
                retired_samples.push(measure_trim(&base, retired_trim_records_to_capacity));
            }
        }

        let retired_p95 = percentile_95(&mut retired_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        let discarded_per_trim = BENCHMARK_RECORD_COUNT - BENCHMARK_CAPACITY;
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
        );
        eprintln!(
            "RUNTIME56_BULK_RECORDER_CAPACITY_TRIM_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} \
records={BENCHMARK_RECORD_COUNT} capacity={BENCHMARK_CAPACITY} \
retired_front_pops_per_trim={discarded_per_trim} optimized_drain_calls_per_trim=1 \
retired_counter_updates_per_trim={discarded_per_trim} optimized_counter_updates_per_trim=1 \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
            retired_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(80),
            "bulk trimming must reduce recorder capacity-shrink P95 by at least 20%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
    }

    fn event_record(sequence: u64) -> InputEventRecord {
        InputEventRecord {
            sequence,
            timestamp_millis: sequence,
            event: InputEvent::CursorMoved {
                x: sequence as f32,
                y: -(sequence as f32),
            },
        }
    }

    fn retired_trim_records_to_capacity(
        records: &mut VecDeque<InputEventRecord>,
        capacity: usize,
        discarded_records: &mut u64,
    ) {
        while records.len() > capacity {
            records.pop_front();
            *discarded_records = discarded_records.saturating_add(1);
        }
    }

    fn measure_trim(
        base: &VecDeque<InputEventRecord>,
        trim: fn(&mut VecDeque<InputEventRecord>, usize, &mut u64),
    ) -> Duration {
        let mut inputs = (0..BENCHMARK_ITERATIONS)
            .map(|_| base.clone())
            .collect::<Vec<_>>();
        let mut discarded_records = vec![0_u64; BENCHMARK_ITERATIONS];
        let started = Instant::now();
        for (records, discarded) in inputs.iter_mut().zip(&mut discarded_records) {
            trim(records, BENCHMARK_CAPACITY, discarded);
        }
        black_box((&inputs, &discarded_records));
        started.elapsed()
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
