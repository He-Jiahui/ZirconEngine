use super::TableAxes;
use zircon_runtime_interface::ui::layout::UiFrame;

#[test]
fn physical_table_frames_keep_extreme_absolute_coordinates_finite() {
    let container = UiFrame::new(f32::MAX, 0.0, f32::MAX, f32::MAX);
    let horizontal =
        TableAxes::HorizontalTb.physical_frame(container, f32::MAX, f32::MAX, f32::MAX, f32::MAX);
    let vertical =
        TableAxes::VerticalRl.physical_frame(container, f32::MAX, f32::MAX, f32::MAX, f32::MAX);

    assert!(horizontal.x.is_finite());
    assert!(horizontal.y.is_finite());
    assert!(vertical.x.is_finite());
    assert!(vertical.y.is_finite());
    assert_eq!(vertical.x, 0.0);
}
