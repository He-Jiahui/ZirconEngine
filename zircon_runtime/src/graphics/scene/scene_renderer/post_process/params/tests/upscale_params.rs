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
