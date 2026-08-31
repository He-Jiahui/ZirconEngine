use super::super::compiler::ShaderAbiBinding;

pub const FULLSCREEN_FRAME_GROUP: u32 = 0;
pub const FULLSCREEN_PASS_INPUT_GROUP: u32 = 1;
pub const FULLSCREEN_PARAMS_BINDING: ShaderAbiBinding = ShaderAbiBinding {
    group: 2,
    binding: 0,
};
pub const FULLSCREEN_FIRST_PASS_INPUT_BINDING: u32 = 0;
pub const FULLSCREEN_TRIANGLE_VERTEX_ENTRY: &str = "zr_fullscreen_triangle_vs";
