@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let entry_count = params.resident_probe_count + params.completed_probe_count;
    if (index == 0u) {
        probe_trace_lighting_updates[0] = entry_count;
        probe_trace_diagnostics[0] = entry_count;
    }
    if (index >= entry_count || params.tile_count == 0u) {
        return;
    }

    var probe_id = 0u;
    var ray_budget = 0u;
    var position_x_q = 0u;
    var position_y_q = 0u;
    var position_z_q = 0u;
    var lineage_trace_lighting_rgb = 0u;
    if (index < params.resident_probe_count) {
        let probe = resident_probe_inputs[index];
        probe_id = probe.probe_id;
        ray_budget = probe.ray_budget;
        position_x_q = probe.position_x_q;
        position_y_q = probe.position_y_q;
        position_z_q = probe.position_z_q;
        lineage_trace_lighting_rgb = probe.lineage_trace_lighting_rgb;
    } else {
        let probe = pending_probe_updates[index - params.resident_probe_count];
        probe_id = probe.probe_id;
        ray_budget = probe.ray_budget;
        position_x_q = probe.position_x_q;
        position_y_q = probe.position_y_q;
        position_z_q = probe.position_z_q;
        lineage_trace_lighting_rgb = probe.lineage_trace_lighting_rgb;
    }

    let trace_result = tile_trace(
        probe_id,
        ray_budget,
        position_x_q,
        position_y_q,
        position_z_q,
        lineage_trace_lighting_rgb,
    );
    let entry_offset = 1u + index * 2u;
    probe_trace_lighting_updates[entry_offset] = probe_id;
    probe_trace_lighting_updates[entry_offset + 1u] = trace_result.rgb;

    let diagnostic_offset = 1u + index * TRACE_DIAGNOSTIC_WORDS_PER_ENTRY;
    probe_trace_diagnostics[diagnostic_offset] = probe_id;
    probe_trace_diagnostics[diagnostic_offset + 1u] = trace_result.intersection_source;
    probe_trace_diagnostics[diagnostic_offset + 2u] = trace_result.lighting_source;
    probe_trace_diagnostics[diagnostic_offset + 3u] = trace_result.intersection_backend_mask;
    probe_trace_diagnostics[diagnostic_offset + 4u] = trace_result.lighting_source_mask;
    probe_trace_diagnostics[diagnostic_offset + 5u] = bitcast<u32>(trace_result.distance);
    probe_trace_diagnostics[diagnostic_offset + 6u] = bitcast<u32>(trace_result.confidence);
    probe_trace_diagnostics[diagnostic_offset + 7u] = trace_result.fallback_reason;
    probe_trace_diagnostics[diagnostic_offset + 8u] = trace_result.texture_samples;
    probe_trace_diagnostics[diagnostic_offset + 9u] = trace_result.page_tests;
    probe_trace_diagnostics[diagnostic_offset + 10u] = trace_result.sdf_steps;
    probe_trace_diagnostics[diagnostic_offset + 11u] = trace_result.voxel_candidates;
    probe_trace_diagnostics[diagnostic_offset + 12u] = 0u;
}
