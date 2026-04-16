use crate::types::EditorOrRuntimeFrame;

pub(super) fn viewport_size(frame: &EditorOrRuntimeFrame) -> zircon_math::UVec2 {
    zircon_math::UVec2::new(frame.viewport.size.x.max(1), frame.viewport.size.y.max(1))
}

pub(super) fn texture_extent(size: zircon_math::UVec2) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: size.x.max(1),
        height: size.y.max(1),
        depth_or_array_layers: 1,
    }
}
