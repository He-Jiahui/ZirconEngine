use crate::core::framework::render::RenderPhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{MeshBatchRef, MeshDrawCommand};

use super::{non_material_rebuild, PendingMeshCommandCacheExtractionStats};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingMeshCommandCacheResidualReason {
    MaterialPhase,
    RebuildInputMissing,
    RebuildRejected,
}

pub(super) fn rebuild_non_material_command_or_record_residual(
    rebuild_batch_for_phase: &mut impl FnMut(RenderPhase) -> Option<MeshBatchRef>,
    phase: RenderPhase,
    extraction_stats: &mut Option<&mut PendingMeshCommandCacheExtractionStats>,
) -> Option<MeshDrawCommand> {
    match rebuild_non_material_command(rebuild_batch_for_phase, phase) {
        Ok(command) => Some(command),
        Err(reason) => {
            record_residual_reason(extraction_stats, reason);
            None
        }
    }
}

fn rebuild_non_material_command(
    rebuild_batch_for_phase: &mut impl FnMut(RenderPhase) -> Option<MeshBatchRef>,
    phase: RenderPhase,
) -> Result<MeshDrawCommand, PendingMeshCommandCacheResidualReason> {
    if !non_material_rebuild::can_rebuild_non_material_command_phase(phase) {
        return Err(PendingMeshCommandCacheResidualReason::MaterialPhase);
    }
    let rebuild_batch = rebuild_batch_for_phase(phase)
        .ok_or(PendingMeshCommandCacheResidualReason::RebuildInputMissing)?;
    non_material_rebuild::rebuild_non_material_command(&rebuild_batch, phase)
        .ok_or(PendingMeshCommandCacheResidualReason::RebuildRejected)
}

fn record_residual_reason(
    extraction_stats: &mut Option<&mut PendingMeshCommandCacheExtractionStats>,
    reason: PendingMeshCommandCacheResidualReason,
) {
    let Some(stats) = extraction_stats.as_mut() else {
        return;
    };
    match reason {
        PendingMeshCommandCacheResidualReason::MaterialPhase => {
            stats.residual_material_phase_draw_count += 1;
        }
        PendingMeshCommandCacheResidualReason::RebuildInputMissing => {
            stats.residual_rebuild_input_missing_draw_count += 1;
        }
        PendingMeshCommandCacheResidualReason::RebuildRejected => {
            stats.residual_rebuild_rejected_draw_count += 1;
        }
    }
}
