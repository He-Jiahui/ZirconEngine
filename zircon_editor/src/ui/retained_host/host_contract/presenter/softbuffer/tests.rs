use std::cell::Cell;

use super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::diagnostics::{
    HostInvalidationDiagnostics, HostRefreshDiagnostics, STARTUP_REFRESH_DIAGNOSTICS_OVERLAY,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::backbuffer::{
    acquire_native_resize_snapshot, capture_native_resize_snapshot, NativeResizeSnapshotAcquisition,
};
use super::diagnostics::{damage_with_debug_overlay, plan_present_for_diagnostics};
use super::surface_io::{
    copy_rgba_to_softbuffer, copy_scaled_rgba_to_softbuffer, damage_pixel_count,
    softbuffer_damage_rect,
};

fn region_damage() -> FrameRect {
    FrameRect {
        x: 10.0,
        y: 80.0,
        width: 20.0,
        height: 10.0,
    }
}

fn top_bar_probe_damage() -> FrameRect {
    FrameRect {
        x: 10.0,
        y: 40.0,
        width: 20.0,
        height: 2.0,
    }
}

fn presentation_with_top_bar_height(height: f32) -> HostWindowPresentationData {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_layout.center_band_frame = FrameRect {
        x: 0.0,
        y: height,
        width: 200.0,
        height: 120.0 - height,
    };
    presentation
}

#[test]
fn region_copy_updates_only_damaged_softbuffer_pixels() {
    let mut frame = HostRgbaFrame::filled(4, 3, [0, 0, 0, 255]);
    let damage = FrameRect {
        x: 1.0,
        y: 1.0,
        width: 2.0,
        height: 1.0,
    };
    frame.fill_rect(&damage, [255, 32, 8, 255]);
    let mut buffer = vec![0x00ff00; 12];

    copy_rgba_to_softbuffer(&frame, &mut buffer, Some(&damage), (4, 3));

    assert_eq!(buffer[5], 0xff2008);
    assert_eq!(buffer[6], 0xff2008);
    for (index, pixel) in buffer.iter().enumerate() {
        if index != 5 && index != 6 {
            assert_eq!(*pixel, 0x00ff00, "pixel {index} should remain untouched");
        }
    }
}

#[test]
fn native_resize_snapshot_keeps_the_first_presented_backbuffer() {
    let mut snapshot = None;
    let mut backbuffer = Some(HostRgbaFrame::filled(2, 1, [255, 0, 0, 255]));

    assert!(capture_native_resize_snapshot(
        &mut snapshot,
        &mut backbuffer
    ));
    assert!(backbuffer.is_none());
    assert_eq!(
        snapshot.as_ref().map(HostRgbaFrame::as_bytes),
        Some(&[255, 0, 0, 255, 255, 0, 0, 255][..])
    );

    backbuffer = Some(HostRgbaFrame::filled(2, 1, [0, 0, 255, 255]));
    assert!(!capture_native_resize_snapshot(
        &mut snapshot,
        &mut backbuffer
    ));
    assert!(backbuffer.is_none());
    assert_eq!(
        snapshot.as_ref().map(HostRgbaFrame::as_bytes),
        Some(&[255, 0, 0, 255, 255, 0, 0, 255][..])
    );
}

#[test]
fn native_resize_snapshot_acquisition_prefers_a_same_size_backbuffer() {
    let mut snapshot = None;
    let mut backbuffer = Some(HostRgbaFrame::filled(2, 1, [255, 0, 0, 255]));
    let fallback_builds = Cell::new(0_u32);

    let first = acquire_native_resize_snapshot(&mut snapshot, &mut backbuffer, || {
        fallback_builds.set(fallback_builds.get() + 1);
        HostRgbaFrame::filled(2, 1, [0, 0, 255, 255])
    });
    let second = acquire_native_resize_snapshot(&mut snapshot, &mut backbuffer, || {
        fallback_builds.set(fallback_builds.get() + 1);
        HostRgbaFrame::filled(2, 1, [0, 0, 255, 255])
    });

    assert_eq!(first, NativeResizeSnapshotAcquisition::CapturedBackbuffer);
    assert_eq!(second, NativeResizeSnapshotAcquisition::Reused);
    assert_eq!(fallback_builds.get(), 0);
    assert!(backbuffer.is_none());
    assert_eq!(
        snapshot.as_ref().map(HostRgbaFrame::as_bytes),
        Some(&[255, 0, 0, 255, 255, 0, 0, 255][..])
    );
}

#[test]
fn native_resize_snapshot_acquisition_builds_the_fallback_exactly_once() {
    let mut snapshot = None;
    let mut backbuffer = None;
    let fallback_builds = Cell::new(0_u32);

    let first = acquire_native_resize_snapshot(&mut snapshot, &mut backbuffer, || {
        fallback_builds.set(fallback_builds.get() + 1);
        HostRgbaFrame::filled(2, 1, [255, 0, 0, 255])
    });
    let second = acquire_native_resize_snapshot(&mut snapshot, &mut backbuffer, || {
        fallback_builds.set(fallback_builds.get() + 1);
        HostRgbaFrame::filled(2, 1, [0, 0, 255, 255])
    });

    assert_eq!(first, NativeResizeSnapshotAcquisition::BuiltFallback);
    assert_eq!(second, NativeResizeSnapshotAcquisition::Reused);
    assert_eq!(fallback_builds.get(), 1);
}

#[test]
fn native_resize_snapshot_scales_with_smooth_physical_pixel_sampling() {
    let mut snapshot = HostRgbaFrame::filled(2, 2, [255, 0, 0, 255]);
    snapshot.fill_rect(
        &FrameRect {
            x: 1.0,
            y: 0.0,
            width: 1.0,
            height: 2.0,
        },
        [0, 0, 255, 255],
    );
    snapshot.fill_rect(
        &FrameRect {
            x: 0.0,
            y: 1.0,
            width: 1.0,
            height: 1.0,
        },
        [0, 255, 0, 255],
    );
    let mut buffer = vec![0_u32; 16];

    copy_scaled_rgba_to_softbuffer(&snapshot, &mut buffer, (4, 4));

    assert_eq!(buffer[0], 0xff0000);
    assert_eq!(buffer[3], 0x0000ff);
    assert_eq!(buffer[12], 0x00ff00);
    assert_eq!(buffer[15], 0x0000ff);
    assert!(buffer[1] != 0xff0000 && buffer[1] != 0x0000ff);
    assert!(buffer[2] != 0xff0000 && buffer[2] != 0x0000ff);
    assert!(buffer[5] != 0xff0000 && buffer[5] != 0x00ff00);
}

#[test]
fn native_resize_snapshot_downscale_blends_source_edges_without_nearest_blocks() {
    let mut snapshot = HostRgbaFrame::filled(3, 1, [255, 0, 0, 255]);
    snapshot.fill_rect(
        &FrameRect {
            x: 1.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        [0, 255, 0, 255],
    );
    snapshot.fill_rect(
        &FrameRect {
            x: 2.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        [0, 0, 255, 255],
    );
    let mut buffer = vec![0_u32; 2];

    copy_scaled_rgba_to_softbuffer(&snapshot, &mut buffer, (2, 1));

    assert_eq!(buffer, vec![0xe18900, 0x0089e1]);
}

#[test]
fn softbuffer_native_resize_product_path_uses_the_frozen_raster_snapshot() {
    let presenter_source = include_str!("../softbuffer.rs");
    let trait_impl = presenter_source
        .split("impl HostChromePresenter for SoftbufferHostPresenter")
        .nth(1)
        .expect("softbuffer presenter trait impl");
    assert!(trait_impl.contains("SoftbufferHostPresenter::present_during_native_resize"));

    let present_source = include_str!("present.rs");
    let ordinary_present = present_source
        .split("fn present(")
        .nth(1)
        .and_then(|source| source.split("fn present_during_native_resize").next())
        .expect("ordinary softbuffer present");
    let clear_snapshot = ordinary_present
        .find("presenter.native_resize_snapshot = None")
        .expect("ordinary present must end the native resize transaction");
    let plan_present = ordinary_present
        .find("plan_present_for_diagnostics")
        .expect("ordinary present diagnostics plan");
    assert!(clear_snapshot < plan_present);

    let native_resize = present_source
        .split("fn present_during_native_resize")
        .nth(1)
        .expect("specialized softbuffer native resize present");
    assert!(native_resize.contains("copy_scaled_rgba_to_softbuffer"));
    assert!(!native_resize.contains("build_chrome_command_stream"));
    assert!(!native_resize.contains("repaint_backbuffer"));
}

#[test]
fn softbuffer_damage_rect_clamps_to_surface_bounds() {
    let damage = FrameRect {
        x: -4.2,
        y: 1.2,
        width: 12.6,
        height: 3.4,
    };

    let rect =
        softbuffer_damage_rect(Some(&damage), (8, 4)).expect("damage should overlap the surface");

    assert_eq!(rect.x, 0);
    assert_eq!(rect.y, 1);
    assert_eq!(rect.width.get(), 8);
    assert_eq!(rect.height.get(), 3);
}

#[test]
fn overlay_text_change_expands_region_damage_without_full_repaint() {
    let damage = region_damage();

    let expanded = damage_with_debug_overlay(
        Some(damage.clone()),
        Some("FPS 59"),
        "FPS 60",
        (200, 120),
        &HostWindowPresentationData::default(),
    )
    .expect("changed overlay should keep region repaint damage");

    assert_eq!(expanded.x, 10.0);
    assert_eq!(expanded.y, 6.0);
    assert!(expanded.width > damage.width);
    assert!(expanded.height > damage.height);
}

#[test]
fn unchanged_overlay_text_keeps_existing_region_damage() {
    let damage = region_damage();

    let unchanged = damage_with_debug_overlay(
        Some(damage.clone()),
        Some("FPS 60"),
        "FPS 60",
        (200, 120),
        &HostWindowPresentationData::default(),
    );

    assert_eq!(unchanged, Some(damage));
}

#[test]
fn overlay_text_change_does_not_turn_full_repaint_into_region_damage() {
    let damage = damage_with_debug_overlay(
        None,
        Some("FPS 59"),
        "FPS 60",
        (200, 120),
        &HostWindowPresentationData::default(),
    );

    assert_eq!(damage, None);
}

#[test]
fn overlay_text_change_expands_region_damage_to_presentation_top_bar_height() {
    let damage = top_bar_probe_damage();
    let presentation = presentation_with_top_bar_height(58.0);

    let expanded = damage_with_debug_overlay(
        Some(damage.clone()),
        Some("FPS 59"),
        "FPS 60",
        (200, 120),
        &presentation,
    )
    .expect("changed overlay should keep region repaint damage");

    assert_eq!(expanded.y, 6.0);
    assert_eq!(expanded.height, 46.0);
}

#[test]
fn presenter_diagnostics_plan_same_frame_overlay_pixels_match_expanded_region_damage() {
    let planned = plan_present_for_diagnostics(
        &HostRefreshDiagnostics::default(),
        true,
        Some(STARTUP_REFRESH_DIAGNOSTICS_OVERLAY),
        &presentation_with_top_bar_height(58.0),
        Some(top_bar_probe_damage()),
        HostInvalidationDiagnostics::default(),
        (200, 120),
    );
    let damage = planned
        .damage
        .as_ref()
        .expect("changed overlay text should expand region damage");
    let expected_pixels = damage_pixel_count(damage, (200, 120));

    assert_eq!(planned.diagnostics.present_count, 1);
    assert_eq!(planned.diagnostics.full_paint_count, 0);
    assert_eq!(planned.diagnostics.region_paint_count, 1);
    assert_eq!(planned.diagnostics.painted_pixel_count, expected_pixels);
    assert_eq!(damage.y, 6.0);
    assert_eq!(damage.height, 46.0);
    assert!(
        planned
            .overlay_text
            .contains(&format!("pixels {expected_pixels}")),
        "overlay text should report the same painted pixels as diagnostics: {}",
        planned.overlay_text
    );
    assert_eq!(
        planned.presentation.host_shell.debug_refresh_rate.as_str(),
        planned.overlay_text
    );
}

#[test]
fn presenter_diagnostics_plan_full_repaint_records_full_pixels_in_same_frame_overlay() {
    let planned = plan_present_for_diagnostics(
        &HostRefreshDiagnostics::default(),
        false,
        None,
        &HostWindowPresentationData::default(),
        Some(region_damage()),
        HostInvalidationDiagnostics {
            slow_path_rebuild_count: 2,
            render_rebuild_count: 3,
            paint_only_request_count: 4,
        },
        (200, 120),
    );

    assert_eq!(planned.damage, None);
    assert_eq!(planned.diagnostics.full_paint_count, 1);
    assert_eq!(planned.diagnostics.region_paint_count, 0);
    assert_eq!(planned.diagnostics.painted_pixel_count, 24_000);
    assert!(planned.overlay_text.contains("pixels 24000"));
    assert!(planned.overlay_text.contains("slow 2"));
    assert!(planned.overlay_text.contains("render 3"));
    assert!(planned.overlay_text.contains("paint-only 4"));
}
