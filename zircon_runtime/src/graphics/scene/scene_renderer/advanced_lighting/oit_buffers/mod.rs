mod executors;
mod fragment_store_pipeline;
mod resolve_pipeline;

pub const OIT_FRAGMENT_STORE_EXECUTOR_ID: &str = "oit.fragment_store";
pub const OIT_RESOLVE_EXECUTOR_ID: &str = "oit.resolve";
pub const OIT_RESOLVE_SHADER_SOURCE: &str = include_str!("shaders/resolve.wgsl");
pub const OIT_DRAW_SHADER_SOURCE: &str = include_str!("../../../../shader/includes/zr_oit.wgsl");

pub(crate) use executors::registrations;
pub(in crate::graphics::scene::scene_renderer) use fragment_store_pipeline::OitFragmentStorePipeline;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oit_draw_shader_uses_per_pixel_atomic_count_and_fixed_layer_stride() {
        assert!(OIT_DRAW_SHADER_SOURCE.contains("atomicAdd(&oit_counts[pixel_index], 1u)"));
        assert!(OIT_DRAW_SHADER_SOURCE.contains("pixel_index * oit_settings.fragments_per_pixel"));
        assert!(OIT_DRAW_SHADER_SOURCE.contains("pack4x8unorm"));
        assert!(OIT_DRAW_SHADER_SOURCE.contains("bitcast<u32>(position.z)"));
        assert!(OIT_DRAW_SHADER_SOURCE.contains("physical_pixel - origin"));
        assert!(OIT_DRAW_SHADER_SOURCE
            .contains("@group(4) @binding(0) var<storage, read_write> oit_layers"));
        assert!(!OIT_DRAW_SHADER_SOURCE.contains("@group(3)"));
    }

    #[test]
    fn oit_resolve_shader_sorts_near_to_far_depth_and_bounds_exact_layers() {
        assert!(OIT_RESOLVE_SHADER_SOURCE.contains("const OIT_MAX_SORTED_FRAGMENTS: u32 = 32u"));
        assert!(OIT_RESOLVE_SHADER_SOURCE.contains("candidate.y < sorted_layers[j - 1u].y"));
        assert!(OIT_RESOLVE_SHADER_SOURCE.contains("min(oit_settings.sorted_fragment_max_count"));
        assert!(OIT_RESOLVE_SHADER_SOURCE.contains("physical_pixel - origin"));
    }
}
