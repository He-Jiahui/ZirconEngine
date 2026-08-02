use crate::core::framework::animation::{
    AnimationResult, AnimationSequenceAsset, AnimationTrackPath,
};
use crate::core::math::Real;
use crate::scene::world::{CompiledScenePropertyWriter, SceneResult, World};

use super::channel_sample::AnimationChannelSampleExt;
use super::conversion::scene_property_value_from_channel;
use super::target::resolve_sequence_target_id;
use super::time::resolve_sequence_sample_time;

/// Runtime projection of one immutable animation sequence asset for one world.
///
/// The asset revision owner must rebuild this projection when its source asset
/// changes. Frame application validates only the compiled writer's world and
/// schema generations; it never resolves entity/property text again.
#[derive(Clone, Debug)]
pub struct CompiledAnimationSequence {
    duration_seconds: Real,
    binding_catalog_generation: u64,
    tracks: Vec<CompiledAnimationSequenceTrack>,
    missing_tracks: Vec<AnimationTrackPath>,
}

/// Fixed-size outcome of one compiled sequence application.
///
/// Per-track diagnostics remain on [`CompiledAnimationSequence`] so the frame
/// path does not allocate report strings or vectors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompiledAnimationSequenceApplyStats {
    pub applied_tracks: usize,
    pub missing_tracks: usize,
}

#[derive(Clone, Debug)]
struct CompiledAnimationSequenceTrack {
    binding_index: usize,
    track_index: usize,
    writer: CompiledScenePropertyWriter,
}

/// Compiles a sequence's entity and property bindings at an import or edit
/// boundary. Missing targets are retained as report-only tracks so a caller can
/// recompile after the affected hierarchy changes.
pub fn compile_sequence_for_world(
    world: &mut World,
    sequence: &AnimationSequenceAsset,
) -> SceneResult<CompiledAnimationSequence> {
    let mut compiled = CompiledAnimationSequence {
        duration_seconds: sequence.duration_seconds,
        binding_catalog_generation: world.scene_binding_catalog_generation(),
        tracks: Vec::new(),
        missing_tracks: Vec::new(),
    };

    for (binding_index, binding) in sequence.bindings.iter().enumerate() {
        let target = binding
            .target_id
            .as_deref()
            .and_then(|target_id| resolve_sequence_target_id(world, target_id))
            .or_else(|| world.get_entity_by_path(&binding.entity_path));
        let target = target.and_then(|entity| world.entity_path(entity).map(|path| (entity, path)));
        let Some((entity, canonical_entity_path)) = target else {
            compiled
                .missing_tracks
                .extend(binding.tracks.iter().map(|track| {
                    AnimationTrackPath::new(
                        binding.entity_path.clone(),
                        track.property_path.clone(),
                    )
                }));
            continue;
        };

        for (track_index, track) in binding.tracks.iter().enumerate() {
            let track_path =
                AnimationTrackPath::new(binding.entity_path.clone(), track.property_path.clone());
            let Some(writer) = world.compile_scene_property_writer_for_entity(
                entity,
                &canonical_entity_path,
                &track.property_path,
            )?
            else {
                compiled.missing_tracks.push(track_path);
                continue;
            };
            compiled.tracks.push(CompiledAnimationSequenceTrack {
                binding_index,
                track_index,
                writer,
            });
        }
    }

    Ok(compiled)
}

impl CompiledAnimationSequence {
    /// Returns compile-boundary diagnostics without making the apply path own
    /// or clone property-path text each frame.
    pub fn missing_tracks(&self) -> &[AnimationTrackPath] {
        &self.missing_tracks
    }

    /// Reports whether this projection can still be applied to `world`.
    ///
    /// A caller must rebuild after topology makes an initially missing target
    /// eligible for lookup, or when one compiled target/schema generation
    /// becomes stale.
    pub fn is_current_for(&self, world: &World) -> bool {
        (self.missing_tracks.is_empty()
            || self.binding_catalog_generation == world.scene_binding_catalog_generation())
            && self
                .tracks
                .iter()
                .all(|track| track.writer.is_current_for(world))
    }
}

/// Applies a precompiled sequence projection without path normalization,
/// entity-path traversal, or property dispatch.
pub fn apply_compiled_sequence_to_world(
    world: &mut World,
    sequence: &AnimationSequenceAsset,
    compiled: &CompiledAnimationSequence,
    time_seconds: Real,
    looping: bool,
) -> AnimationResult<CompiledAnimationSequenceApplyStats> {
    let mut stats = CompiledAnimationSequenceApplyStats {
        applied_tracks: 0,
        missing_tracks: compiled.missing_tracks.len(),
    };
    let sample_time =
        resolve_sequence_sample_time(compiled.duration_seconds, time_seconds, looping);

    for compiled_track in &compiled.tracks {
        let Some(track) = sequence
            .bindings
            .get(compiled_track.binding_index)
            .and_then(|binding| binding.tracks.get(compiled_track.track_index))
        else {
            stats.missing_tracks = stats.missing_tracks.saturating_add(1);
            continue;
        };
        let Some(sample) = track.channel.sample(sample_time) else {
            stats.missing_tracks = stats.missing_tracks.saturating_add(1);
            continue;
        };
        let value = scene_property_value_from_channel(&sample)?;
        match world.write_compiled_scene_property(&compiled_track.writer, value) {
            Ok(_) => stats.applied_tracks = stats.applied_tracks.saturating_add(1),
            Err(_) => stats.missing_tracks = stats.missing_tracks.saturating_add(1),
        }
    }

    Ok(stats)
}
