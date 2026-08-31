use std::collections::HashSet;

use crate::core::framework::render::{RenderPhase, ShaderQualityTier};
use crate::core::TaskPool;

use super::super::super::cached_mesh_draw_commands::{CachedMeshDrawCommands, CachedMeshDrawKey};
use super::super::super::mesh_pass_processor::MeshBatchRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ParallelPreparationMode {
    Parallel,
    SingleWorker,
    SmallBatch,
    DuplicateCacheKey,
}

impl ParallelPreparationMode {
    pub(super) fn select(
        batches: &[MeshBatchRef],
        shader_quality: ShaderQualityTier,
        task_pool: &TaskPool,
    ) -> Self {
        if task_pool.parallelism() <= 1 {
            return Self::SingleWorker;
        }
        if batches.len() < 2 {
            return Self::SmallBatch;
        }
        if has_duplicate_cache_keys(batches, shader_quality) {
            return Self::DuplicateCacheKey;
        }
        Self::Parallel
    }

    pub(super) const fn is_parallel(self) -> bool {
        matches!(self, Self::Parallel)
    }

    pub(super) const fn profile_code(self) -> u8 {
        match self {
            Self::Parallel => 0,
            Self::SingleWorker => 1,
            Self::SmallBatch => 2,
            Self::DuplicateCacheKey => 3,
        }
    }
}

pub(super) fn should_prepare_batches_in_parallel(
    batches: &[MeshBatchRef],
    task_pool: &TaskPool,
) -> bool {
    ParallelPreparationMode::select(batches, ShaderQualityTier::default(), task_pool).is_parallel()
}

fn has_duplicate_cache_keys(batches: &[MeshBatchRef], shader_quality: ShaderQualityTier) -> bool {
    let mut keys = HashSet::new();
    for batch in batches {
        for phase in [
            RenderPhase::Prepass,
            RenderPhase::Shadow,
            RenderPhase::Opaque3d,
            RenderPhase::AlphaMask3d,
        ] {
            if !CachedMeshDrawCommands::is_cacheable_batch_phase(batch, phase) {
                continue;
            }
            if let Some(key) = CachedMeshDrawKey::from_batch_phase(batch, phase, shader_quality) {
                if !keys.insert(key) {
                    return true;
                }
            }
        }
    }
    false
}
