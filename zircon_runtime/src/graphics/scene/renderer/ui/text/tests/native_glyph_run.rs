use super::{
    aligned_device_text_start_x, native_bitmap_atlas_relative_baseline, vertical_subpixel_bin,
    UiFrame, UiTextAlign, UiTextDirection,
};

#[test]
fn native_bitmap_glyph_run_retains_four_vertical_raster_phases() {
    assert_eq!(vertical_subpixel_bin(12.0), 0);
    assert_eq!(vertical_subpixel_bin(12.26), 1);
    assert_eq!(vertical_subpixel_bin(12.51), 2);
    assert_eq!(vertical_subpixel_bin(12.76), 3);
}

#[test]
fn native_bitmap_glyph_run_keeps_logical_start_end_alignment_after_device_origin_snap() {
    let frame = UiFrame::new(10.0, 7.0, 100.0, 24.0);

    assert_eq!(
        aligned_device_text_start_x(
            frame,
            UiTextAlign::Start,
            UiTextDirection::LeftToRight,
            30.0,
        ),
        10.0
    );
    assert_eq!(
        aligned_device_text_start_x(
            frame,
            UiTextAlign::Center,
            UiTextDirection::LeftToRight,
            f32::NAN,
        ),
        60.0
    );
    assert_eq!(
        aligned_device_text_start_x(
            frame,
            UiTextAlign::Start,
            UiTextDirection::RightToLeft,
            30.0,
        ),
        80.0
    );
    assert_eq!(
        aligned_device_text_start_x(frame, UiTextAlign::End, UiTextDirection::LeftToRight, 30.0,),
        80.0
    );
    assert_eq!(
        aligned_device_text_start_x(frame, UiTextAlign::End, UiTextDirection::RightToLeft, 30.0,),
        10.0
    );
}

#[test]
fn native_bitmap_glyph_run_prefers_layout_decoration_baseline_and_sanitizes_invalid_input() {
    assert_eq!(
        native_bitmap_atlas_relative_baseline(Some(35.0), 20.0, 30.0, 20.0),
        15.0
    );
    assert_eq!(
        native_bitmap_atlas_relative_baseline(Some(f32::NAN), 20.0, 30.0, 20.0),
        21.0
    );
}
