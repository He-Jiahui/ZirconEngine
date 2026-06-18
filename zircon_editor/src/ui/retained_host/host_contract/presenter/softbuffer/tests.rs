use super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::diagnostics::{
    HostInvalidationDiagnostics, HostRefreshDiagnostics, STARTUP_REFRESH_DIAGNOSTICS_OVERLAY,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::diagnostics::{damage_with_debug_overlay, plan_present_for_diagnostics};
use super::surface_io::{copy_rgba_to_softbuffer, damage_pixel_count, softbuffer_damage_rect};

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
