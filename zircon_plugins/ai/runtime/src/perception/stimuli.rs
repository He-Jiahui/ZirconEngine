use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
};

use zircon_runtime::core::framework::ai::{
    AiPerceptionSense, AiPerceptionSnapshot, AiPerceptionStimulus,
};
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::ecs::Resource;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct StimulusKey {
    source: EntityId,
    sense: AiPerceptionSense,
}

impl PartialOrd for StimulusKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StimulusKey {
    fn cmp(&self, other: &Self) -> Ordering {
        sense_rank(self.sense)
            .cmp(&sense_rank(other.sense))
            .then_with(|| self.source.cmp(&other.source))
    }
}

#[derive(Debug, Default)]
pub struct PerceivedStimuli {
    by_receiver: BTreeMap<EntityId, BTreeMap<StimulusKey, AiPerceptionStimulus>>,
    scan_cursor: usize,
}

impl Resource for PerceivedStimuli {}

impl PerceivedStimuli {
    pub fn snapshot(&self, receiver: EntityId) -> Option<AiPerceptionSnapshot> {
        self.by_receiver
            .get(&receiver)
            .map(|stimuli| snapshot(receiver, stimuli))
    }

    pub fn snapshots(&self) -> Vec<AiPerceptionSnapshot> {
        self.by_receiver
            .iter()
            .map(|(receiver, stimuli)| snapshot(*receiver, stimuli))
            .collect()
    }

    pub(crate) fn begin_frame(
        &mut self,
        delta_seconds: Real,
        receivers: &[(EntityId, Real)],
    ) -> usize {
        let delta_seconds = if delta_seconds.is_finite() {
            delta_seconds.max(0.0)
        } else {
            0.0
        };
        let forget_by_receiver = receivers.iter().copied().collect::<HashMap<_, _>>();
        self.by_receiver
            .retain(|receiver, _| forget_by_receiver.contains_key(receiver));
        for (receiver, _) in receivers {
            self.by_receiver.entry(*receiver).or_default();
        }

        let before = stimulus_count(&self.by_receiver);
        for (receiver, stimuli) in &mut self.by_receiver {
            let forget_seconds = forget_by_receiver
                .get(receiver)
                .copied()
                .unwrap_or_default()
                .max(0.0);
            stimuli.retain(|_, stimulus| {
                stimulus.age_seconds += delta_seconds;
                stimulus.age_seconds < forget_seconds
            });
        }
        before.saturating_sub(stimulus_count(&self.by_receiver))
    }

    pub(crate) fn refresh(&mut self, receiver: EntityId, stimulus: AiPerceptionStimulus) {
        let key = StimulusKey {
            source: stimulus.source,
            sense: stimulus.sense,
        };
        let stimuli = self.by_receiver.entry(receiver).or_default();
        match stimuli.get_mut(&key) {
            Some(current) if current.age_seconds <= stimulus.age_seconds => {}
            Some(current) => *current = stimulus,
            None => {
                stimuli.insert(key, stimulus);
            }
        }
    }

    pub(crate) fn scan_cursor(&self, pair_slot_count: usize) -> usize {
        if pair_slot_count == 0 {
            0
        } else {
            self.scan_cursor % pair_slot_count
        }
    }

    pub(crate) fn set_scan_cursor(&mut self, scan_cursor: usize) {
        self.scan_cursor = scan_cursor;
    }
}

fn snapshot(
    receiver: EntityId,
    stimuli: &BTreeMap<StimulusKey, AiPerceptionStimulus>,
) -> AiPerceptionSnapshot {
    AiPerceptionSnapshot {
        agent: receiver,
        stimuli: stimuli.values().cloned().collect(),
    }
}

fn sense_rank(sense: AiPerceptionSense) -> u8 {
    match sense {
        AiPerceptionSense::Sight => 0,
        AiPerceptionSense::Hearing => 1,
        AiPerceptionSense::Damage => 2,
        AiPerceptionSense::Touch => 3,
        AiPerceptionSense::Custom => 4,
    }
}

fn stimulus_count(
    by_receiver: &BTreeMap<EntityId, BTreeMap<StimulusKey, AiPerceptionStimulus>>,
) -> usize {
    by_receiver.values().map(BTreeMap::len).sum()
}

#[cfg(test)]
mod ordered_snapshot_tests {
    use std::{collections::HashMap, hint::black_box, time::Instant};

    use zircon_runtime::core::framework::ai::{
        AiPerceptionSense, AiPerceptionSnapshot, AiPerceptionStimulus,
    };
    use zircon_runtime::core::framework::scene::EntityId;
    use zircon_runtime::core::math::Vec3;

    use super::{PerceivedStimuli, StimulusKey, sense_rank};

    const BENCHMARK_STIMULUS_COUNT: usize = 8_192;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;
    const TEST_RECEIVER: EntityId = 1;

    #[test]
    fn ordered_stimulus_storage_preserves_snapshot_contract_after_out_of_order_refresh() {
        let mut perceived = PerceivedStimuli::default();
        for (source, sense) in [
            (9, AiPerceptionSense::Hearing),
            (3, AiPerceptionSense::Sight),
            (2, AiPerceptionSense::Hearing),
            (7, AiPerceptionSense::Sight),
        ] {
            perceived.refresh(TEST_RECEIVER, stimulus(source, sense));
        }

        assert_eq!(
            perceived
                .snapshot(TEST_RECEIVER)
                .unwrap()
                .stimuli
                .into_iter()
                .map(|stimulus| (stimulus.sense, stimulus.source))
                .collect::<Vec<_>>(),
            [
                (AiPerceptionSense::Sight, 3),
                (AiPerceptionSense::Sight, 7),
                (AiPerceptionSense::Hearing, 2),
                (AiPerceptionSense::Hearing, 9),
            ]
        );
    }

    #[test]
    fn snapshot_reads_do_not_sort_ordered_stimulus_storage() {
        let source = include_str!("stimuli.rs");
        let snapshot = source
            .split("fn snapshot(")
            .nth(1)
            .and_then(|body| body.split("fn sense_rank").next())
            .expect("snapshot source");

        assert!(source.contains("BTreeMap<StimulusKey, AiPerceptionStimulus>"));
        assert!(!snapshot.contains("sort_by"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn ordered_perception_stimuli_release_benchmark_evidence() {
        let mut perceived = PerceivedStimuli::default();
        let mut legacy = HashMap::with_capacity(BENCHMARK_STIMULUS_COUNT);
        for source in 1..=BENCHMARK_STIMULUS_COUNT as u64 {
            let sense = benchmark_sense(source);
            let stimulus = stimulus(source, sense);
            legacy.insert(StimulusKey { source, sense }, stimulus.clone());
            perceived.refresh(TEST_RECEIVER, stimulus);
        }
        assert_eq!(
            legacy_snapshot(TEST_RECEIVER, &legacy),
            perceived.snapshot(TEST_RECEIVER).unwrap()
        );

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || legacy_snapshot(TEST_RECEIVER, &legacy),
            || perceived.snapshot(TEST_RECEIVER).unwrap(),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins15_ordered_perception_stimuli stimuli={} samples={} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_snapshot_sorts=1 optimized_snapshot_sorts=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
            BENCHMARK_STIMULUS_COUNT,
            BENCHMARK_SAMPLE_COUNT,
            legacy_p50,
            legacy_p95,
            optimized_p50,
            optimized_p95
        );
        assert!(
            optimized_p95 * 4 <= legacy_p95 * 3,
            "optimized P95 {optimized_p95}ns must be no more than 75% of legacy P95 {legacy_p95}ns"
        );
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn legacy_snapshot(
        receiver: EntityId,
        stimuli: &HashMap<StimulusKey, AiPerceptionStimulus>,
    ) -> AiPerceptionSnapshot {
        let mut stimuli = stimuli.values().cloned().collect::<Vec<_>>();
        stimuli.sort_by_key(|stimulus| (sense_rank(stimulus.sense), stimulus.source));
        AiPerceptionSnapshot {
            agent: receiver,
            stimuli,
        }
    }

    fn benchmark_sense(source: u64) -> AiPerceptionSense {
        match source % 5 {
            0 => AiPerceptionSense::Sight,
            1 => AiPerceptionSense::Hearing,
            2 => AiPerceptionSense::Damage,
            3 => AiPerceptionSense::Touch,
            _ => AiPerceptionSense::Custom,
        }
    }

    fn stimulus(source: EntityId, sense: AiPerceptionSense) -> AiPerceptionStimulus {
        AiPerceptionStimulus {
            source,
            sense,
            position: Vec3::new(source as f32, 0.0, 0.0),
            strength: 1.0,
            age_seconds: 0.0,
        }
    }

    fn benchmark_paired_samples<L, O>(
        mut legacy: impl FnMut() -> L,
        mut optimized: impl FnMut() -> O,
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

    fn benchmark_sample<T>(operation: &mut impl FnMut() -> T) -> u128 {
        let started = Instant::now();
        let result = black_box(operation());
        let elapsed = started.elapsed().as_nanos();
        black_box(&result);
        elapsed
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
