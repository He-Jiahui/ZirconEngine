use crate::engine::SoundEngineState;

pub(crate) fn retain_timeline_sequences_for_automation_bindings(state: &mut SoundEngineState) {
    let SoundEngineState {
        automation_bindings,
        timeline_sequences,
        ..
    } = state;
    timeline_sequences.retain(|playback| {
        playback
            .sequence
            .tracks
            .iter()
            .all(|track| automation_bindings.contains_key(&track.binding))
    });
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::core::framework::sound::{
        SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationCurve,
        SoundAutomationKeyframe, SoundAutomationTarget, SoundParameterId,
        SoundTimelineAutomationTrack, SoundTimelineSequence, SoundTimelineSequenceId,
    };

    use crate::timeline::playback::SoundTimelineSequencePlayback;
    use crate::SoundConfig;

    use super::*;

    const BENCHMARK_SAMPLE_PAIRS: usize = 21;
    const BENCHMARK_BINDING_COUNT: usize = 2_048;

    #[test]
    fn direct_timeline_binding_lookup_matches_legacy_retention() {
        let mut legacy = timeline_retention_state(5, 3, true);
        let mut optimized = timeline_retention_state(5, 3, true);

        legacy_retain_timeline_sequences(&mut legacy);
        retain_timeline_sequences_for_automation_bindings(&mut optimized);

        assert_eq!(optimized.timeline_sequences.len(), 3);
        assert_eq!(
            optimized
                .timeline_sequences
                .iter()
                .map(|playback| playback.sequence.id.as_str())
                .collect::<Vec<_>>(),
            legacy
                .timeline_sequences
                .iter()
                .map(|playback| playback.sequence.id.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    #[ignore = "release-only performance gate"]
    fn direct_timeline_binding_lookup_release_gate() {
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

        println!(
            "PERF_RESULT task=plugins11_direct_timeline_binding_lookup bindings={BENCHMARK_BINDING_COUNT} sequences={BENCHMARK_BINDING_COUNT} tracks_per_sequence=1 sample_pairs={BENCHMARK_SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_transient_index_builds_per_sample=1 optimized_transient_index_builds_per_sample=0 legacy_binding_id_copies_per_sample={BENCHMARK_BINDING_COUNT} optimized_binding_id_copies_per_sample=0 threshold_percent=25 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={legacy_ns} optimized_raw_ns={optimized_ns}"
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
            "optimized P95 {optimized_p95}ns must be at least 25% faster than legacy P95 {legacy_p95}ns"
        );
    }

    fn timeline_retention_state(
        sequence_count: usize,
        binding_count: usize,
        interleave_missing_bindings: bool,
    ) -> SoundEngineState {
        let mut state = SoundEngineState::new(&SoundConfig::default());
        for index in 0..binding_count {
            let binding = SoundAutomationBindingId::new(index as u64 + 1);
            state.automation_bindings.insert(
                binding,
                SoundAutomationBinding {
                    id: binding,
                    timeline_track_path: "Timeline/Retention:sound.value".to_string(),
                    target: SoundAutomationTarget::SynthParameter(SoundParameterId::new(
                        "timeline.retention",
                    )),
                    parameter: SoundParameterId::new("value"),
                },
            );
        }
        state.timeline_sequences = (0..sequence_count)
            .map(|sequence_index| {
                let binding = if interleave_missing_bindings && sequence_index % 2 == 1 {
                    SoundAutomationBindingId::new(binding_count as u64 + sequence_index as u64 + 1)
                } else {
                    SoundAutomationBindingId::new(sequence_index as u64 % binding_count as u64 + 1)
                };
                SoundTimelineSequencePlayback {
                    sequence: SoundTimelineSequence::new(
                        SoundTimelineSequenceId::new(format!(
                            "timeline-retention-{sequence_index}"
                        )),
                        1.0,
                        true,
                        vec![SoundTimelineAutomationTrack {
                            binding,
                            curve: SoundAutomationCurve::from_keyframes([
                                SoundAutomationKeyframe::linear(0.0, 0.0),
                                SoundAutomationKeyframe::linear(1.0, 1.0),
                            ]),
                        }],
                    ),
                    time_seconds: 0.25,
                }
            })
            .collect();
        state
    }

    fn legacy_retain_timeline_sequences(state: &mut SoundEngineState) {
        let automation_binding_ids = state
            .automation_bindings
            .keys()
            .copied()
            .collect::<HashSet<_>>();
        state.timeline_sequences.retain(|playback| {
            playback
                .sequence
                .tracks
                .iter()
                .all(|track| automation_binding_ids.contains(&track.binding))
        });
    }

    fn legacy_benchmark_sample() -> u128 {
        benchmark_sample(legacy_retain_timeline_sequences)
    }

    fn optimized_benchmark_sample() -> u128 {
        benchmark_sample(retain_timeline_sequences_for_automation_bindings)
    }

    fn benchmark_sample(operation: fn(&mut SoundEngineState)) -> u128 {
        let mut state =
            timeline_retention_state(BENCHMARK_BINDING_COUNT, BENCHMARK_BINDING_COUNT, false);
        let started = Instant::now();
        operation(black_box(&mut state));
        let elapsed = started.elapsed().as_nanos();
        assert_eq!(state.timeline_sequences.len(), BENCHMARK_BINDING_COUNT);
        black_box(state.timeline_sequences);
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
