//! 编译期嵌入后处理 WGSL，并把屏幕空间反射源码拼接到共享 shader；测试同时守护绑定和入口契约。
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
#[path = "tests/shader_sources.rs"]
mod tests;
