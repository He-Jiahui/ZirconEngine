//! 屏幕形状供精确命中和粗筛范围共同消费；环分段顺序来自世界投影，评分仍须表达整个形状的接近度。

use zircon_runtime_interface::math::Vec2;

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
