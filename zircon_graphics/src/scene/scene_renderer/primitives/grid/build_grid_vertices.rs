use zircon_math::{Vec3, Vec4};

use crate::scene::scene_renderer::primitives::LineVertex;

pub(crate) fn build_grid_vertices() -> Vec<LineVertex> {
    let mut vertices = Vec::new();
    for index in -10..=10 {
        let color = if index == 0 {
            Vec4::new(0.24, 0.36, 0.88, 1.0)
        } else if index % 5 == 0 {
            Vec4::new(0.22, 0.24, 0.3, 1.0)
        } else {
            Vec4::new(0.16, 0.17, 0.2, 1.0)
        };
        let z = index as f32;
        vertices.push(LineVertex::new(Vec3::new(-10.0, 0.0, z), color));
        vertices.push(LineVertex::new(Vec3::new(10.0, 0.0, z), color));
        vertices.push(LineVertex::new(Vec3::new(z, 0.0, -10.0), color));
        vertices.push(LineVertex::new(Vec3::new(z, 0.0, 10.0), color));
    }
    vertices
}
