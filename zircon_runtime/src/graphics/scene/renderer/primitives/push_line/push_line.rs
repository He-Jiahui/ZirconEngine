use crate::core::framework::render::OverlayLineSegment;

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

/// 将上层已抽取的线段保持端点顺序传给 gizmo 线条管线，顶点单位与世界坐标一致。
pub(crate) fn push_line(vertices: &mut Vec<LineVertex>, line: &OverlayLineSegment) {
    vertices.push(LineVertex::new(line.start, line.color));
    vertices.push(LineVertex::new(line.end, line.color));
}
