use zircon_runtime::core::framework::sound::{SoundAutomationTarget, SoundError, SoundParameterId};

use crate::automation::values::ensure_finite_value;
use crate::descriptor_validation::listener::validate_listener_descriptor;
use crate::descriptor_validation::source::validate_source_descriptor;
use crate::descriptor_validation::volume::validate_volume_descriptor;
use crate::engine::SoundEngineState;
use crate::kira_bridge::validate_track_controls;

use super::{effect, listener, parameter_values, source, track, volume};

pub(crate) fn apply_automation_target(
    state: &mut SoundEngineState,
    target: SoundAutomationTarget,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    ensure_automation_execution_available(state.kira.is_active())?;
    ensure_finite_value("sound automation value", value)?;
    match target {
        SoundAutomationTarget::Track(track) => {
            let track_index = state
                .graph
                .tracks
                .iter()
                .position(|candidate| candidate.id == track)
                .ok_or(SoundError::UnknownTrack { track })?;
            let track_descriptor = &state.graph.tracks[track_index];
            let mut controls = track_descriptor.controls;
            track::apply_track_parameter(&mut controls, parameter, value)?;
            validate_track_controls(&track_descriptor.display_name, controls)?;
            state.commit_validated_graph_mutation(|graph| {
                graph.tracks[track_index].controls = controls;
            });
            Ok(())
        }
        SoundAutomationTarget::Effect { track, effect } => {
            let track_index = state
                .graph
                .tracks
                .iter()
                .position(|candidate| candidate.id == track)
                .ok_or(SoundError::UnknownTrack { track })?;
            let effect_index = state.graph.tracks[track_index]
                .effects
                .iter()
                .position(|candidate| candidate.id == effect)
                .ok_or(SoundError::UnknownEffect { effect })?;
            let mut effect_descriptor =
                state.graph.tracks[track_index].effects[effect_index].clone();
            effect::apply_effect_parameter(&mut effect_descriptor, parameter, value)?;
            state.commit_validated_graph_mutation(move |graph| {
                graph.tracks[track_index].effects[effect_index] = effect_descriptor;
            });
            Ok(())
        }
        SoundAutomationTarget::Source(source_id) => {
            let mut descriptor = state
                .sources
                .get(&source_id)
                .ok_or(SoundError::UnknownSource { source_id })?
                .descriptor
                .clone();
            source::apply_source_parameter(&mut descriptor, parameter, value)?;
            validate_source_descriptor(state, &descriptor)?;
            state
                .sources
                .get_mut(&source_id)
                .ok_or(SoundError::UnknownSource { source_id })?
                .descriptor = descriptor;
            Ok(())
        }
        SoundAutomationTarget::Listener(listener) => {
            let mut descriptor = state
                .listeners
                .get(&listener)
                .ok_or(SoundError::UnknownListener { listener })?
                .clone();
            listener::apply_listener_parameter(&mut descriptor, parameter, value)?;
            validate_listener_descriptor(state, &descriptor)?;
            state.listeners.insert(listener, descriptor);
            Ok(())
        }
        SoundAutomationTarget::Volume(volume) => {
            let mut descriptor = state
                .volumes
                .get(&volume)
                .ok_or(SoundError::UnknownVolume { volume })?
                .clone();
            volume::apply_volume_parameter(&mut descriptor, parameter, value)?;
            validate_volume_descriptor(&descriptor)?;
            state.volumes.insert(volume, descriptor);
            Ok(())
        }
        SoundAutomationTarget::SynthParameter(target_parameter) => {
            if parameter.as_str() != "value" && parameter.as_str() != target_parameter.as_str() {
                return Err(parameter_values::unsupported_automation_parameter(
                    "synth parameter",
                    parameter,
                ));
            }
            state.parameters.insert(target_parameter, value);
            Ok(())
        }
    }
}

pub(crate) fn ensure_automation_execution_available(kira_active: bool) -> Result<(), SoundError> {
    if kira_active {
        return Err(SoundError::UnsupportedAdvancedFeature(
            "active sound automation execution is enabled by Sound M5".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::sync::Arc;
    use std::time::Instant;

    use zircon_runtime::core::framework::sound::{
        SoundEffectDescriptor, SoundEffectId, SoundEffectKind, SoundGainEffect,
        SoundTrackDescriptor, SoundTrackId,
    };

    use crate::kira_bridge::validate_graph;
    use crate::SoundConfig;

    use super::*;

    const BENCHMARK_SAMPLE_PAIRS: usize = 21;
    const BENCHMARK_TRACKS: usize = 64;
    const BENCHMARK_EFFECTS_PER_TRACK: usize = 4;
    const BENCHMARK_APPLICATIONS_PER_SAMPLE: usize = 512;

    #[test]
    fn track_and_effect_automation_reuse_uniquely_owned_graph_allocation() {
        let mut state = graph_state(4, 2);
        let original_graph = Arc::as_ptr(&state.graph);

        apply_automation_target(
            &mut state,
            SoundAutomationTarget::Track(SoundTrackId::master()),
            &SoundParameterId::new("gain"),
            0.4,
        )
        .unwrap();
        apply_automation_target(
            &mut state,
            SoundAutomationTarget::Effect {
                track: SoundTrackId::master(),
                effect: SoundEffectId::new(1),
            },
            &SoundParameterId::new("wet"),
            0.25,
        )
        .unwrap();

        assert_eq!(Arc::as_ptr(&state.graph), original_graph);
        assert_eq!(state.graph_revision, 2);
        assert_eq!(state.graph.tracks[0].controls.gain, 0.4);
        assert_eq!(state.graph.tracks[0].effects[0].wet, 0.25);
    }

    #[test]
    fn automation_copy_on_write_preserves_published_graph_snapshot() {
        let mut state = graph_state(4, 2);
        let snapshot = state.graph_snapshot();
        let snapshot_graph = Arc::as_ptr(&snapshot.graph);

        apply_automation_target(
            &mut state,
            SoundAutomationTarget::Track(SoundTrackId::master()),
            &SoundParameterId::new("gain"),
            0.3,
        )
        .unwrap();

        assert_eq!(Arc::as_ptr(&snapshot.graph), snapshot_graph);
        assert_ne!(Arc::as_ptr(&state.graph), snapshot_graph);
        assert_eq!(snapshot.revision, 0);
        assert_eq!(snapshot.graph.tracks[0].controls.gain, 1.0);
        assert_eq!(state.graph.tracks[0].controls.gain, 0.3);
        assert_eq!(state.graph_revision, 1);
    }

    #[test]
    fn rejected_automation_keeps_graph_allocation_revision_and_values() {
        let mut state = graph_state(4, 2);
        let original_graph = Arc::as_ptr(&state.graph);

        assert!(apply_automation_target(
            &mut state,
            SoundAutomationTarget::Track(SoundTrackId::master()),
            &SoundParameterId::new("pan"),
            2.0,
        )
        .is_err());
        assert!(apply_automation_target(
            &mut state,
            SoundAutomationTarget::Effect {
                track: SoundTrackId::master(),
                effect: SoundEffectId::new(1),
            },
            &SoundParameterId::new("wet"),
            2.0,
        )
        .is_err());

        assert_eq!(Arc::as_ptr(&state.graph), original_graph);
        assert_eq!(state.graph_revision, 0);
        assert_eq!(state.graph.tracks[0].controls.pan, 0.0);
        assert_eq!(state.graph.tracks[0].effects[0].wet, 1.0);
    }

    #[test]
    #[ignore = "release-only performance gate"]
    fn automation_graph_copy_on_write_release_gate() {
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
            "PERF_RESULT task=runtime08b_automation_graph_cow tracks={BENCHMARK_TRACKS} effects_per_track={BENCHMARK_EFFECTS_PER_TRACK} applications_per_sample={BENCHMARK_APPLICATIONS_PER_SAMPLE} sample_pairs={BENCHMARK_SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_full_graph_clones={BENCHMARK_APPLICATIONS_PER_SAMPLE} optimized_full_graph_clones=0 threshold_percent=25 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={legacy_ns} optimized_raw_ns={optimized_ns}"
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
            "optimized P95 {optimized_p95}ns must be at least 25% faster than legacy P95 {legacy_p95}ns"
        );
    }

    fn graph_state(track_count: usize, effects_per_track: usize) -> SoundEngineState {
        assert!(track_count > 0);
        let mut state = SoundEngineState::new(&SoundConfig::default());
        let graph = Arc::make_mut(&mut state.graph);
        for track_index in 0..track_count {
            if track_index > 0 {
                graph.tracks.push(SoundTrackDescriptor::child(
                    SoundTrackId::new(track_index as u64),
                    format!("Track {track_index}"),
                ));
            }
            let track = &mut graph.tracks[track_index];
            for effect_index in 0..effects_per_track {
                let effect_id = (track_index * effects_per_track + effect_index + 1) as u64;
                track.effects.push(SoundEffectDescriptor::new(
                    SoundEffectId::new(effect_id),
                    format!("Effect {effect_id}"),
                    SoundEffectKind::Gain(SoundGainEffect { gain: 1.0 }),
                ));
            }
        }
        state
    }

    fn legacy_apply_track_automation(
        state: &mut SoundEngineState,
        value: f32,
    ) -> Result<(), SoundError> {
        let mut graph = (*state.graph).clone();
        let track = graph
            .tracks
            .iter_mut()
            .find(|track| track.id == SoundTrackId::master())
            .ok_or(SoundError::UnknownTrack {
                track: SoundTrackId::master(),
            })?;
        track::apply_track_parameter(&mut track.controls, &SoundParameterId::new("gain"), value)?;
        validate_graph(&graph)?;
        state.replace_graph(graph);
        Ok(())
    }

    fn legacy_benchmark_sample() -> u128 {
        benchmark_sample(legacy_apply_track_automation)
    }

    fn optimized_benchmark_sample() -> u128 {
        benchmark_sample(|state, value| {
            apply_automation_target(
                state,
                SoundAutomationTarget::Track(SoundTrackId::master()),
                &SoundParameterId::new("gain"),
                value,
            )
        })
    }

    fn benchmark_sample(
        operation: fn(&mut SoundEngineState, f32) -> Result<(), SoundError>,
    ) -> u128 {
        let mut state = graph_state(BENCHMARK_TRACKS, BENCHMARK_EFFECTS_PER_TRACK);
        let started = Instant::now();
        for index in 0..BENCHMARK_APPLICATIONS_PER_SAMPLE {
            operation(
                black_box(&mut state),
                black_box((index % 101) as f32 / 100.0),
            )
            .unwrap();
        }
        let elapsed = started.elapsed().as_nanos();
        assert_eq!(
            state.graph.tracks[0].controls.gain,
            ((BENCHMARK_APPLICATIONS_PER_SAMPLE - 1) % 101) as f32 / 100.0
        );
        black_box(state.graph);
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
