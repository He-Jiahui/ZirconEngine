use zircon_math::Vec2;
use zircon_ui::UiFrame;

pub(in crate::editing::viewport::pointer) fn frame_from_points(
    points: &[Vec2],
    expand: f32,
) -> UiFrame {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for point in points {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    UiFrame::new(
        min_x - expand,
        min_y - expand,
        (max_x - min_x) + expand * 2.0,
        (max_y - min_y) + expand * 2.0,
    )
}
