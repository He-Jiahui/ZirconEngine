use zircon_runtime::core::framework::sound::{
    SoundError, SoundTimelineAutomationSample, SoundTimelineSequence, SoundTimelineSequenceAdvance,
};

use crate::automation::curve::sample_automation_curve;
use crate::automation::target::apply_automation_target;
use crate::automation::values::ensure_finite_value;
use crate::engine::SoundEngineState;

pub(crate) fn advance_timeline_sequences(
    state: &mut SoundEngineState,
    delta_seconds: f32,
) -> Result<Vec<SoundTimelineSequenceAdvance>, SoundError> {
    ensure_finite_value("timeline sequence delta", delta_seconds)?;
    if delta_seconds < 0.0 {
        return Err(SoundError::InvalidParameter(
            "timeline sequence delta must be non-negative".to_string(),
        ));
    }

    let mut scheduled = std::mem::take(&mut state.timeline_sequences);
    let scheduled_count = scheduled.len();
    let mut retained = Vec::with_capacity(scheduled_count);
    let mut reports = Vec::with_capacity(scheduled_count);
    for mut playback in scheduled.drain(..) {
        let raw_time = playback.time_seconds + delta_seconds;
        let (sample_time, completed) = resolve_sample_time(
            playback.sequence.duration_seconds,
            raw_time,
            playback.sequence.looping,
        );
        let samples = apply_timeline_sequence_at(state, &playback.sequence, sample_time)?;
        reports.push(SoundTimelineSequenceAdvance {
            sequence: playback.sequence.id.clone(),
            time_seconds: sample_time,
            completed,
            samples,
        });
        if !completed {
            playback.time_seconds = sample_time;
            retained.push(playback);
        }
    }
    state.timeline_sequences = retained;
    Ok(reports)
}

fn resolve_sample_time(duration_seconds: f32, time_seconds: f32, looping: bool) -> (f32, bool) {
    if looping {
        (time_seconds.rem_euclid(duration_seconds), false)
    } else {
        (
            time_seconds.min(duration_seconds),
            time_seconds >= duration_seconds,
        )
    }
}

fn apply_timeline_sequence_at(
    state: &mut SoundEngineState,
    sequence: &SoundTimelineSequence,
    time_seconds: f32,
) -> Result<Vec<SoundTimelineAutomationSample>, SoundError> {
    let track_count = sequence.tracks.len();
    let mut samples = Vec::with_capacity(track_count);
    let mut applications = Vec::with_capacity(track_count);
    for track in &sequence.tracks {
        let value = sample_automation_curve(&track.curve, time_seconds)?;
        let binding = state.automation_bindings.get(&track.binding).ok_or(
            SoundError::UnknownAutomationBinding {
                binding: track.binding,
            },
        )?;
        samples.push(SoundTimelineAutomationSample {
            binding: track.binding,
            value,
        });
        applications.push((binding.target.clone(), binding.parameter.clone(), value));
    }
    for (target, parameter, value) in applications {
        apply_automation_target(state, target, &parameter, value)?;
    }
    Ok(samples)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::core::framework::sound::{
        SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationCurve,
        SoundAutomationKeyframe, SoundAutomationTarget, SoundParameterId,
        SoundTimelineAutomationTrack, SoundTimelineSequenceId,
    };

    use crate::timeline::playback::SoundTimelineSequencePlayback;
    use crate::SoundConfig;

    use super::*;

    const BENCHMARK_SAMPLE_PAIRS: usize = 21;
    const BENCHMARK_SEQUENCE_COUNT: usize = 300;
    const BENCHMARK_TRACKS_PER_SEQUENCE: usize = 10;
    const BENCHMARK_UNUSED_PATH_BYTES: usize = 256;

    #[test]
    fn timeline_capacity_preserves_legacy_reports_and_state() {
        let mut legacy = timeline_state(7, 3, 32, true);
        let mut optimized = timeline_state(7, 3, 32, true);

        let legacy_reports = legacy_advance_timeline_sequences(&mut legacy, 0.75).unwrap();
        let optimized_reports = advance_timeline_sequences(&mut optimized, 0.75).unwrap();

        assert_eq!(optimized_reports, legacy_reports);
        assert_eq!(optimized.parameters, legacy.parameters);
        assert_eq!(optimized.timeline_sequences.len(), 4);
        assert_eq!(
            optimized.timeline_sequences.len(),
            legacy.timeline_sequences.len()
        );
        for (optimized_playback, legacy_playback) in optimized
            .timeline_sequences
            .iter()
            .zip(&legacy.timeline_sequences)
        {
            assert_eq!(optimized_playback.sequence, legacy_playback.sequence);
            assert_eq!(
                optimized_playback.time_seconds,
                legacy_playback.time_seconds
            );
        }
    }

    #[test]
    fn timeline_capacity_matches_known_sequence_and_track_counts() {
        let mut state = timeline_state(11, 5, 32, false);

        let reports = advance_timeline_sequences(&mut state, 0.25).unwrap();

        assert_eq!(reports.len(), 11);
        assert_eq!(reports.capacity(), 11);
        assert_eq!(state.timeline_sequences.len(), 11);
        assert_eq!(state.timeline_sequences.capacity(), 11);
        assert!(reports
            .iter()
            .all(|report| report.samples.len() == 5 && report.samples.capacity() == 5));
    }

    #[test]
    #[ignore = "release-only performance gate"]
    fn timeline_capacity_release_gate() {
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
        let unused_path_clone_count = BENCHMARK_SEQUENCE_COUNT * BENCHMARK_TRACKS_PER_SEQUENCE;
        let unused_path_clone_bytes = unused_path_clone_count * BENCHMARK_UNUSED_PATH_BYTES;

        println!(
            "PERF_RESULT task=plugins11_bounded_timeline_capacity sequences={BENCHMARK_SEQUENCE_COUNT} tracks_per_sequence={BENCHMARK_TRACKS_PER_SEQUENCE} sample_pairs={BENCHMARK_SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_unused_path_clone_allocations={unused_path_clone_count} optimized_unused_path_clone_allocations=0 legacy_unused_path_clone_bytes={unused_path_clone_bytes} optimized_unused_path_clone_bytes=0 threshold_percent=15 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={legacy_ns} optimized_raw_ns={optimized_ns}"
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(85),
            "optimized P95 {optimized_p95}ns must be at least 15% faster than legacy P95 {legacy_p95}ns"
        );
    }

    fn timeline_state(
        sequence_count: usize,
        tracks_per_sequence: usize,
        unused_path_bytes: usize,
        alternate_completion: bool,
    ) -> SoundEngineState {
        let mut state = SoundEngineState::new(&SoundConfig::default());
        let tracks = (0..tracks_per_sequence)
            .map(|track_index| {
                let binding = SoundAutomationBindingId::new(track_index as u64 + 1);
                let target_parameter =
                    SoundParameterId::new(format!("timeline.capacity.target.{track_index}"));
                state.automation_bindings.insert(
                    binding,
                    SoundAutomationBinding {
                        id: binding,
                        timeline_track_path: "p".repeat(unused_path_bytes),
                        target: SoundAutomationTarget::SynthParameter(target_parameter),
                        parameter: SoundParameterId::new("value"),
                    },
                );
                SoundTimelineAutomationTrack {
                    binding,
                    curve: SoundAutomationCurve::from_keyframes([
                        SoundAutomationKeyframe::linear(0.0, 0.0),
                        SoundAutomationKeyframe::linear(1.0, 1.0),
                    ]),
                }
            })
            .collect::<Vec<_>>();
        state.timeline_sequences = (0..sequence_count)
            .map(|sequence_index| SoundTimelineSequencePlayback {
                sequence: SoundTimelineSequence::new(
                    SoundTimelineSequenceId::new(format!("timeline-capacity-{sequence_index}")),
                    1.0,
                    !alternate_completion || sequence_index % 2 == 0,
                    tracks.clone(),
                ),
                time_seconds: 0.5,
            })
            .collect();
        state
    }

    fn legacy_advance_timeline_sequences(
        state: &mut SoundEngineState,
        delta_seconds: f32,
    ) -> Result<Vec<SoundTimelineSequenceAdvance>, SoundError> {
        let mut scheduled = std::mem::take(&mut state.timeline_sequences);
        let mut retained = Vec::new();
        let mut reports = Vec::new();
        for mut playback in scheduled.drain(..) {
            let raw_time = playback.time_seconds + delta_seconds;
            let (sample_time, completed) = resolve_sample_time(
                playback.sequence.duration_seconds,
                raw_time,
                playback.sequence.looping,
            );
            let samples =
                legacy_apply_timeline_sequence_at(state, &playback.sequence, sample_time)?;
            reports.push(SoundTimelineSequenceAdvance {
                sequence: playback.sequence.id.clone(),
                time_seconds: sample_time,
                completed,
                samples,
            });
            if !completed {
                playback.time_seconds = sample_time;
                retained.push(playback);
            }
        }
        state.timeline_sequences = retained;
        Ok(reports)
    }

    fn legacy_apply_timeline_sequence_at(
        state: &mut SoundEngineState,
        sequence: &SoundTimelineSequence,
        time_seconds: f32,
    ) -> Result<Vec<SoundTimelineAutomationSample>, SoundError> {
        let mut samples = Vec::new();
        let mut applications = Vec::new();
        for track in &sequence.tracks {
            let value = sample_automation_curve(&track.curve, time_seconds)?;
            let binding = state
                .automation_bindings
                .get(&track.binding)
                .cloned()
                .ok_or(SoundError::UnknownAutomationBinding {
                    binding: track.binding,
                })?;
            samples.push(SoundTimelineAutomationSample {
                binding: track.binding,
                value,
            });
            applications.push((binding.target, binding.parameter, value));
        }
        for (target, parameter, value) in applications {
            apply_automation_target(state, target, &parameter, value)?;
        }
        Ok(samples)
    }

    fn legacy_benchmark_sample() -> u128 {
        benchmark_sample(legacy_advance_timeline_sequences)
    }

    fn optimized_benchmark_sample() -> u128 {
        benchmark_sample(advance_timeline_sequences)
    }

    fn benchmark_sample(
        operation: fn(
            &mut SoundEngineState,
            f32,
        ) -> Result<Vec<SoundTimelineSequenceAdvance>, SoundError>,
    ) -> u128 {
        let mut state = timeline_state(
            BENCHMARK_SEQUENCE_COUNT,
            BENCHMARK_TRACKS_PER_SEQUENCE,
            BENCHMARK_UNUSED_PATH_BYTES,
            false,
        );
        let started = Instant::now();
        let reports = black_box(operation(black_box(&mut state), 0.25).unwrap());
        let elapsed = started.elapsed().as_nanos();
        assert_eq!(reports.len(), BENCHMARK_SEQUENCE_COUNT);
        assert_eq!(state.timeline_sequences.len(), BENCHMARK_SEQUENCE_COUNT);
        black_box((reports, state.parameters));
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
