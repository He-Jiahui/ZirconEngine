use crate::core::framework::render::{
    RenderCameraTargetKind, RenderFrameExtract, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
use crate::graphics::types::ViewportRenderFrame;
use crate::scene::World;

use super::output_target_texture_id;

#[test]
fn output_target_texture_id_uses_resolved_texture_target_only() {
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/output-target/texture",
    ));
    let frame =
        empty_frame().with_output_target(crate::graphics::ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(64, 64),
            format: crate::graphics::types::FRAMEWORK_OUTPUT_FORMAT_LABEL,
        });

    assert_eq!(output_target_texture_id(&frame), Some(texture.id()));
}

#[test]
fn output_target_texture_id_ignores_non_texture_targets() {
    let headless =
        empty_frame().with_output_target(crate::graphics::ViewportRenderOutputTarget::Headless {
            size: UVec2::new(64, 64),
        });
    let primary = empty_frame();

    assert_eq!(
        headless.output_target().kind(),
        RenderCameraTargetKind::Headless
    );
    assert_eq!(output_target_texture_id(&headless), None);
    assert_eq!(
        primary.output_target().kind(),
        RenderCameraTargetKind::PrimarySurface
    );
    assert_eq!(output_target_texture_id(&primary), None);
}

fn empty_frame() -> ViewportRenderFrame {
    ViewportRenderFrame::from_extract(
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        ),
        UVec2::new(64, 64),
    )
}
