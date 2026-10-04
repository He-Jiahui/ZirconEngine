use super::*;

#[test]
fn collapsed_picker_surfaces_do_not_expand_into_drawable_frames() {
    let outer = FrameRect {
        x: 12.0,
        y: 8.0,
        width: 0.0,
        height: 0.0,
    };
    let field = picker_field_frame(&outer);
    let icon = picker_field_icon_frame(&field);
    let popup = picker_popup_frame(&outer, &field);
    let header = picker_popup_header_frame(&popup);
    let cell = picker_popup_cell_frame(&popup);

    for frame in [field, icon, popup, header, cell] {
        assert_eq!((frame.width, frame.height), (0.0, 0.0));
    }
}

#[test]
fn non_finite_picker_inputs_do_not_emit_non_finite_geometry() {
    let outer = FrameRect {
        x: f32::INFINITY,
        y: f32::NAN,
        width: 72.0,
        height: 88.0,
    };
    let root = picker_root_frame(&outer);
    let field = picker_field_frame(&outer);
    let icon = picker_field_icon_frame(&field);
    let popup = picker_popup_frame(&outer, &field);
    let header = picker_popup_header_frame(&popup);
    let cell = picker_popup_cell_frame(&popup);

    for frame in [root, field, icon, popup, header, cell] {
        assert!(frame.x.is_finite() && frame.y.is_finite());
        assert!(frame.width.is_finite() && frame.height.is_finite());
        assert!(frame.right().is_finite() && frame.bottom().is_finite());
    }
}

#[test]
fn picker_root_frame_collapses_extents_that_overflow_the_coordinate_range() {
    let root = picker_root_frame(&FrameRect {
        x: f32::MAX,
        y: f32::MAX,
        width: 72.0,
        height: 88.0,
    });

    assert_eq!((root.width, root.height), (0.0, 0.0));
    assert!(root.right().is_finite() && root.bottom().is_finite());
}
