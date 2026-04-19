use zircon_framework::render::OverlayLineSegment;

use crate::scene::scene_renderer::primitives::LineVertex;

pub(crate) fn push_line(vertices: &mut Vec<LineVertex>, line: &OverlayLineSegment) {
    vertices.push(LineVertex::new(line.start, line.color));
    vertices.push(LineVertex::new(line.end, line.color));
}
