use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    CapturedFrame, RenderCameraClear, RenderCameraTargetKind, RenderCaptureReport, RenderFramework,
    RenderFrameworkError, RenderViewportDescriptor, RenderViewportHandle,
};
use crate::core::math::{UVec2, Vec4};
use crate::graphics::WgpuRenderFramework;

use super::empty_extract;

const SMALL: UVec2 = UVec2::new(40, 24);
const LARGE: UVec2 = UVec2::new(64, 48);
const RED: usize = 0;
const GREEN: usize = 1;

type CaptureResult = Result<Option<CapturedFrame>, RenderFrameworkError>;

#[test]
fn graphics_surface_ldr_capture_does_not_cross_viewport_owner() {
    verify_fresh_a(LARGE, |framework, a| framework.capture_frame(a));
}

#[test]
fn graphics_surface_ldr_capture_if_newer_none_respects_equal_extent_owners() {
    verify_fresh_a(SMALL, |framework, a| {
        framework.capture_frame_if_newer(a, None)
    });
}

#[test]
fn graphics_surface_ldr_capture_if_newer_some_does_not_cross_viewport_owner() {
    verify_fresh_a(LARGE, |framework, a| {
        framework.capture_frame_if_newer(a, Some(0))
    });
}

#[test]
fn graphics_surface_ldr_capture_preserves_a_cache_after_rendering_b() {
    verify_warm_a(false);
}

#[test]
fn graphics_surface_ldr_capture_if_newer_rejects_equal_generation_without_poisoning_a() {
    verify_warm_a(true);
}

#[test]
fn graphics_surface_ldr_capture_distinguishes_unknown_viewport_and_empty_cache() {
    let (wgpu, a, b) = viewport_pair(LARGE);
    let framework: &dyn RenderFramework = &wgpu;
    let b_frame = submit_and_capture(framework, b, SMALL, GREEN, "B");
    assert_a_poll_empty(framework, a);

    let unknown = RenderViewportHandle::new(u64::MAX);
    assert!(unknown != a && unknown != b);
    let expected = RenderFrameworkError::UnknownViewport {
        viewport: unknown.raw(),
    };
    for result in [
        framework.capture_frame(unknown),
        framework.capture_frame_if_newer(unknown, None),
        framework.capture_frame_if_newer(unknown, Some(0)),
        framework.poll_captured_frame_if_newer(unknown, None),
    ] {
        assert_eq!(result.err().as_ref(), Some(&expected));
    }
    assert_a_poll_empty(framework, a);
    let b_poll = framework
        .poll_captured_frame_if_newer(b, Some(0))
        .unwrap()
        .expect("B's cache must remain readable");
    assert!(same_product(&b_poll, &b_frame));
}

fn viewport_pair(
    a_size: UVec2,
) -> (
    WgpuRenderFramework,
    RenderViewportHandle,
    RenderViewportHandle,
) {
    let framework = WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default()))
        .expect("headless WGPU adapter and renderer are required");
    let a = framework
        .create_viewport(RenderViewportDescriptor::new(a_size))
        .unwrap();
    let b = framework
        .create_viewport(RenderViewportDescriptor::new(SMALL))
        .unwrap();
    assert_ne!(a, b);
    (framework, a, b)
}

fn submit_and_capture(
    framework: &dyn RenderFramework,
    viewport: RenderViewportHandle,
    size: UVec2,
    dominant: usize,
    label: &str,
) -> CapturedFrame {
    let mut extract = empty_extract();
    let mut rgb = [0.0; 3];
    rgb[dominant] = 1.0;
    extract
        .view
        .selected_camera_descriptor_mut()
        .expect("ordinary extract must carry a selected camera")
        .clear = RenderCameraClear::Color(Vec4::new(rgb[0], rgb[1], rgb[2], 1.0));
    framework
        .submit_frame_extract(viewport, extract)
        .expect("offscreen submission must succeed");
    let frame = framework
        .capture_frame(viewport)
        .expect("LDR reference readback must succeed")
        .expect("the just-submitted viewport must yield an LDR reference");
    assert_eq!((frame.width, frame.height), (size.x, size.y));
    assert_eq!(frame.rgba.len(), (size.x * size.y * 4) as usize);
    assert_eq!(
        frame.capture_report,
        RenderCaptureReport::framework_offscreen(RenderCameraTargetKind::PrimarySurface, size)
    );
    assert_ne!(frame.generation, 0);
    assert_eq!(
        Some(frame.generation),
        framework.query_stats().unwrap().last_generation
    );

    // Use channel dominance, not exact display encoding, to establish each real reference.
    let pixel = center_pixel(&frame).expect("LDR reference must have a center pixel");
    assert!(
        pixel[dominant] > 80
            && pixel[..3]
                .iter()
                .enumerate()
                .all(|(i, other)| i == dominant
                    || u16::from(pixel[dominant]) > u16::from(*other) + 40),
        "reference {label} must be visibly channel {dominant}, got {pixel:?}"
    );
    eprintln!(
        "LDR reference {label}: handle={} {}x{} generation={} center={pixel:?}",
        viewport.raw(),
        frame.width,
        frame.height,
        frame.generation
    );
    frame
}

fn assert_a_poll_empty(framework: &dyn RenderFramework, a: RenderViewportHandle) {
    for last_generation in [None, Some(0)] {
        assert!(
            framework
                .poll_captured_frame_if_newer(a, last_generation)
                .unwrap()
                .is_none(),
            "uncached A must not see any completed capture before a synchronous request"
        );
    }
}

fn verify_fresh_a(
    a_size: UVec2,
    request_a: impl FnOnce(&dyn RenderFramework, RenderViewportHandle) -> CaptureResult,
) {
    let (wgpu, a, b) = viewport_pair(a_size);
    let framework: &dyn RenderFramework = &wgpu;
    let b_frame = submit_and_capture(framework, b, SMALL, GREEN, "B");
    assert_a_poll_empty(framework, a);
    let before = framework.query_stats().unwrap();
    assert_eq!(before.submitted_frames, 1, "only B was submitted");

    // Collect sync and both polls before asserting so wrong bytes and cache pollution are separable.
    let direct = request_a(framework, a).unwrap();
    let cached = framework.poll_captured_frame_if_newer(a, None).unwrap();
    let newer = framework.poll_captured_frame_if_newer(a, Some(0)).unwrap();
    let after = framework.query_stats().unwrap();
    assert!(
        direct.is_none() && cached.is_none() && newer.is_none()
            && after.captured_frames == before.captured_frames,
        "never-submitted A must not see B; (generation, extent, center, equals_B): sync={:?}, poll(None)={:?}, poll(Some(0))={:?}; captured_frames {} -> {}",
        signature(direct.as_ref(), &b_frame),
        signature(cached.as_ref(), &b_frame),
        signature(newer.as_ref(), &b_frame),
        before.captured_frames,
        after.captured_frames,
    );
}

fn verify_warm_a(reject_b_generation: bool) {
    let (wgpu, a, b) = viewport_pair(SMALL);
    let framework: &dyn RenderFramework = &wgpu;
    let a_frame = submit_and_capture(framework, a, SMALL, RED, "A");
    let b_frame = submit_and_capture(framework, b, SMALL, GREEN, "B");
    assert!(b_frame.generation > a_frame.generation);
    assert_ne!(a_frame.rgba, b_frame.rgba);

    let prior = framework.poll_captured_frame_if_newer(a, None).unwrap();
    let eligible = framework.capture_frame_if_newer(a, Some(0)).unwrap();
    for (operation, result) in [("poll(None)", prior), ("sync(Some(0))", eligible)] {
        assert!(
            result
                .as_ref()
                .is_some_and(|frame| same_product(frame, &a_frame)),
            "{operation} must still see A's eligible cache before the fallback; A={:?}, got={:?}",
            signature(Some(&a_frame), &a_frame),
            signature(result.as_ref(), &a_frame)
        );
    }

    let direct = if reject_b_generation {
        // Returning None is not sufficient: generation rejection must precede any foreign cache write.
        framework
            .capture_frame_if_newer(a, Some(b_frame.generation))
            .unwrap()
    } else {
        framework.capture_frame(a).unwrap()
    };
    let cached = framework.poll_captured_frame_if_newer(a, None).unwrap();
    let newer = framework
        .poll_captured_frame_if_newer(a, Some(a_frame.generation))
        .unwrap();
    // No requirement to retain A's old GPU target: unfiltered sync may return its own cached A or None.
    let allowed = direct
        .as_ref()
        .is_none_or(|frame| !reject_b_generation && same_product(frame, &a_frame));
    assert!(
        allowed && cached.as_ref().is_some_and(|frame| same_product(frame, &a_frame)) && newer.is_none(),
        "after B, A must not lose or overwrite its completed cache; (generation, extent, center, equals_B): sync={:?}, poll(None)={:?}, poll(Some(A.generation))={:?}",
        signature(direct.as_ref(), &b_frame),
        signature(cached.as_ref(), &b_frame),
        signature(newer.as_ref(), &b_frame),
    );
}

fn same_product(frame: &CapturedFrame, reference: &CapturedFrame) -> bool {
    (frame.width, frame.height) == (reference.width, reference.height)
        && frame.generation == reference.generation
        && frame.capture_report == reference.capture_report
        && frame.rgba == reference.rgba
}

fn signature(
    frame: Option<&CapturedFrame>,
    reference: &CapturedFrame,
) -> Option<(u64, (u32, u32), Option<[u8; 4]>, bool)> {
    frame.map(|frame| {
        (
            frame.generation,
            (frame.width, frame.height),
            center_pixel(frame),
            same_product(frame, reference),
        )
    })
}

fn center_pixel(frame: &CapturedFrame) -> Option<[u8; 4]> {
    let pixel = (frame.height as usize / 2) * frame.width as usize + frame.width as usize / 2;
    frame.rgba.chunks_exact(4).nth(pixel)?.try_into().ok()
}
