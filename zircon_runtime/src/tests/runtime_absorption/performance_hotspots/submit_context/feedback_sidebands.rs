use super::sources::SubmitContextSources;

pub(super) fn assert_feedback_sidebands_move_owned_payloads(sources: &SubmitContextSources) {
    let camera_loop = sources.camera_loop;
    let collect_feedback = sources.collect_feedback;
    let build_runtime_frame = sources.build_runtime_frame;
    let submit_runtime_frame = sources.submit_runtime_frame;
    let record_submission = sources.record_submission;
    let record_present_submission = sources.record_present_submission;
    let record_camera_history = sources.record_camera_history;
    let prepared_submission = sources.prepared_submission;
    let prepared_runtime_sidebands = sources.prepared_runtime_sidebands;
    let viewport_render_frame = sources.viewport_render_frame;

    for required_feedback_anchor in [
        "sidebands: &mut RenderPreparedRuntimeSidebands",
        "sidebands.take_hybrid_gi_readback_outputs()",
        "sidebands.take_particle_readback_outputs()",
        "sidebands.take_virtual_geometry_readback_outputs()",
        "sideband_outputs: RenderHybridGiReadbackOutputs",
        "sideband_outputs: RenderParticleGpuReadbackOutputs",
        "sideband_outputs: RenderVirtualGeometryReadbackOutputs",
        "renderer_outputs.cache_entries.extend(cache_entries);",
        "renderer_outputs.page_replacements.extend(page_replacements);",
    ] {
        assert!(
            collect_feedback.contains(required_feedback_anchor),
            "Runtime 07 F3 feedback sideband merge should retain owned-merge anchor `{required_feedback_anchor}`"
        );
    }
    for required_prepared_sideband_anchor in [
        "fn take_hybrid_gi_readback_outputs(",
        "fn take_particle_readback_outputs(",
        "fn take_virtual_geometry_readback_outputs(",
    ] {
        assert!(
            prepared_runtime_sidebands.contains(required_prepared_sideband_anchor),
            "RenderPreparedRuntimeSidebands should retain sideband take anchor `{required_prepared_sideband_anchor}`"
        );
    }
    for forbidden_feedback_clone in [
        "return sideband_outputs.clone();",
        "sideband_outputs: &RenderHybridGiReadbackOutputs",
        "sideband_outputs: &RenderParticleGpuReadbackOutputs",
        "sideband_outputs: &RenderVirtualGeometryReadbackOutputs",
        "sideband_outputs.cache_entries.iter().cloned()",
        "sideband_outputs.completed_page_assignments.iter().cloned()",
        "sideband_outputs.scene_prepare.clone()",
    ] {
        assert!(
            !collect_feedback.contains(forbidden_feedback_clone),
            "Runtime 07 F3 feedback sideband merge should not restore borrowed clone `{forbidden_feedback_clone}`"
        );
    }
    for required_prepared_sideband_move_anchor in [
        "fn into_prepared_runtime_sidebands(self) -> RenderPreparedRuntimeSidebands",
        "prepared.into_prepared_runtime_sidebands()",
        "sidebands: &mut RenderPreparedRuntimeSidebands",
        "sidebands.take_hybrid_gi_readback_outputs()",
        "sidebands.take_particle_readback_outputs()",
        "sidebands.take_virtual_geometry_readback_outputs()",
        "with_evictable_probe_ids(sidebands.take_hybrid_gi_evictable_probe_ids())",
        "with_evictable_page_ids(sidebands.take_virtual_geometry_evictable_page_ids())",
        "pub(crate) fn prepared_runtime_sidebands_mut(&mut self) -> &mut RenderPreparedRuntimeSidebands",
    ] {
        assert!(
            prepared_submission.contains(required_prepared_sideband_move_anchor)
                || prepared_runtime_sidebands.contains(required_prepared_sideband_move_anchor)
                || collect_feedback.contains(required_prepared_sideband_move_anchor)
                || build_runtime_frame.contains(required_prepared_sideband_move_anchor)
                || submit_runtime_frame.contains(required_prepared_sideband_move_anchor)
                || viewport_render_frame.contains(required_prepared_sideband_move_anchor),
            "Runtime 07 F3 prepared sideband frame-owner move should retain anchor `{required_prepared_sideband_move_anchor}`"
        );
    }
    for forbidden_prepared_sideband_clone in [
        "plugin_renderer_outputs.clone()",
        "hybrid_gi_evictable_probe_ids.clone()",
        "virtual_geometry_evictable_page_ids.clone()",
        "prepared.prepared_runtime_sidebands()",
        "mut prepared: PreparedRuntimeSubmission",
        "prepared.take_hybrid_gi_evictable_probe_ids()",
        "prepared.take_virtual_geometry_evictable_page_ids()",
        "with_prepared_runtime_sidebands(frame.prepared_runtime_sidebands.clone())",
        "frame.prepared_runtime_sidebands.clone()",
    ] {
        assert!(
            !prepared_submission.contains(forbidden_prepared_sideband_clone)
                && !camera_loop.contains(forbidden_prepared_sideband_clone)
                && !build_runtime_frame.contains(forbidden_prepared_sideband_clone)
                && !submit_runtime_frame.contains(forbidden_prepared_sideband_clone)
                && !record_submission.contains(forbidden_prepared_sideband_clone)
                && !record_present_submission.contains(forbidden_prepared_sideband_clone)
                && !record_camera_history.contains(forbidden_prepared_sideband_clone),
            "Runtime 07 F3 prepared sideband frame-owner move should not restore `{forbidden_prepared_sideband_clone}`"
        );
    }
}
