use super::{assert_contains_all, read_runtime_src};

#[test]
fn runtime_90_ui_resource_writes_share_the_frame_upload_transaction() {
    let transaction = read_runtime_src("graphics/scene/scene_renderer/ui/resource_upload.rs");
    let record = read_runtime_src("graphics/scene/scene_renderer/ui/render/record.rs");
    let image = read_runtime_src("graphics/scene/scene_renderer/ui/image.rs");
    let atlas_instances =
        read_runtime_src("graphics/scene/scene_renderer/ui/atlas_renderer/instance_buffer.rs");
    let atlas_renderer =
        read_runtime_src("graphics/scene/scene_renderer/ui/atlas_renderer/renderer.rs");
    let text = read_runtime_src("graphics/scene/scene_renderer/ui/text.rs");
    let sdf_material = read_runtime_src("graphics/scene/scene_renderer/ui/sdf_render/material.rs");
    let sdf_vertices =
        read_runtime_src("graphics/scene/scene_renderer/ui/sdf_render/vertex_buffer.rs");
    let direct = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs",
    );
    let compiled = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs",
    );

    let buffer_owners = [
        record.as_str(),
        image.as_str(),
        atlas_instances.as_str(),
        atlas_renderer.as_str(),
        sdf_material.as_str(),
        sdf_vertices.as_str(),
    ];
    assert_eq!(
        buffer_owners
            .iter()
            .map(|source| source.matches("queue.write_buffer(").count())
            .sum::<usize>(),
        0,
        "UI buffer owners must not bypass the frame upload transaction"
    );
    assert_eq!(
        buffer_owners
            .iter()
            .map(|source| source.matches("WgpuBufferUpload::from_bytes(").count())
            .sum::<usize>(),
        6,
        "the six UI dynamic buffer owners must each publish one neutral upload site"
    );
    assert_contains_all(
        "screen-space UI resource upload reservation and retry state",
        &transaction,
        &[
            "compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)",
            "let force_full_upload = self.committed_generation != self.prepared_generation;",
            "if !prepared.attached",
            "Arc::ptr_eq(&self.owner, &prepared.owner)",
            "if !prepared.force_full_upload || prepared.full_upload_prepared",
            "impl Drop for ScreenSpaceUiPreparedUpload",
            "dropped_preparation_forces_the_next_frame_to_upload_in_full",
            "an_empty_retry_frame_does_not_clear_the_forced_full_upload",
            "a_prepared_full_retry_clears_the_forced_full_upload_after_commit",
            "overlapping_preparations_are_rejected_until_drop",
        ],
    );
    assert_contains_all(
        "text atlas owner-scoped retry and commit state",
        &text,
        &[
            "struct ScreenSpaceUiTextAtlasRecoveryState",
            "transaction_force_full_upload || self.bitmap_full_replay_required",
            "transaction_force_full_upload || self.sdf_full_replay_required",
            "note_bitmap_abort(pending.owner_contents_changed)",
            "note_sdf_abort(pending.owner_contents_changed)",
            "commit_bitmap_recovery(pending.recovery_complete)",
            "commit_sdf_recovery(pending.recovery_complete)",
            "bitmap_force_full_upload",
            "sdf_force_full_upload",
            "pending_bitmap_recovery",
            "pending_sdf_atlas_upload",
            "sdf_owner_has_render_contents(&sdf_renderer_report)",
        ],
    );
    assert!(
        !text.contains("sdf_renderer_report.material_count > 0"),
        "the always-present default SDF material must not create empty-frame recovery debt"
    );
    let text_production = text
        .split("#[cfg(test)]")
        .next()
        .expect("screen-space text production source");
    assert!(
        !text_production.contains(".expect("),
        "screen-space text transaction preparation must fail closed without production panics"
    );
    assert_contains_all(
        "inactive UI owners invalidate speculative upload caches",
        &format!("{record}\n{image}\n{atlas_renderer}\n{sdf_vertices}"),
        &[
            "vertex_segment.payload_hash = None",
            "self.payload_hash = None",
            "self.last_viewport_transform = None",
            "*payload_hash = None",
        ],
    );

    let direct_attach = direct
        .find("&mut frame_texture_uploads,")
        .expect("direct UI upload attachment");
    let direct_accept = direct
        .find("backend.enqueue_copy_resource_upload_batch(")
        .expect("direct frame upload acceptance");
    let direct_commit = direct
        .find("renderer.commit_prepared_upload(prepared_upload)")
        .expect("direct UI upload commit");
    let direct_scene_validation = direct
        .find("submission_transaction.validate_scene_submission(scene_submission)")
        .expect("direct scene ticket validation");
    let direct_scene_submit = direct
        .find("backend.submit_graphics_command_buffers_with_frame_diagnostics_and_surface(")
        .expect("direct scene submission");
    let direct_pipeline_usage = direct[direct_scene_submit..]
        .find(".bind_recorded_pipeline_usage_to_submission(scene_submission)")
        .map(|offset| direct_scene_submit + offset)
        .expect("direct submitted pipeline usage settlement");
    let direct_cubemap_commit = direct[direct_scene_submit..]
        .find("self.scene_environment_cubemap.commit_pending_upload()")
        .map(|offset| direct_scene_submit + offset)
        .expect("direct submitted cubemap settlement");
    let direct_realtime_ibl_commit = direct[direct_scene_submit..]
        .find("self.realtime_ibl.complete_submission(submission, true)")
        .map(|offset| direct_scene_submit + offset)
        .expect("direct submitted realtime IBL settlement");
    assert!(
        direct_attach < direct_accept
            && direct_accept < direct_scene_validation
            && direct_scene_validation < direct_commit
    );
    assert!(direct_scene_submit < direct_pipeline_usage);
    assert!(direct_pipeline_usage < direct_scene_validation);
    assert!(direct_cubemap_commit < direct_scene_validation);
    assert!(direct_realtime_ibl_commit < direct_scene_validation);

    let compiled_take = compiled
        .find("graph_execution.take_screen_space_ui_upload_commits()")
        .expect("compiled UI commit handoff");
    let compiled_accept = compiled
        .find(".enqueue_copy_resource_upload_batch(")
        .expect("compiled frame upload acceptance");
    let compiled_commit = compiled
        .find("renderer.commit_prepared_upload(prepared)")
        .expect("compiled UI upload commit");
    let compiled_scene_validation = compiled
        .find("submission_transaction.validate_scene_submission(scene_submission)")
        .expect("compiled scene ticket validation");
    assert!(
        compiled_take < compiled_accept
            && compiled_accept < compiled_scene_validation
            && compiled_scene_validation < compiled_commit
    );
}
