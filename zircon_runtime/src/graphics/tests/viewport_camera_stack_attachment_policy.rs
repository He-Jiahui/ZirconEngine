use super::*;
use crate::core::framework::render::ViewportCameraSnapshot;
use crate::core::math::Vec4;
use crate::render_graph::RenderGraphAttachmentStoreOp;

fn camera(render_type: CameraRenderType, clear: RenderCameraClear) -> CameraRenderDescriptor {
    CameraRenderDescriptor {
        render_type,
        clear,
        ..CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default())
    }
}

#[test]
fn base_camera_clear_modes_translate_to_scene_load_ops() {
    let preview_clear = Vec4::new(0.125, 0.25, 0.5, 1.0);
    let skybox = ViewportCameraStackAttachmentPolicy::from_camera(&camera(
        CameraRenderType::Base,
        RenderCameraClear::Skybox,
    ));
    assert_eq!(
        skybox.scene_clear_plan(),
        ViewportSceneClearPlan::new(Some(ViewportSceneColorClear::Preview), true)
    );
    assert_eq!(
        skybox
            .scene_clear_plan()
            .scene_color()
            .unwrap()
            .resolve(preview_clear),
        preview_clear
    );
    assert_eq!(
        skybox.scene_color_ops(),
        RenderGraphAttachmentOps::load_store()
    );
    assert_eq!(
        skybox.scene_depth_ops(),
        RenderGraphAttachmentOps::load_store()
    );

    let color = ViewportCameraStackAttachmentPolicy::from_camera(&camera(
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::ONE),
    ));
    assert_eq!(
        color.scene_clear_plan(),
        ViewportSceneClearPlan::new(Some(ViewportSceneColorClear::Color(Vec4::ONE)), true)
    );
    assert_eq!(
        color.scene_color_ops(),
        RenderGraphAttachmentOps::load_store()
    );
    assert_eq!(
        color.scene_depth_ops(),
        RenderGraphAttachmentOps::load_store()
    );

    let depth_only = ViewportCameraStackAttachmentPolicy::from_camera(&camera(
        CameraRenderType::Base,
        RenderCameraClear::DepthOnly,
    ));
    assert_eq!(
        depth_only.scene_clear_plan(),
        ViewportSceneClearPlan::new(None, true)
    );
    assert_eq!(
        depth_only.scene_color_ops(),
        RenderGraphAttachmentOps::load_store()
    );
    assert_eq!(
        depth_only.scene_depth_ops(),
        RenderGraphAttachmentOps::load_store()
    );

    let no_clear = ViewportCameraStackAttachmentPolicy::from_camera(&camera(
        CameraRenderType::Base,
        RenderCameraClear::None,
    ));
    assert_eq!(
        no_clear.scene_clear_plan(),
        ViewportSceneClearPlan::default()
    );
    assert_eq!(
        no_clear.scene_color_ops(),
        RenderGraphAttachmentOps::load_store()
    );
    assert_eq!(
        no_clear.scene_depth_ops(),
        RenderGraphAttachmentOps::load_store()
    );
}

#[test]
fn base_camera_none_clear_with_msaa_clears_scene_color_only() {
    let mut descriptor = camera(CameraRenderType::Base, RenderCameraClear::None);
    descriptor.camera.msaa_samples = 4;

    let policy = ViewportCameraStackAttachmentPolicy::from_camera(&descriptor);

    assert_eq!(
        policy.scene_clear_plan(),
        ViewportSceneClearPlan::new(Some(ViewportSceneColorClear::Transparent), false)
    );
    assert_eq!(
        policy.scene_color_ops(),
        RenderGraphAttachmentOps::load_store()
    );
    assert_eq!(
        policy.scene_depth_ops(),
        RenderGraphAttachmentOps::load_store()
    );
}

#[test]
fn overlay_camera_never_clears_scene_color_and_uses_clear_depth_for_depth() {
    let mut descriptor = camera(
        CameraRenderType::Overlay,
        RenderCameraClear::Color(Vec4::ONE),
    );
    descriptor.clear_depth = true;

    let clear_depth = ViewportCameraStackAttachmentPolicy::from_camera(&descriptor);
    assert_eq!(
        clear_depth.scene_clear_plan(),
        ViewportSceneClearPlan::new(None, true)
    );
    assert_eq!(
        clear_depth.scene_color_ops(),
        RenderGraphAttachmentOps::load_store()
    );
    assert_eq!(
        clear_depth.scene_depth_ops(),
        RenderGraphAttachmentOps::load_store()
    );

    descriptor.clear_depth = false;
    let load_depth = ViewportCameraStackAttachmentPolicy::from_camera(&descriptor);
    assert_eq!(
        load_depth.scene_clear_plan(),
        ViewportSceneClearPlan::default()
    );
    assert_eq!(
        load_depth.scene_color_ops(),
        RenderGraphAttachmentOps::load_store()
    );
    assert_eq!(
        load_depth.scene_depth_ops(),
        RenderGraphAttachmentOps::load_store()
    );
}

#[test]
fn policy_only_rewrites_scene_first_clear_writes_and_preserves_store() {
    let policy = ViewportCameraStackAttachmentPolicy::from_camera(&camera(
        CameraRenderType::Overlay,
        RenderCameraClear::Skybox,
    ));
    let clear_discard = RenderGraphAttachmentOps::clear_discard();

    assert_eq!(
        policy.apply_to_first_attachment_write(
            PostProcessGraphResourceNames::SCENE_COLOR,
            clear_discard,
        ),
        RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Discard,
        }
    );
    assert_eq!(
        policy.apply_to_first_attachment_write(
            PostProcessGraphResourceNames::SCENE_DEPTH,
            clear_discard,
        ),
        RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Discard,
        }
    );
    assert_eq!(
        policy.apply_to_first_attachment_write(
            PostProcessGraphResourceNames::SCENE_COLOR,
            RenderGraphAttachmentOps::load_store(),
        ),
        RenderGraphAttachmentOps::load_store()
    );
    assert_eq!(
        policy.apply_to_first_attachment_write("taa-output", clear_discard),
        clear_discard
    );
}
