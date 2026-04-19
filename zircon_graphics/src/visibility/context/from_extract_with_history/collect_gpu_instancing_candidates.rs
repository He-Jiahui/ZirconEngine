use zircon_framework::scene::Mobility;

use super::super::super::declarations::VisibilityBatch;

pub(super) fn collect_gpu_instancing_candidates(
    visible_batches: &[VisibilityBatch],
) -> Vec<VisibilityBatch> {
    visible_batches
        .iter()
        .filter(|batch| batch.key.mobility == Mobility::Dynamic && batch.entities.len() > 1)
        .cloned()
        .collect()
}

