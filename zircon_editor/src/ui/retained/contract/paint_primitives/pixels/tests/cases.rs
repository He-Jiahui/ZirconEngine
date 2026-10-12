use super::geometry::{clamp_to_ordered_range, rounded_rect_pixel_coverage};
use super::span::write_pixel_with_coverage;
use super::{fill_rounded_border_pixels, fill_rounded_pixel_rect};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_geometry::PixelRect;

#[test]
fn rounded_rect_center_clamp_tolerates_crossed_float_bounds() {
    let clamped = clamp_to_ordered_range(40.0, 40.0, 39.999_992);
    assert!((clamped - 39.999_996).abs() <= f32::EPSILON);
}

#[test]
fn rounded_fill_supersamples_fractional_corner_coverage() {
    let mut frame = HostRgbaFrame::filled(8, 8, [0, 0, 0, 255]);
    let rect = FrameRect {
        x: 1.25,
        y: 1.25,
        width: 5.5,
        height: 5.5,
    };
    let target = PixelRect::from_frame(&rect, None, frame.width(), frame.height())
        .expect("visible rounded target");

    fill_rounded_pixel_rect(&mut frame, &target, &rect, [255, 255, 255, 255], 2.5);

    let left_edge = pixel(&frame, 2, 1);
    let right_edge = pixel(&frame, 5, 1);
    assert!(
        (1..255).contains(&left_edge[0]),
        "rounded edge must retain fractional coverage, got {left_edge:?}"
    );
    assert_eq!(left_edge, right_edge, "corner coverage must stay symmetric");
    assert_eq!(pixel(&frame, 3, 3), [255, 255, 255, 255]);
    assert_eq!(pixel(&frame, 0, 0), [0, 0, 0, 255]);
}

#[test]
fn rounded_fill_resolves_edge_coverage_at_eight_by_eight_precision() {
    let rect = FrameRect {
        x: 1.05,
        y: 1.05,
        width: 9.4,
        height: 8.7,
    };

    let coverage = rounded_rect_pixel_coverage(2, 1, &rect, 3.35);

    assert_eq!(coverage, 22.0 / 64.0);
    assert_ne!(
        (coverage * 16.0).fract(),
        0.0,
        "edge coverage must not collapse to the former four-by-four sample steps"
    );
}

#[test]
fn rounded_coverage_blends_half_white_over_black_in_linear_light() {
    let mut frame = HostRgbaFrame::filled(1, 1, [0, 0, 0, 255]);

    write_pixel_with_coverage(&mut frame, 0, 0, [255, 255, 255, 255], 0.5);

    let resolved = pixel(&frame, 0, 0);
    assert!(
        (187..=189).contains(&resolved[0]),
        "half coverage must encode near sRGB 188, got {resolved:?}"
    );
    assert_eq!(resolved[0], resolved[1]);
    assert_eq!(resolved[1], resolved[2]);
    assert_eq!(resolved[3], 255);
}

#[test]
fn rounded_border_supersamples_outer_and_inner_edges() {
    let mut frame = HostRgbaFrame::filled(10, 10, [0, 0, 0, 255]);
    let rect = FrameRect {
        x: 1.25,
        y: 1.25,
        width: 7.5,
        height: 7.5,
    };
    let target = PixelRect::from_frame(&rect, None, frame.width(), frame.height())
        .expect("visible rounded border target");

    fill_rounded_border_pixels(&mut frame, &target, &rect, [255, 255, 255, 255], 1.5, 3.0);

    let outer_edge = pixel(&frame, 4, 1);
    let inner_edge = pixel(&frame, 4, 2);
    assert!(
        (1..255).contains(&outer_edge[0]),
        "outer edge: {outer_edge:?}"
    );
    assert!(
        (1..255).contains(&inner_edge[0]),
        "inner edge: {inner_edge:?}"
    );
    assert_eq!(pixel(&frame, 5, 5), [0, 0, 0, 255]);
}

fn pixel(frame: &HostRgbaFrame, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * frame.width() as usize) + x as usize) * 4;
    frame.as_bytes()[offset..offset + 4]
        .try_into()
        .expect("RGBA pixel")
}
