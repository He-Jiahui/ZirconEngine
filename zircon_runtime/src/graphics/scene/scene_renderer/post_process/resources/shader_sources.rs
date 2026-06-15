pub(in crate::graphics::scene::scene_renderer::post_process) const POST_PROCESS_SCREEN_SPACE_REFLECTION_SHADER: &str =
    include_str!("../shaders/post_process_screen_space_reflection.wgsl");

pub(in crate::graphics::scene::scene_renderer::post_process) const POST_PROCESS_SHADER: &str = concat!(
    include_str!("../shaders/post_process.wgsl"),
    "\n",
    include_str!("../shaders/post_process_screen_space_reflection.wgsl")
);
pub(in crate::graphics::scene::scene_renderer::post_process) const OUTPUT_TRANSFER_SHADER: &str =
    include_str!("../shaders/output_transfer.wgsl");

#[cfg(test)]
mod tests {
    use super::{
        OUTPUT_TRANSFER_SHADER, POST_PROCESS_SCREEN_SPACE_REFLECTION_SHADER, POST_PROCESS_SHADER,
    };

    const POST_PROCESS_BASE_SHADER: &str = include_str!("../shaders/post_process.wgsl");

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
        assert!(OUTPUT_TRANSFER_SHADER.contains("textureLoad(tonemapped_tex"));
    }
}
