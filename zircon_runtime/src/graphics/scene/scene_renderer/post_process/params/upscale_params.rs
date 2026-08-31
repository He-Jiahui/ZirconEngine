use bytemuck::{Pod, Zeroable};

use crate::core::math::UVec2;

/// Logical source and destination extents for the spatial upscale pass.
///
/// Graph allocations may be aligned larger than their logical ViewFamily viewport. The shader
/// must therefore derive normalized coordinates from these values rather than from texture size.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct UpscaleParams {
    pub(super) input_output_size: [u32; 4],
}

impl UpscaleParams {
    pub(super) fn from_logical_sizes(input: UVec2, output: UVec2) -> Self {
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
mod tests {
    use crate::core::math::UVec2;

    use super::UpscaleParams;

    #[test]
    fn upscale_params_encode_logical_sizes_not_aligned_allocations() {
        assert_eq!(
            UpscaleParams::from_logical_sizes(UVec2::new(1440, 810), UVec2::new(1920, 1080),)
                .input_output_size,
            [1440, 810, 1920, 1080]
        );
    }
}
