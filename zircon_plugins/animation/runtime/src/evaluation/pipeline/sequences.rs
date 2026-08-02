use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::animation::{
    CompiledAnimationSequence, apply_compiled_sequence_to_world, compile_sequence_for_world,
};
use zircon_runtime::asset::AssetId;
use zircon_runtime::core::framework::animation::AnimationSequenceAsset;
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::LevelSystem;

use super::AnimationEvaluationPipeline;

#[derive(Debug)]
pub(super) struct CachedCompiledSequence {
    asset_revision: Option<u64>,
    compiled: CompiledAnimationSequence,
}

pub(super) struct LoadedSequenceSample {
    pub(super) asset_id: AssetId,
    pub(super) asset_revision: Option<u64>,
    pub(super) sequence: AnimationSequenceAsset,
    pub(super) time_seconds: Real,
    pub(super) looping: bool,
}

pub(super) fn apply_loaded_sequences(
    level: &LevelSystem,
    loaded_sequences: &[LoadedSequenceSample],
) {
    level.with_world_mut(|world| {
        let mut sequence_cache = world
            .resource_mut::<AnimationEvaluationPipeline>()
            .take_sequence_cache();
        let requested_assets = loaded_sequences
            .iter()
            .map(|sample| sample.asset_id)
            .collect::<BTreeSet<_>>();

        for sample in loaded_sequences {
            // Without an asset revision, this frame has no durable proof that a
            // retained projection still represents the loaded asset.
            let must_recompile = sample.asset_revision.is_none()
                || sequence_cache.get(&sample.asset_id).map_or(true, |cached| {
                    cached.asset_revision != sample.asset_revision
                        || !cached.compiled.is_current_for(world)
                });
            if must_recompile {
                let Ok(compiled) = compile_sequence_for_world(world, &sample.sequence) else {
                    sequence_cache.remove(&sample.asset_id);
                    continue;
                };
                sequence_cache.insert(
                    sample.asset_id,
                    CachedCompiledSequence {
                        asset_revision: sample.asset_revision,
                        compiled,
                    },
                );
            }

            let Some(cached) = sequence_cache.get(&sample.asset_id) else {
                continue;
            };
            let _ = apply_compiled_sequence_to_world(
                world,
                &sample.sequence,
                &cached.compiled,
                sample.time_seconds,
                sample.looping,
            );
        }

        sequence_cache.retain(|asset_id, _| requested_assets.contains(asset_id));
        world
            .resource_mut::<AnimationEvaluationPipeline>()
            .restore_sequence_cache(sequence_cache);
    });
}
