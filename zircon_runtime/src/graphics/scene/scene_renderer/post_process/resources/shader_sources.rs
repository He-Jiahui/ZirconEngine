pub(in crate::graphics::scene::scene_renderer::post_process) const POST_PROCESS_SCREEN_SPACE_REFLECTION_SHADER: &str =
    include_str!("../shaders/post_process_screen_space_reflection.wgsl");

pub(in crate::graphics::scene::scene_renderer::post_process) const POST_PROCESS_SHADER: &str = concat!(
    include_str!("../shaders/post_process.wgsl"),
    "\n",
    include_str!("../shaders/post_process_screen_space_reflection.wgsl")
);
pub(in crate::graphics::scene::scene_renderer::post_process) const OUTPUT_TRANSFER_SHADER: &str =
    include_str!("../shaders/output_transfer.wgsl");
pub(in crate::graphics::scene::scene_renderer::post_process) const UPSCALE_SHADER: &str =
    include_str!("../shaders/upscale.wgsl");
pub(in crate::graphics::scene::scene_renderer::post_process) const HALF_RES_TRANSPARENCY_SHADER:
    &str = include_str!("../shaders/half_res_transparency.wgsl");
pub(in crate::graphics::scene::scene_renderer::post_process) const FXAA_SHADER: &str =
    include_str!("../shaders/fxaa.wgsl");
pub(in crate::graphics::scene::scene_renderer::post_process) const SMAA_SHADER: &str =
    include_str!("../shaders/smaa.wgsl");

#[cfg(test)]
mod tests {
    use super::{
        FXAA_SHADER, HALF_RES_TRANSPARENCY_SHADER, OUTPUT_TRANSFER_SHADER,
        POST_PROCESS_SCREEN_SPACE_REFLECTION_SHADER, POST_PROCESS_SHADER, SMAA_SHADER,
        UPSCALE_SHADER,
    };

    const POST_PROCESS_BASE_SHADER: &str = include_str!("../shaders/post_process.wgsl");
    const COLOR_LUT_BAKE_SHADER: &str = include_str!("../shaders/color_lut_bake.wgsl");

    #[test]
    fn post_process_shader_source_assembles_screen_space_reflection_module() {
        assert!(POST_PROCESS_BASE_SHADER.contains("fn fs_main"));
        assert!(!POST_PROCESS_BASE_SHADER.contains("fn trace_screen_space_reflection"));
        assert!(POST_PROCESS_SCREEN_SPACE_REFLECTION_SHADER
            .contains("fn trace_screen_space_reflection"));
        assert!(POST_PROCESS_SHADER.contains("fn fs_main"));
        assert!(POST_PROCESS_SHADER.contains("fn trace_screen_space_reflection"));
        assert!(POST_PROCESS_SHADER.contains("fn resolve_screen_space_reflection_history"));
        assert!(POST_PROCESS_SHADER.contains("fn fs_screen_space_reflection_resolve"));
        assert!(POST_PROCESS_SHADER.contains("fn load_resolved_screen_space_reflection"));
    }

    #[test]
    fn output_transfer_shader_source_declares_single_texture_input() {
        assert!(OUTPUT_TRANSFER_SHADER.contains("@binding(0) var tonemapped_tex"));
        assert!(OUTPUT_TRANSFER_SHADER.contains("@binding(1) var<uniform> params"));
        assert!(OUTPUT_TRANSFER_SHADER.contains("textureLoad(tonemapped_tex"));
    }

    #[test]
    fn upscale_shader_source_declares_filtered_source_sampling() {
        assert!(UPSCALE_SHADER.contains("@binding(0) var source_tex"));
        assert!(UPSCALE_SHADER.contains("@binding(1) var source_sampler"));
        assert!(UPSCALE_SHADER.contains("textureSampleLevel(source_tex"));
    }

    #[test]
    fn half_resolution_transparency_shader_declares_conservative_depth_and_bilateral_composite() {
        assert!(HALF_RES_TRANSPARENCY_SHADER.contains("fn fs_depth_downsample"));
        assert!(HALF_RES_TRANSPARENCY_SHADER.contains("min(min(depth00, depth10)"));
        assert!(HALF_RES_TRANSPARENCY_SHADER.contains("fn fs_composite"));
        assert!(HALF_RES_TRANSPARENCY_SHADER.contains("fn depth_weight"));
        assert!(HALF_RES_TRANSPARENCY_SHADER.contains("depth_sigma: f32"));
        assert!(HALF_RES_TRANSPARENCY_SHADER.contains("@binding(4) var<uniform>"));
    }

    #[test]
    fn fxaa_shader_source_declares_terminal_input_and_neighbor_filter() {
        assert!(FXAA_SHADER.contains("@binding(0) var terminal_input_tex"));
        assert!(FXAA_SHADER.contains("@binding(1) var<uniform> params"));
        assert!(FXAA_SHADER.contains("fn apply_fxaa"));
        assert!(FXAA_SHADER.contains("textureLoad(terminal_input_tex"));
    }

    #[test]
    fn smaa_shader_source_declares_terminal_input_and_edge_filter() {
        assert!(SMAA_SHADER.contains("@binding(0) var terminal_input_tex"));
        assert!(SMAA_SHADER.contains("@binding(1) var smaa_stage_tex"));
        assert!(SMAA_SHADER.contains("@binding(2) var<uniform> params"));
        assert!(SMAA_SHADER.contains("fn fs_edge"));
        assert!(SMAA_SHADER.contains("fn fs_blend"));
        assert!(SMAA_SHADER.contains("fn fs_resolve"));
        assert!(SMAA_SHADER.contains("fn apply_smaa_resolve"));
        assert!(SMAA_SHADER.contains("fn smaa_edge_weight"));
        assert!(SMAA_SHADER.contains("textureLoad(terminal_input_tex"));
    }

    #[test]
    fn color_lut_bake_shader_source_declares_internal_bake_outputs() {
        assert!(COLOR_LUT_BAKE_SHADER.contains("texture_storage_3d<rgba16float, write>"));
        assert!(COLOR_LUT_BAKE_SHADER.contains("fn apply_tonemap"));
        assert!(COLOR_LUT_BAKE_SHADER.contains("fn apply_color_grading"));
        assert!(COLOR_LUT_BAKE_SHADER.contains("textureStore(color_lut_out"));
    }
}
