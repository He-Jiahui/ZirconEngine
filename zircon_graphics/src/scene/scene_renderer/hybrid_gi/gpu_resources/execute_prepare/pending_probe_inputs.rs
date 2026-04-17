use std::collections::BTreeSet;

use zircon_scene::RenderHybridGiExtract;

use crate::types::HybridGiPrepareFrame;

use super::super::gpu_pending_probe_input::GpuPendingProbeInput;
use super::probe_quantization::{
    probe_parent_probe_id, probe_position_x_q, probe_position_y_q, probe_position_z_q,
    probe_radius_q, probe_resident_ancestor,
};

pub(super) fn pending_probe_inputs(
    prepare: &HybridGiPrepareFrame,
    extract: Option<&RenderHybridGiExtract>,
) -> Vec<GpuPendingProbeInput> {
    let resident_probe_ids = prepare
        .resident_probes
        .iter()
        .map(|probe| probe.probe_id)
        .collect::<BTreeSet<_>>();
    prepare
        .pending_updates
        .iter()
        .enumerate()
        .map(|(index, update)| {
            let (resident_ancestor_probe_id, resident_ancestor_depth) =
                probe_resident_ancestor(extract, &resident_probe_ids, update.probe_id);
            GpuPendingProbeInput {
                probe_id: update.probe_id,
                logical_index: prepare.resident_probes.len() as u32 + index as u32,
                ray_budget: update.ray_budget,
                position_x_q: probe_position_x_q(extract, update.probe_id),
                position_y_q: probe_position_y_q(extract, update.probe_id),
                position_z_q: probe_position_z_q(extract, update.probe_id),
                radius_q: probe_radius_q(extract, update.probe_id),
                parent_probe_id: probe_parent_probe_id(extract, update.probe_id),
                resident_ancestor_probe_id,
                resident_ancestor_depth,
            }
        })
        .collect()
}
