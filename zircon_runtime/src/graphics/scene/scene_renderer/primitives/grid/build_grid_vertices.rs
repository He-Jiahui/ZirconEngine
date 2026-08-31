use crate::core::math::{Vec3, Vec4};

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

const GRID_HALF_EXTENT: i32 = 10;
const GRID_VERTICES_PER_INDEX: usize = 4;
const GRID_INDEX_COUNT: usize = (GRID_HALF_EXTENT * 2 + 1) as usize;

pub(crate) fn build_grid_vertices() -> Vec<LineVertex> {
    let mut vertices = Vec::with_capacity(GRID_INDEX_COUNT * GRID_VERTICES_PER_INDEX);
    let extent = GRID_HALF_EXTENT as f32;
    for index in -GRID_HALF_EXTENT..=GRID_HALF_EXTENT {
        let color = if index == 0 {
            Vec4::new(0.24, 0.36, 0.88, 1.0)
        } else if index % 5 == 0 {
            Vec4::new(0.22, 0.24, 0.3, 1.0)
        } else {
            Vec4::new(0.16, 0.17, 0.2, 1.0)
        };
        let z = index as f32;
        vertices.push(LineVertex::new(Vec3::new(-extent, 0.0, z), color));
        vertices.push(LineVertex::new(Vec3::new(extent, 0.0, z), color));
        vertices.push(LineVertex::new(Vec3::new(z, 0.0, -extent), color));
        vertices.push(LineVertex::new(Vec3::new(z, 0.0, extent), color));
    }
    vertices
}

#[cfg(test)]
#[path = "build_grid_vertices/capacity_tests.rs"]
mod capacity_tests;
