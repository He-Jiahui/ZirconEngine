use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::EditorPlugin;
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceBindingAsset, AnimationSequenceTrackAsset,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};
use zircon_runtime::plugin::PluginModuleKind;

#[test]
fn timeline_authoring_registration_exposes_menu_items_and_payload_schemas() {
    let mut registry = zircon_editor::core::editor_extension::EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("timeline authoring registration");
    let operation =
        EditorOperationPath::parse("timeline_sequence.keyframe.move").expect("timeline operation");
    let descriptor = registry
        .commands()
        .command(&operation)
        .expect("move keyframe operation registered");

    assert_eq!(
        descriptor
            .menu_path()
            .expect("move command menu path")
            .stable_path(),
        "plugins/timeline_sequence/timeline_sequence.keyframe.move"
    );
    assert_eq!(
        descriptor.payload_schema_id(),
        Some("timeline_sequence.move_keyframe.v1")
    );
    assert!(registry.menu_items().next().is_none());
}

#[test]
fn timeline_sequence_package_manifest_declares_editor_only_metadata() {
    let manifest = package_manifest();
    let editor_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == zircon_runtime::plugin::PluginModuleKind::Editor)
        .expect("timeline sequence editor module");

    assert_eq!(manifest.category, "authoring");
    assert_eq!(
        manifest.supported_targets,
        vec![zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost]
    );
    assert_eq!(manifest.capabilities, vec![CAPABILITY.to_string()]);
    assert_eq!(editor_module.capabilities, manifest.capabilities);
}

#[test]
fn timeline_sequence_package_manifest_declares_editor_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest
        .default_packaging
        .contains(&ExportPackagingStrategy::NativeDynamic));
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("timeline_sequence declares standalone distribution");
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.dist_crate, TIMELINE_SEQUENCE_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert!(distribution.runtime_entry.is_empty());
    assert_eq!(
        distribution.editor_entry,
        TIMELINE_SEQUENCE_DIST_EDITOR_ENTRY
    );

    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "timeline_sequence.dist")
        .expect("timeline_sequence dist module is declared");
    assert_eq!(dist_module.kind, PluginModuleKind::Native);
    assert_eq!(dist_module.crate_name, TIMELINE_SEQUENCE_DIST_CRATE_NAME);
    assert_eq!(
        dist_module.target_modes,
        vec![RuntimeTargetMode::EditorHost]
    );
    assert_eq!(dist_module.capabilities, vec![CAPABILITY.to_string()]);
}

#[test]
fn timeline_sequence_validation_accepts_sorted_keyframes_in_range() {
    let sequence = sequence_with_keys([0.0, 0.5, 1.0]);

    assert!(validate_timeline_sequence(&sequence).is_empty());
}

#[test]
fn timeline_sequence_validation_reports_range_and_sorting_errors() {
    let sequence = sequence_with_keys([0.75, 0.25, 1.5]);

    let diagnostics = validate_timeline_sequence(&sequence);

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("outside timeline range")));
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("must be sorted by time")));
}

#[test]
fn timeline_track_paths_are_sorted_for_deterministic_authoring() {
    let sequence = AnimationSequenceAsset {
        name: Some("Timeline".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![
            binding("root/z", "Transform.translation"),
            binding("root/a", "Transform.translation"),
        ],
    };

    assert_eq!(
        sorted_timeline_track_paths(&sequence),
        vec![
            "root/a:Transform.translation".to_string(),
            "root/z:Transform.translation".to_string()
        ]
    );
}

#[test]
fn timeline_keyframe_move_updates_time_and_restores_track_sort_order() {
    let mut sequence = sequence_with_keys([0.0, 0.25, 1.0]);

    move_timeline_keyframe(
        &mut sequence,
        &TimelineKeyframeMoveRequest {
            binding_index: 0,
            track_index: 0,
            key_index: 0,
            new_time_seconds: 0.75,
        },
    )
    .expect("keyframe move is valid");

    let times = sequence.bindings[0].tracks[0]
        .channel
        .keys
        .iter()
        .map(|key| key.time_seconds)
        .collect::<Vec<_>>();
    assert_eq!(times, vec![0.25, 0.75, 1.0]);
}

#[test]
fn timeline_keyframe_move_reports_bad_indices_and_time_range() {
    let mut sequence = sequence_with_keys([0.0, 0.25, 1.0]);

    let diagnostics = move_timeline_keyframe(
        &mut sequence,
        &TimelineKeyframeMoveRequest {
            binding_index: 0,
            track_index: 0,
            key_index: 5,
            new_time_seconds: 2.0,
        },
    )
    .expect_err("keyframe index and time are invalid");

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("outside timeline range")));
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("keyframe index 5")));
}

#[test]
fn timeline_keyframe_move_failure_preserves_the_entire_sequence() {
    let mut sequence = sequence_with_keys([0.0, 0.25, 1.0]);
    sequence
        .bindings
        .push(binding("root/invalid", "Transform.scale"));
    sequence.bindings[1].tracks[0].channel.keys = vec![AnimationChannelKeyAsset {
        time_seconds: 2.0,
        value: AnimationChannelValueAsset::Vec3([9.0, 0.0, 0.0]),
        in_tangent: None,
        out_tangent: None,
    }];
    let before = sequence.clone();

    let diagnostics = move_timeline_keyframe(
        &mut sequence,
        &TimelineKeyframeMoveRequest {
            binding_index: 0,
            track_index: 0,
            key_index: 0,
            new_time_seconds: 0.75,
        },
    )
    .expect_err("an invalid sequence cannot publish a partial move");

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("outside timeline range")));
    assert_eq!(sequence, before);
}

#[test]
fn timeline_keyframe_move_rejects_non_finite_time_without_mutation() {
    for new_time_seconds in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut sequence = sequence_with_keys([0.0, 0.25, 1.0]);
        let before = sequence.clone();

        let diagnostics = move_timeline_keyframe(
            &mut sequence,
            &TimelineKeyframeMoveRequest {
                binding_index: 0,
                track_index: 0,
                key_index: 1,
                new_time_seconds,
            },
        )
        .expect_err("non-finite keyframe time must fail closed");

        assert!(diagnostics
            .iter()
            .any(|message| message.contains("must be finite")));
        assert_eq!(sequence, before);
    }
}

#[test]
fn timeline_keyframe_move_preserves_stable_order_for_equal_times() {
    let mut sequence =
        sequence_with_tagged_keys(&[(0.0, 0.0), (0.5, 1.0), (0.75, 2.0), (0.75, 3.0), (1.0, 4.0)]);

    move_timeline_keyframe(
        &mut sequence,
        &TimelineKeyframeMoveRequest {
            binding_index: 0,
            track_index: 0,
            key_index: 1,
            new_time_seconds: 0.75,
        },
    )
    .expect("equal-time move is valid");

    assert_eq!(key_tags(&sequence), vec![0.0, 1.0, 2.0, 3.0, 4.0]);
}

#[test]
#[ignore = "release performance gate"]
fn timeline_keyframe_move_release_gate_uses_atomic_binary_insertion() {
    const SAMPLE_PAIRS: usize = 21;
    const KEYS_PER_TRACK: usize = 16_384;
    const MOVES_PER_SAMPLE: usize = 32;
    const REQUIRED_IMPROVEMENT_PERCENT: u128 = 20;

    let base = large_sequence(KEYS_PER_TRACK);
    let request = TimelineKeyframeMoveRequest {
        binding_index: 0,
        track_index: 0,
        key_index: 0,
        new_time_seconds: (KEYS_PER_TRACK - 2) as f32,
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_move_batch(
                &base,
                &request,
                MOVES_PER_SAMPLE,
                legacy_move_timeline_keyframe,
            ));
            optimized_samples.push(measure_move_batch(
                &base,
                &request,
                MOVES_PER_SAMPLE,
                move_timeline_keyframe,
            ));
        } else {
            optimized_samples.push(measure_move_batch(
                &base,
                &request,
                MOVES_PER_SAMPLE,
                move_timeline_keyframe,
            ));
            legacy_samples.push(measure_move_batch(
                &base,
                &request,
                MOVES_PER_SAMPLE,
                legacy_move_timeline_keyframe,
            ));
        }
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples).as_nanos();
    let optimized_p95 = nearest_rank_p95(&optimized_samples).as_nanos();
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins08_timeline_atomic_move sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even keys_per_track={KEYS_PER_TRACK} moves_per_sample={MOVES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT}",
        durations_csv(&legacy_samples),
        durations_csv(&optimized_samples)
    );
    assert!(
        improvement_percent >= REQUIRED_IMPROVEMENT_PERCENT,
        "atomic binary insertion must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
    );
}

#[test]
fn timeline_event_marker_payload_validation_rejects_empty_event_and_bad_payload_key() {
    let marker = TimelineEventMarker {
        time_seconds: 1.0,
        event: " ".to_string(),
        payload: BTreeMap::new(),
    };
    assert!(validate_event_marker_payload(&marker, 1.0)
        .expect_err("event name is required")
        .contains("must name an event"));

    let mut payload = BTreeMap::new();
    payload.insert(" ".to_string(), "value".to_string());
    let marker = TimelineEventMarker {
        time_seconds: 0.5,
        event: "Footstep".to_string(),
        payload: payload.clone(),
    };
    assert!(validate_event_marker_payload(&marker, 1.0)
        .expect_err("payload keys are checked")
        .contains("payload keys must not be empty"));

    let marker = TimelineEventMarker {
        time_seconds: 2.0,
        event: "Footstep".to_string(),
        payload,
    };
    assert!(validate_event_marker_payload(&marker, 1.0)
        .expect_err("event time range is checked")
        .contains("outside timeline range"));
}

fn sequence_with_keys(times: [f32; 3]) -> AnimationSequenceAsset {
    let mut binding = binding("root/player", "Transform.translation");
    binding.tracks[0].channel.keys = times
        .into_iter()
        .map(|time_seconds| AnimationChannelKeyAsset {
            time_seconds,
            value: AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0]),
            in_tangent: None,
            out_tangent: None,
        })
        .collect();
    AnimationSequenceAsset {
        name: Some("Timeline".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![binding],
    }
}

fn sequence_with_tagged_keys(keys: &[(f32, f32)]) -> AnimationSequenceAsset {
    let mut sequence = sequence_with_keys([0.0, 0.25, 1.0]);
    sequence.duration_seconds = keys
        .iter()
        .map(|(time_seconds, _)| *time_seconds)
        .fold(1.0_f32, f32::max);
    sequence.bindings[0].tracks[0].channel.keys = keys
        .iter()
        .map(|(time_seconds, tag)| AnimationChannelKeyAsset {
            time_seconds: *time_seconds,
            value: AnimationChannelValueAsset::Vec3([*tag, 0.0, 0.0]),
            in_tangent: None,
            out_tangent: None,
        })
        .collect();
    sequence
}

fn key_tags(sequence: &AnimationSequenceAsset) -> Vec<f32> {
    sequence.bindings[0].tracks[0]
        .channel
        .keys
        .iter()
        .map(|key| match &key.value {
            AnimationChannelValueAsset::Vec3(value) => value[0],
            _ => panic!("fixture uses Vec3 key values"),
        })
        .collect()
}

fn large_sequence(key_count: usize) -> AnimationSequenceAsset {
    let keys = (0..key_count)
        .map(|index| (index as f32, index as f32))
        .collect::<Vec<_>>();
    sequence_with_tagged_keys(&keys)
}

fn legacy_move_timeline_keyframe(
    sequence: &mut AnimationSequenceAsset,
    request: &TimelineKeyframeMoveRequest,
) -> Result<(), Vec<String>> {
    let key = &mut sequence.bindings[request.binding_index].tracks[request.track_index]
        .channel
        .keys[request.key_index];
    key.time_seconds = request.new_time_seconds;
    sequence.bindings[request.binding_index].tracks[request.track_index]
        .channel
        .keys
        .sort_by(|left, right| left.time_seconds.total_cmp(&right.time_seconds));
    let diagnostics = validate_timeline_sequence(sequence);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn measure_move_batch(
    base: &AnimationSequenceAsset,
    request: &TimelineKeyframeMoveRequest,
    moves_per_sample: usize,
    move_keyframe: fn(
        &mut AnimationSequenceAsset,
        &TimelineKeyframeMoveRequest,
    ) -> Result<(), Vec<String>>,
) -> Duration {
    let mut sequences = (0..moves_per_sample)
        .map(|_| base.clone())
        .collect::<Vec<_>>();
    let started = Instant::now();
    for sequence in &mut sequences {
        black_box(move_keyframe(black_box(sequence), black_box(request)))
            .expect("benchmark move remains valid");
    }
    let elapsed = started.elapsed();
    black_box(sequences);
    elapsed
}

fn nearest_rank_p95(samples: &[Duration]) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[((sorted.len() * 95).div_ceil(100)).saturating_sub(1)]
}

fn durations_csv(samples: &[Duration]) -> String {
    samples
        .iter()
        .map(|sample| sample.as_nanos().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn binding(entity: &str, property: &str) -> AnimationSequenceBindingAsset {
    AnimationSequenceBindingAsset {
        entity_path: EntityPath::parse(entity).unwrap(),
        target_id: None,
        tracks: vec![AnimationSequenceTrackAsset {
            property_path: ComponentPropertyPath::parse(property).unwrap(),
            channel: AnimationChannelAsset {
                interpolation: AnimationInterpolationAsset::Hermite,
                keys: Vec::new(),
            },
        }],
    }
}
