use zircon_runtime_interface::math::Vec2;
use zircon_runtime_interface::ui::layout::UiFrame;

pub(in crate::scene::viewport::pointer) fn frame_from_points(
    points: impl IntoIterator<Item = Vec2>,
    expand: f32,
) -> Option<UiFrame> {
    let mut points = points.into_iter();
    let first = points.next()?;
    let mut min = first;
    let mut max = first;
    for point in points {
        min = min.min(point);
        max = max.max(point);
    }
    Some(UiFrame::new(
        min.x - expand,
        min.y - expand,
        (max.x - min.x) + expand * 2.0,
        (max.y - min.y) + expand * 2.0,
    ))
}

#[cfg(test)]
#[path = "frame_from_points/vec2_bounds_tests.rs"]
mod vec2_bounds_tests;
