use bytemuck::{Pod, Zeroable};

use crate::core::math::UVec2;

/// Logical source and destination extents for the spatial upscale pass.
///
/// Graph allocations may be aligned larger than their logical ViewFamily viewport. The shader
/// must therefore derive normalized coordinates from these values rather than from texture size.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct UpscaleParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) input_output_size: [u32; 4],
}

impl UpscaleParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) fn from_logical_sizes(
        input: UVec2,
        output: UVec2,
    ) -> Self {
        Self {
            input_output_size: [
                input.x.max(1),
                input.y.max(1),
                output.x.max(1),
                output.y.max(1),
            ],
        }
    }
}

#[cfg(test)]
#[path = "tests/upscale_params.rs"]
mod tests;
