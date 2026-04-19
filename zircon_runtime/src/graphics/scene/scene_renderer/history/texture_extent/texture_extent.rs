use crate::core::math::UVec2;

pub(in crate::graphics::scene::scene_renderer::history) fn texture_extent(size: UVec2) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: size.x.max(1),
        height: size.y.max(1),
        depth_or_array_layers: 1,
    }
}
