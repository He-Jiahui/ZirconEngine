use crate::core::math::UVec2;
use crate::graphics::backend::RenderBackend;
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use super::{SceneFrameHistoryRequirements, SceneFrameHistoryTextures};

#[test]
fn exposure_history_reset_commits_only_after_prepared_upload_acceptance() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let requirements = SceneFrameHistoryRequirements::new(false, false, false, false, true, None);
    let (mut history, _) = SceneFrameHistoryTextures::new_with_requirements_and_initialization(
        &backend.device,
        UVec2::splat(8),
        UVec2::splat(8),
        requirements,
    );
    assert!(!history.exposure_history_reset_pending());

    history.request_exposure_history_reset();
    let mut uploads = WgpuBufferUploadBatch::new();
    assert!(history.prepare_exposure_history_reset(&mut uploads));
    assert!(!uploads.is_empty());
    assert!(history.exposure_history_reset_pending());

    assert!(history.commit_exposure_history_reset());
    assert!(!history.exposure_history_reset_pending());
}

#[test]
fn exposure_only_requirements_create_no_image_history_or_clear_commands() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let requirements = SceneFrameHistoryRequirements::new(false, false, false, false, true, None);
    let (history, initialization_command_buffer) =
        SceneFrameHistoryTextures::new_with_requirements_and_initialization(
            &backend.device,
            UVec2::new(32, 16),
            UVec2::new(16, 8),
            requirements,
        );

    assert!(initialization_command_buffer.is_none());
    assert!(history.exposure_previous_buffer().is_some());
    assert!(history.taa_scene_color_previous_texture().is_none());
    assert!(history.global_illumination_texture().is_none());
    assert!(history.screen_space_reflection_texture().is_none());
    assert!(history.hzb_furthest_texture().is_none());
    assert!(history.volumetric_history_texture().is_none());
}

#[test]
fn enabling_ssr_preserves_unrelated_taa_and_hzb_allocations() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let initial = SceneFrameHistoryRequirements::new(true, false, false, true, false, None);
    let (mut history, _) = SceneFrameHistoryTextures::new_with_requirements_and_initialization(
        &backend.device,
        UVec2::new(32, 16),
        UVec2::new(16, 8),
        initial,
    );
    let taa_identity = history.taa_scene_color_current_identity().unwrap();
    let hzb_identity = history.hzb_resource_identity().unwrap();
    let with_ssr = SceneFrameHistoryRequirements::new(true, false, true, true, false, None);

    let (changes, initialization_command_buffer) = history
        .reconcile_with_requirements_and_initialization(
            &backend.device,
            UVec2::new(32, 16),
            UVec2::new(16, 8),
            with_ssr,
        );

    assert!(changes.changed(super::SceneHistoryDomain::ScreenSpaceReflection));
    assert!(!changes.changed(super::SceneHistoryDomain::TaaSceneColor));
    assert!(!changes.changed(super::SceneHistoryDomain::HzbFurthest));
    assert!(initialization_command_buffer.is_some());
    assert_eq!(
        history.taa_scene_color_current_identity(),
        Some(taa_identity)
    );
    assert_eq!(history.hzb_resource_identity(), Some(hzb_identity));
    assert!(history.screen_space_reflection_texture().is_some());
}

#[test]
fn disabling_image_history_releases_only_that_domain_without_clear_commands() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let initial = SceneFrameHistoryRequirements::new(false, false, true, false, true, None);
    let (mut history, _) = SceneFrameHistoryTextures::new_with_requirements_and_initialization(
        &backend.device,
        UVec2::new(32, 16),
        UVec2::new(16, 8),
        initial,
    );
    let exposure_size = history.exposure_previous_buffer().unwrap().size();
    let exposure_only = SceneFrameHistoryRequirements::new(false, false, false, false, true, None);

    let (changes, initialization_command_buffer) = history
        .reconcile_with_requirements_and_initialization(
            &backend.device,
            UVec2::new(32, 16),
            UVec2::new(16, 8),
            exposure_only,
        );

    assert!(changes.changed(super::SceneHistoryDomain::ScreenSpaceReflection));
    assert!(!changes.changed(super::SceneHistoryDomain::Exposure));
    assert!(initialization_command_buffer.is_none());
    assert!(history.screen_space_reflection_texture().is_none());
    assert_eq!(
        history.exposure_previous_buffer().unwrap().size(),
        exposure_size
    );
}

#[test]
fn shared_scene_history_does_not_allocate_unqualified_ambient_occlusion_storage() {
    let owner = include_str!("../scene_frame_history_textures.rs");
    let owner = owner.split("#[cfg(test)]").next().unwrap();
    let construct = include_str!("../construct.rs");
    let construct = construct.split("#[cfg(test)]").next().unwrap();

    assert!(!owner.contains("ambient_occlusion: wgpu::Texture"));
    assert!(!owner.contains("ambient_occlusion_texture("));
    assert!(!owner.contains("ambient_occlusion_view("));
    assert!(!owner.contains("ambient_occlusion_desc("));
    assert!(!construct.contains("zircon-history-ambient-occlusion"));
}

#[test]
fn exposure_history_reset_uses_frame_upload_transaction_without_raw_queue() {
    let source = include_str!("../scene_frame_history_textures.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();
    let render =
        include_str!("../../../core/scene_renderer_core_render_compiled_scene/render/render.rs");
    let submit = include_str!(
        "../../../core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs"
    );

    assert!(production.contains("prepare_exposure_history_reset"));
    assert!(!production.contains("queue.write_buffer("));
    assert!(!production.contains("queue: &wgpu::Queue"));
    let accept = render
        .find(".enqueue_copy_resource_upload_batch(")
        .expect("frame owner must accept the merged upload batch");
    let ledger = render[accept..]
        .find("RenderFrameSubmissionProducer::FrameResourceUpload")
        .map(|offset| accept + offset)
        .expect("frame upload ticket must enter the ledger");
    // BUG: [CR-W12-RENDER-AUX-A-0001] 执行到此查找时必取 None 并在 expect 中失败：
    // 旧提交方法在当前 submit 文件中匹配数为零；生产调用已改为携带 graph receipt 的提交入口。
    let scene_submit = submit
        .find(".submit_graphics_command_buffers_with_frame_diagnostics_and_surface(")
        .expect("compiled scene must reach its ticketed submit boundary");
    let commit = submit
        .find("commit_exposure_history_reset")
        .expect("accepted exposure reset intent must commit after scene submission");
    assert!(accept < ledger);
    assert!(scene_submit < commit);
}
