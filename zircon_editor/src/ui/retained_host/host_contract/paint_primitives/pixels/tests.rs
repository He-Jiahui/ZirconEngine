use super::geometry::clamp_to_ordered_range;
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
