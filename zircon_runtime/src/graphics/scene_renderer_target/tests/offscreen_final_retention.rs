use super::ensure_offscreen_target;
use crate::core::framework::render::RenderViewportHandle;
use crate::core::math::UVec2;
use crate::graphics::{backend::RenderBackend, ViewportRenderOutputTarget};

#[test]
fn offscreen_final_color_reuse_is_scoped_by_display_extent_and_owner() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let device = &backend.device;
    let a = Some(RenderViewportHandle::new(1));
    let b = Some(RenderViewportHandle::new(2));
    let display = UVec2::new(160, 120);
    let left = UVec2::new(80, 120);
    let primary = Some(ViewportRenderOutputTarget::PrimarySurface);
    let mut active = None;
    assert!(ensure_offscreen_target(
        device,
        &mut active,
        display,
        left,
        a,
        primary
    ));
    let target = active.as_ref().unwrap();
    let (a_final, a_view, a_scene, a_depth) = (
        target.final_color.clone(),
        target.final_color_view.clone(),
        target.scene_color.clone(),
        target.depth.clone(),
    );

    assert!(ensure_offscreen_target(
        device,
        &mut active,
        display,
        display,
        a,
        primary
    ));
    let target = active.as_ref().unwrap();
    assert_eq!(target.final_color, a_final);
    assert_eq!(target.final_color_view, a_view);
    assert_ne!(target.scene_color, a_scene);
    assert_ne!(target.depth, a_depth);
    assert_eq!(target.scene_color.width(), display.x);
    assert_eq!(target.retained_frame_texture_count(), 10);

    // An interleaved direct import has no write access to A's scratch final color.
    assert!(ensure_offscreen_target(
        device,
        &mut active,
        display,
        left,
        a,
        None
    ));
    assert_eq!(active.as_ref().unwrap().final_color, a_final);
    assert!(!ensure_offscreen_target(
        device,
        &mut active,
        display,
        left,
        a,
        primary
    ));
    assert_eq!(active.as_ref().unwrap().final_color, a_final);

    // Same display and internal extent but a different viewport: replace only final color.
    let internal = active.as_ref().unwrap().scene_color.clone();
    assert!(!ensure_offscreen_target(
        device,
        &mut active,
        display,
        left,
        b,
        primary
    ));
    let target = active.as_ref().unwrap();
    assert_ne!(target.final_color, a_final);
    assert_eq!(target.scene_color, internal);
    let b_final = target.final_color.clone();

    let resized = UVec2::new(192, 120);
    assert!(ensure_offscreen_target(
        device,
        &mut active,
        resized,
        left,
        b,
        primary
    ));
    let target = active.as_ref().unwrap();
    assert_ne!(target.final_color, b_final);
    assert_eq!(target.final_color.width(), resized.x);
    assert_eq!(target.scene_color.width(), left.x);
}
