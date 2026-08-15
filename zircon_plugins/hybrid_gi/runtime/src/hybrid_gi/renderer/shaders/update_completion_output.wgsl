@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let completed_probe_count = min(params.pending_probe_count, available_probe_completion_budget());
    let completed_trace_count = min(params.trace_region_count, params.tracing_budget);
    let irradiance_count = params.resident_probe_count + completed_probe_count;

    if (index == 0u) {
        completed_probe_updates[0] = completed_probe_count;
        completed_trace_regions[0] = completed_trace_count;
        probe_irradiance_updates[0] = irradiance_count;
        probe_trace_lighting_updates[0] = irradiance_count;
    }

    if (index < params.resident_probe_count) {
        let probe = resident_probe_inputs[index];
        let entry_offset = 1u + index * 2u;
        let traced = traced_contribution_rgb_with_resident_ancestors(
            probe.position_x_q,
            probe.position_y_q,
            probe.position_z_q,
            probe.radius_q,
            probe.ray_budget,
            probe.resident_ancestor_probe_id,
            probe.resident_ancestor_depth,
            probe.resident_secondary_ancestor_probe_id,
            probe.resident_secondary_ancestor_depth,
            probe.resident_tertiary_ancestor_probe_id,
            probe.resident_tertiary_ancestor_depth,
            probe.resident_quaternary_ancestor_probe_id,
            probe.resident_quaternary_ancestor_depth,
            probe.skip_scene_prepare_for_trace_q,
        );
        let continued_traced = apply_lineage_trace_lighting_continuation(
            traced,
            probe.lineage_trace_lighting_rgb,
            probe.lineage_trace_support_q,
            probe.ray_budget,
        );
        let traced_for_irradiance = traced_contribution_rgb_with_resident_ancestors(
            probe.position_x_q,
            probe.position_y_q,
            probe.position_z_q,
            probe.radius_q,
            probe.ray_budget,
            probe.resident_ancestor_probe_id,
            probe.resident_ancestor_depth,
            probe.resident_secondary_ancestor_probe_id,
            probe.resident_secondary_ancestor_depth,
            probe.resident_tertiary_ancestor_probe_id,
            probe.resident_tertiary_ancestor_depth,
            probe.resident_quaternary_ancestor_probe_id,
            probe.resident_quaternary_ancestor_depth,
            probe.skip_scene_prepare_for_irradiance_q,
        );
        let continued_traced_for_irradiance = apply_lineage_trace_lighting_continuation(
            traced_for_irradiance,
            probe.lineage_trace_lighting_rgb,
            probe.lineage_trace_support_q,
            probe.ray_budget,
        );
        let gathered = gathered_resident_rgb(
            probe.probe_id,
            probe.parent_probe_id,
            probe.resident_ancestor_probe_id,
            probe.resident_ancestor_depth,
            probe.resident_secondary_ancestor_probe_id,
            probe.resident_secondary_ancestor_depth,
            probe.resident_tertiary_ancestor_probe_id,
            probe.resident_tertiary_ancestor_depth,
            probe.resident_quaternary_ancestor_probe_id,
            probe.resident_quaternary_ancestor_depth,
            probe.position_x_q,
            probe.position_y_q,
            probe.position_z_q,
            probe.radius_q,
            probe.ray_budget,
        );
        let contribution = combine_traced_and_gathered_with_runtime_hierarchy_fallback(
            continued_traced_for_irradiance,
            gathered,
            probe.runtime_hierarchy_irradiance_rgb,
            probe.runtime_hierarchy_irradiance_weight_q,
            probe.lineage_trace_support_q,
            probe.ray_budget,
        );
        probe_irradiance_updates[entry_offset] = probe.probe_id;
        probe_irradiance_updates[entry_offset + 1u] = select(
            temporal_update_rgb(
                probe.previous_irradiance_rgb,
                contribution,
                temporal_update_weight(probe.ray_budget, params.tracing_budget),
            ),
            probe.previous_irradiance_rgb,
            contribution == 0u,
        );
        probe_trace_lighting_updates[entry_offset] = probe.probe_id;
        probe_trace_lighting_updates[entry_offset + 1u] = continued_traced;
    }

    if (index < completed_probe_count) {
        let probe = pending_probe_updates[index];
        completed_probe_updates[index + 1u] = probe.probe_id;
        let entry_index = params.resident_probe_count + index;
        let entry_offset = 1u + entry_index * 2u;
        let traced = traced_contribution_rgb_with_resident_ancestors(
            probe.position_x_q,
            probe.position_y_q,
            probe.position_z_q,
            probe.radius_q,
            probe.ray_budget,
            probe.resident_ancestor_probe_id,
            probe.resident_ancestor_depth,
            probe.resident_secondary_ancestor_probe_id,
            probe.resident_secondary_ancestor_depth,
            probe.resident_tertiary_ancestor_probe_id,
            probe.resident_tertiary_ancestor_depth,
            probe.resident_quaternary_ancestor_probe_id,
            probe.resident_quaternary_ancestor_depth,
            probe.skip_scene_prepare_for_trace_q,
        );
        let continued_traced = apply_lineage_trace_lighting_continuation(
            traced,
            probe.lineage_trace_lighting_rgb,
            probe.lineage_trace_support_q,
            probe.ray_budget,
        );
        let traced_for_irradiance = traced_contribution_rgb_with_resident_ancestors(
            probe.position_x_q,
            probe.position_y_q,
            probe.position_z_q,
            probe.radius_q,
            probe.ray_budget,
            probe.resident_ancestor_probe_id,
            probe.resident_ancestor_depth,
            probe.resident_secondary_ancestor_probe_id,
            probe.resident_secondary_ancestor_depth,
            probe.resident_tertiary_ancestor_probe_id,
            probe.resident_tertiary_ancestor_depth,
            probe.resident_quaternary_ancestor_probe_id,
            probe.resident_quaternary_ancestor_depth,
            probe.skip_scene_prepare_for_irradiance_q,
        );
        let continued_traced_for_irradiance = apply_lineage_trace_lighting_continuation(
            traced_for_irradiance,
            probe.lineage_trace_lighting_rgb,
            probe.lineage_trace_support_q,
            probe.ray_budget,
        );
        let gathered = gathered_resident_rgb(
            probe.probe_id,
            probe.parent_probe_id,
            probe.resident_ancestor_probe_id,
            probe.resident_ancestor_depth,
            probe.resident_secondary_ancestor_probe_id,
            probe.resident_secondary_ancestor_depth,
            probe.resident_tertiary_ancestor_probe_id,
            probe.resident_tertiary_ancestor_depth,
            probe.resident_quaternary_ancestor_probe_id,
            probe.resident_quaternary_ancestor_depth,
            probe.position_x_q,
            probe.position_y_q,
            probe.position_z_q,
            probe.radius_q,
            probe.ray_budget,
        );
        probe_irradiance_updates[entry_offset] = probe.probe_id;
        probe_irradiance_updates[entry_offset + 1u] =
            combine_traced_and_gathered_with_runtime_hierarchy_fallback(
                continued_traced_for_irradiance,
                gathered,
                probe.runtime_hierarchy_irradiance_rgb,
                probe.runtime_hierarchy_irradiance_weight_q,
                probe.lineage_trace_support_q,
                probe.ray_budget,
            );
        probe_trace_lighting_updates[entry_offset] = probe.probe_id;
        probe_trace_lighting_updates[entry_offset + 1u] = continued_traced;
    }

    if (index < completed_trace_count) {
        completed_trace_regions[index + 1u] = scheduled_trace_regions[index].region_id;
    }
}
