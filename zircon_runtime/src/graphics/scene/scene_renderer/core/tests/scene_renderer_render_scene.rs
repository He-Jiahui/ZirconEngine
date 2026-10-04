#[test]
fn renderer_admission_uses_world_registry_and_authoritative_streamer_bridge() {
    let source = include_str!("../scene_renderer_render_scene.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("render-scene admission must retain a test boundary");

    assert!(production.contains("render_scene_registry.projector_for_frame(frame)"));
    assert!(production.contains(".admit_render_scene_frame("));
    assert!(production.contains("RenderSceneAdmission(error.to_string())"));
    assert!(!production.contains("frame.meshes()"));
}

#[test]
fn renderer_gpu_scene_journal_is_staged_at_admission_and_committed_after_submission() {
    let admission = include_str!("../scene_renderer_render_scene.rs");
    let direct = include_str!("../scene_renderer_render/render_frame.rs");
    let compiled = include_str!(
        "../scene_renderer_render_with_pipeline/render_frame_with_pipeline/frame_submission_owner.rs"
    );

    assert!(admission.contains("projected_consumer.stage_owned(journal"));
    assert!(admission.contains("pending_gpu_scene_journals.push_back("));
    assert!(admission.contains("fn stage_pending_gpu_scene_membership("));
    assert!(admission.contains(".apply_journal_membership(&self.backend.device, plan)"));
    assert!(admission.contains("fn commit_pending_gpu_scene_journals("));
    assert!(admission.contains("std::mem::take(&mut self.pending_gpu_scene_journals)"));

    for owner in [direct, compiled] {
        let validate = owner
            .find("submission_transaction.validate_scene_submission(scene_submission)")
            .expect("frame owner must validate the accepted scene submission");
        let commit = owner
            .find("self.commit_pending_gpu_scene_journals()")
            .expect("frame owner must publish its staged GPUScene journal");
        let finish = owner
            .find("submission_transaction.finish(scene_submission)")
            .expect("frame owner must finish its submission receipt");
        assert!(validate < commit);
        assert!(commit < finish);
    }
}

#[test]
fn renderer_world_release_delegates_to_the_same_registry_and_streamer_owners() {
    let source = include_str!("../scene_renderer_render_scene.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("render-scene admission must retain a test boundary");

    assert!(production.contains("fn release_render_scene_world("));
    assert!(production.contains("&mut self.render_scene_registry"));
    assert!(production.contains("&self.backend"));
    assert!(production.contains("consumer.resident_stable_keys()"));
    assert!(production.contains("self.core.gpu_scene.unregister(stable_instance_key)"));
}
