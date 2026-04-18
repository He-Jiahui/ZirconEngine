use std::collections::BTreeSet;

use zircon_scene::RenderHybridGiExtract;

use crate::types::HybridGiPrepareFrame;

use super::super::gpu_resident_probe_input::GpuResidentProbeInput;
use super::probe_quantization::{
    pack_rgb8, probe_lineage_trace_lighting_rgb, probe_lineage_trace_support_q,
    probe_parent_probe_id, probe_position_x_q, probe_position_y_q, probe_position_z_q,
    probe_radius_q, probe_resident_ancestors,
};

pub(super) fn resident_probe_inputs(
    prepare: &HybridGiPrepareFrame,
    extract: Option<&RenderHybridGiExtract>,
) -> Vec<GpuResidentProbeInput> {
    let resident_probe_ids = prepare
        .resident_probes
        .iter()
        .map(|probe| probe.probe_id)
        .collect::<BTreeSet<_>>();
    prepare
        .resident_probes
        .iter()
        .map(|probe| {
            let [
                (resident_ancestor_probe_id, resident_ancestor_depth),
                (
                    resident_secondary_ancestor_probe_id,
                    resident_secondary_ancestor_depth,
                ),
                (
                    resident_tertiary_ancestor_probe_id,
                    resident_tertiary_ancestor_depth,
                ),
                (
                    resident_quaternary_ancestor_probe_id,
                    resident_quaternary_ancestor_depth,
                ),
            ] = probe_resident_ancestors(extract, &resident_probe_ids, probe.probe_id);
            GpuResidentProbeInput {
                probe_id: probe.probe_id,
                slot: probe.slot,
                ray_budget: probe.ray_budget,
                lineage_trace_support_q: probe_lineage_trace_support_q(
                    extract,
                    &prepare.scheduled_trace_region_ids,
                    probe.probe_id,
                ),
                position_x_q: probe_position_x_q(extract, probe.probe_id),
                position_y_q: probe_position_y_q(extract, probe.probe_id),
                position_z_q: probe_position_z_q(extract, probe.probe_id),
                radius_q: probe_radius_q(extract, probe.probe_id),
                previous_irradiance_rgb: pack_rgb8(probe.irradiance_rgb),
                lineage_trace_lighting_rgb: probe_lineage_trace_lighting_rgb(
                    extract,
                    &prepare.scheduled_trace_region_ids,
                    probe.probe_id,
                ),
                parent_probe_id: probe_parent_probe_id(extract, probe.probe_id),
                resident_ancestor_probe_id,
                resident_ancestor_depth,
                resident_secondary_ancestor_probe_id,
                resident_secondary_ancestor_depth,
                resident_tertiary_ancestor_probe_id,
                resident_tertiary_ancestor_depth,
                resident_quaternary_ancestor_probe_id,
                resident_quaternary_ancestor_depth,
            }
        })
        .collect()
}
