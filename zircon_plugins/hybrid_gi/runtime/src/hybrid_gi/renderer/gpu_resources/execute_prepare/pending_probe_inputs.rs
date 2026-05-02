use std::collections::BTreeSet;

use zircon_runtime::core::framework::render::RenderHybridGiExtract;

use crate::hybrid_gi::types::{HybridGiPrepareFrame, HybridGiResolveRuntime};

use super::super::gpu_pending_probe_input::GpuPendingProbeInput;
use super::probe_quantization::{
    probe_lineage_trace_lighting_rgb, probe_lineage_trace_support_q, probe_parent_probe_id,
    probe_position_x_q, probe_position_y_q, probe_position_z_q, probe_radius_q,
    probe_resident_ancestors, scheduled_live_trace_region_ids,
};
use super::runtime_trace_source::{
    merge_trace_sources, runtime_irradiance_source, runtime_trace_source,
};

pub(super) fn pending_probe_inputs(
    prepare: &HybridGiPrepareFrame,
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    extract: Option<&RenderHybridGiExtract>,
    trace_extract: Option<&RenderHybridGiExtract>,
) -> Vec<GpuPendingProbeInput> {
    let scheduled_trace_region_ids = scheduled_live_trace_region_ids(
        resolve_runtime,
        trace_extract,
        &prepare.scheduled_trace_region_ids,
    );
    let current_trace_schedule_is_empty = scheduled_trace_region_ids.is_empty();
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
            ] = probe_resident_ancestors(
                resolve_runtime,
                extract,
                &resident_probe_ids,
                update.probe_id,
            );
            let scheduled_trace_support_q = probe_lineage_trace_support_q(
                resolve_runtime,
                extract,
                &scheduled_trace_region_ids,
                update.probe_id,
            );
            let scheduled_trace_lighting_rgb = probe_lineage_trace_lighting_rgb(
                resolve_runtime,
                extract,
                &scheduled_trace_region_ids,
                update.probe_id,
            );
            let (
                runtime_hierarchy_irradiance_weight_q,
                runtime_hierarchy_irradiance_rgb,
                runtime_hierarchy_irradiance_includes_scene_truth,
            ) = runtime_irradiance_source(resolve_runtime, update.probe_id);
            let (
                runtime_trace_support_q,
                runtime_trace_lighting_rgb,
                runtime_trace_includes_scene_truth,
            ) = runtime_trace_source(resolve_runtime, update.probe_id);
            let (lineage_trace_support_q, lineage_trace_lighting_rgb) = merge_trace_sources(
                scheduled_trace_support_q,
                scheduled_trace_lighting_rgb,
                runtime_trace_support_q,
                runtime_trace_lighting_rgb,
            );
            GpuPendingProbeInput {
                probe_id: update.probe_id,
                logical_index: prepare.resident_probes.len() as u32 + index as u32,
                ray_budget: update.ray_budget,
                lineage_trace_support_q,
                position_x_q: probe_position_x_q(resolve_runtime, extract, update.probe_id),
                position_y_q: probe_position_y_q(resolve_runtime, extract, update.probe_id),
                position_z_q: probe_position_z_q(resolve_runtime, extract, update.probe_id),
                radius_q: probe_radius_q(resolve_runtime, extract, update.probe_id),
                runtime_hierarchy_irradiance_rgb,
                runtime_hierarchy_irradiance_weight_q,
                skip_scene_prepare_for_irradiance_q: u32::from(
                    current_trace_schedule_is_empty
                        && runtime_hierarchy_irradiance_includes_scene_truth,
                ),
                lineage_trace_lighting_rgb,
                skip_scene_prepare_for_trace_q: u32::from(
                    current_trace_schedule_is_empty && runtime_trace_includes_scene_truth,
                ),
                parent_probe_id: probe_parent_probe_id(resolve_runtime, extract, update.probe_id),
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
