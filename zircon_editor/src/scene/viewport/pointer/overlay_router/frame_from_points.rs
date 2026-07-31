use zircon_runtime_interface::math::Vec2;
use zircon_runtime_interface::ui::layout::UiFrame;

pub(in crate::scene::viewport::pointer) fn frame_from_points(
    points: impl IntoIterator<Item = Vec2>,
    expand: f32,
) -> Option<UiFrame> {
    let mut points = points.into_iter();
    let first = points.next()?;
    let mut min_x = first.x;
    let mut min_y = first.y;
    let mut max_x = first.x;
    let mut max_y = first.y;
    for point in points {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    Some(UiFrame::new(
        min_x - expand,
        min_y - expand,
        (max_x - min_x) + expand * 2.0,
        (max_y - min_y) + expand * 2.0,
    ))
}
