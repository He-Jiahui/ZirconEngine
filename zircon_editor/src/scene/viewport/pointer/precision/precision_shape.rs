use zircon_runtime::core::math::Vec2;

#[derive(Clone, Debug)]
pub(in crate::scene::viewport::pointer) enum PrecisionShape {
    Line {
        start: Vec2,
        end: Vec2,
        radius_px: f32,
        threshold_px: f32,
        depth: f32,
    },
    Circle {
        center: Vec2,
        radius_px: f32,
        threshold_px: f32,
        depth: f32,
    },
    Ring {
        segments: Vec<(Vec2, Vec2)>,
        radius_px: f32,
        thickness_px: f32,
        threshold_px: f32,
        depth: f32,
    },
}
