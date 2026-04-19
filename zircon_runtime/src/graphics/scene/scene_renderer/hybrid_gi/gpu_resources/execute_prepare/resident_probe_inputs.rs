use std::collections::BTreeSet;

use crate::core::framework::render::RenderHybridGiExtract;

use crate::graphics::types::{HybridGiPrepareFrame, HybridGiResolveRuntime};

use super::super::gpu_resident_probe_input::GpuResidentProbeInput;
use super::probe_quantization::{
    pack_rgb8, probe_lineage_trace_lighting_rgb, probe_lineage_trace_support_q,
    probe_parent_probe_id, probe_position_x_q, probe_position_y_q, probe_position_z_q,
    probe_radius_q, probe_resident_ancestors,
};
use super::runtime_trace_source::{
    merge_trace_sources, runtime_irradiance_source, runtime_trace_source,
};

pub(super) fn resident_probe_inputs(
    prepare: &HybridGiPrepareFrame,
    resolve_runtime: Option<&HybridGiResolveRuntime>,
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
            let scheduled_trace_support_q = probe_lineage_trace_support_q(
                extract,
                &prepare.scheduled_trace_region_ids,
                probe.probe_id,
            );
            let scheduled_trace_lighting_rgb = probe_lineage_trace_lighting_rgb(
                extract,
                &prepare.scheduled_trace_region_ids,
                probe.probe_id,
            );
            let (
                runtime_hierarchy_irradiance_weight_q,
                runtime_hierarchy_irradiance_rgb,
            ) = runtime_irradiance_source(resolve_runtime, extract, probe.probe_id);
            let (
                runtime_trace_support_q,
                runtime_trace_lighting_rgb,
            ) = runtime_trace_source(resolve_runtime, extract, probe.probe_id);
            let (lineage_trace_support_q, lineage_trace_lighting_rgb) = merge_trace_sources(
                scheduled_trace_support_q,
                scheduled_trace_lighting_rgb,
                runtime_trace_support_q,
                runtime_trace_lighting_rgb,
            );
            GpuResidentProbeInput {
                probe_id: probe.probe_id,
                slot: probe.slot,
                ray_budget: probe.ray_budget,
                lineage_trace_support_q,
                position_x_q: probe_position_x_q(extract, probe.probe_id),
                position_y_q: probe_position_y_q(extract, probe.probe_id),
                position_z_q: probe_position_z_q(extract, probe.probe_id),
                radius_q: probe_radius_q(extract, probe.probe_id),
                previous_irradiance_rgb: pack_rgb8(probe.irradiance_rgb),
                runtime_hierarchy_irradiance_rgb,
                runtime_hierarchy_irradiance_weight_q,
                lineage_trace_lighting_rgb,
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
