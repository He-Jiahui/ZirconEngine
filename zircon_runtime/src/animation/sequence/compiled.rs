use std::sync::Arc;

use crate::core::framework::animation::compiler::sequence::{
    AnimationCompiledSequence, AnimationCompiledSequenceTrack,
};
use crate::core::framework::animation::{
    AnimationChannelValueAsset, AnimationResult, AnimationTrackPath,
};
use crate::core::math::Real;
use crate::scene::world::{CompiledScenePropertyWriter, SceneResult, World};
use crate::scene::SceneError;

use super::channel_sample::AnimationCompiledSequenceTrackSampleExt;
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
    source: Arc<AnimationCompiledSequence>,
    tracks: Vec<CompiledAnimationSequenceTrack>,
    missing_tracks: Vec<AnimationTrackPath>,
    unresolved: Arc<[(usize, usize)]>,
    owner: crate::scene::world::DetachPreparationOwner,
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
    source: AnimationCompiledSequence,
) -> SceneResult<CompiledAnimationSequence> {
    compile_shared(world, Arc::new(source))
}

fn compile_shared(
    world: &mut World,
    source: Arc<AnimationCompiledSequence>,
) -> SceneResult<CompiledAnimationSequence> {
    let duration_seconds = source.duration_seconds();
    let binding_catalog_generation = world.scene_binding_catalog_generation();
    let source_track_capacity = source.track_count();
    let mut tracks = Vec::with_capacity(source_track_capacity);
    let mut missing_tracks = Vec::new();
    let mut unresolved = Vec::new();

    for (binding_index, binding) in source.bindings().iter().enumerate() {
        let target = binding
            .target_id()
            .and_then(|target_id| resolve_sequence_target_id(world, target_id))
            .or_else(|| world.get_entity_by_path(binding.entity_path()));
        let target = target.and_then(|entity| world.entity_path(entity).map(|path| (entity, path)));
        let Some((entity, canonical_entity_path)) = target else {
            if missing_tracks.is_empty() {
                missing_tracks.reserve(source_track_capacity);
            }
            for (track_index, track) in binding.tracks().iter().enumerate() {
                unresolved.push((binding_index, track_index));
                missing_tracks.push(AnimationTrackPath::new(
                    binding.entity_path().clone(),
                    track.property_path().clone(),
                ));
            }
            continue;
        };

        for (track_index, track) in binding.tracks().iter().enumerate() {
            let Some(writer) = world.compile_scene_property_writer_for_entity(
                entity,
                &canonical_entity_path,
                track.property_path(),
            )?
            else {
                if missing_tracks.is_empty() {
                    missing_tracks.reserve(source_track_capacity);
                }
                unresolved.push((binding_index, track_index));
                missing_tracks.push(AnimationTrackPath::new(
                    binding.entity_path().clone(),
                    track.property_path().clone(),
                ));
                continue;
            };
            tracks.push(CompiledAnimationSequenceTrack {
                binding_index,
                track_index,
                writer,
            });
        }
    }

    Ok(CompiledAnimationSequence {
        duration_seconds,
        binding_catalog_generation,
        source,
        tracks,
        missing_tracks,
        unresolved: unresolved.into(),
        owner: world.owner_identity().clone(),
    })
}

impl CompiledAnimationSequence {
    #[doc(hidden)]
    pub fn from_validated_playback_source(
        world: &mut World,
        source: Arc<AnimationCompiledSequence>,
    ) -> SceneResult<Self> {
        compile_shared(world, source)
    }

    #[doc(hidden)]
    pub fn playback_binding_catalog(world: &World) -> u64 {
        world.scene_binding_catalog_generation()
    }

    /// Source-local slots which still lack a writer or were rejected as stale by the writer
    /// guard. A changed owner stops playback and receives no settlement. Illegal samples and
    /// other writer errors are not late-binding debt.
    #[doc(hidden)]
    pub fn apply_playback_slots(
        &self,
        world: &mut World,
        time_seconds: Real,
        looping: bool,
        only: Option<&Arc<[(usize, usize)]>>,
        owner_live: impl Fn(&World) -> bool,
    ) -> Option<(Option<u64>, Arc<[(usize, usize)]>)> {
        let mut attempted = Some(Self::playback_binding_catalog(world));
        let time = resolve_sequence_sample_time(self.duration_seconds, time_seconds, looping);
        let live =
            |world: &World| self.owner.is_same_world(world.owner_identity()) && owner_live(world);
        if let Some(only) = only {
            let mut remaining: Option<Vec<_>> = None;
            for (index, &slot) in only.iter().enumerate() {
                if !live(world) {
                    return None;
                }
                let row = self
                    .tracks
                    .binary_search_by_key(&slot, |track| (track.binding_index, track.track_index));
                let keep = match row.map(|row| apply_track(world, self, &self.tracks[row], time)) {
                    Err(_) => true,
                    Ok(Ok(WriteOutcome::Stale)) => {
                        attempted = None;
                        true
                    }
                    // Only this selected slot is terminal; an independent selected slot still
                    // owns its original debt. Never add new slots on the selective path.
                    Ok(_) => false,
                };
                if keep {
                    if let Some(remaining) = &mut remaining {
                        remaining.push(slot);
                    }
                } else {
                    remaining.get_or_insert_with(|| only[..index].to_vec());
                }
            }
            return live(world).then(|| {
                (
                    attempted,
                    remaining.map_or_else(|| Arc::clone(only), Arc::from),
                )
            });
        }
        let mut retry = Vec::new();
        let mut halted_at = None;
        for track in &self.tracks {
            if !live(world) {
                return None;
            }
            let slot = (track.binding_index, track.track_index);
            match apply_track(world, self, track, time) {
                Ok(WriteOutcome::Stale) => {
                    attempted = None;
                    retry.push(slot);
                }
                // Preserve the ordinary conversion-error stop. Its unvisited suffix is not
                // a successful write, but also not a topology retry for the same bad input.
                Err(_) => {
                    halted_at = Some(slot);
                    break;
                }
                Ok(_) => (),
            }
        }
        live(world).then(|| {
            (
                attempted,
                if retry.is_empty() && halted_at.is_none() {
                    Arc::clone(&self.unresolved)
                } else {
                    retry.extend(
                        self.unresolved
                            .iter()
                            .copied()
                            .take_while(|slot| halted_at.is_none_or(|stop| *slot < stop)),
                    );
                    retry.sort_unstable();
                    retry.into()
                },
            )
        })
    }

    /// Cross-incarnation correspondence needs the old validated source, not just a matching
    /// asset revision, registry hash or new projection. Steps can carry unused NaN tangents.
    #[doc(hidden)]
    pub fn has_playback_source(&self, other: &Arc<AnimationCompiledSequence>) -> bool {
        Arc::ptr_eq(&self.source, other) || source_matches(&self.source, other)
    }

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
        self.owner.is_same_world(world.owner_identity())
            && (self.missing_tracks.is_empty()
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

    for track in &compiled.tracks {
        match apply_track(world, compiled, track, sample_time)? {
            WriteOutcome::Accepted => stats.applied_tracks = stats.applied_tracks.saturating_add(1),
            _ => stats.missing_tracks = stats.missing_tracks.saturating_add(1),
        }
    }
    Ok(stats)
}

enum WriteOutcome {
    Accepted,
    Stale,
    Rejected,
}

fn apply_track(
    world: &mut World,
    compiled: &CompiledAnimationSequence,
    track: &CompiledAnimationSequenceTrack,
    time: Real,
) -> AnimationResult<WriteOutcome> {
    let Some(sample) = compiled
        .source
        .bindings()
        .get(track.binding_index)
        .and_then(|binding| binding.tracks().get(track.track_index))
        .and_then(|source| source.sample_compiled(time))
    else {
        return Ok(WriteOutcome::Rejected);
    };
    let value = scene_property_value_from_channel(&sample)?;
    Ok(
        match world.write_compiled_scene_property(&track.writer, value) {
            Ok(_) => WriteOutcome::Accepted,
            // This variant is emitted by the compiled writer's initial freshness check, before
            // any typed field write. Missing components, mismatched values and other errors differ.
            Err(SceneError::PropertyUnavailable { .. }) if !track.writer.is_current_for(world) => {
                WriteOutcome::Stale
            }
            Err(_) => WriteOutcome::Rejected,
        },
    )
}

fn source_matches(a: &AnimationCompiledSequence, b: &AnimationCompiledSequence) -> bool {
    use AnimationChannelValueAsset::*;
    fn numbers(a: &[Real], b: &[Real]) -> bool {
        a.iter().zip(b).all(|(a, b)| a.to_bits() == b.to_bits())
    }
    fn value(
        a: Option<&AnimationChannelValueAsset>,
        b: Option<&AnimationChannelValueAsset>,
    ) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(Bool(a)), Some(Bool(b))) => a == b,
            (Some(Integer(a)), Some(Integer(b))) => a == b,
            (Some(Scalar(a)), Some(Scalar(b))) => a.to_bits() == b.to_bits(),
            (Some(Vec2(a)), Some(Vec2(b))) => numbers(a, b),
            (Some(Vec3(a)), Some(Vec3(b))) => numbers(a, b),
            (Some(Vec4(a)), Some(Vec4(b))) | (Some(Quaternion(a)), Some(Quaternion(b))) => {
                numbers(a, b)
            }
            _ => false,
        }
    }
    a.duration_seconds().to_bits() == b.duration_seconds().to_bits()
        && a.frames_per_second().to_bits() == b.frames_per_second().to_bits()
        && a.bindings().len() == b.bindings().len()
        && a.bindings().iter().zip(b.bindings()).all(|(a, b)| {
            a.entity_path() == b.entity_path()
                && a.target_id() == b.target_id()
                && a.tracks().len() == b.tracks().len()
                && a.tracks().iter().zip(b.tracks()).all(|(a, b)| {
                    a.property_path() == b.property_path()
                        && a.interpolation() == b.interpolation()
                        && a.value_kind() == b.value_kind()
                        && a.keys().len() == b.keys().len()
                        && a.keys().iter().zip(b.keys()).all(|(a, b)| {
                            a.time_seconds().to_bits() == b.time_seconds().to_bits()
                                && value(Some(a.value()), Some(b.value()))
                                && value(a.in_tangent(), b.in_tangent())
                                && value(a.out_tangent(), b.out_tangent())
                        })
                })
        })
}

#[cfg(test)]
#[path = "tests/compiled_missing_track_capacity.rs"]
mod compiled_missing_track_capacity_tests;
