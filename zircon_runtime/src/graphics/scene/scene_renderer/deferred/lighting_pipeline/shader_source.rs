pub(in crate::graphics::scene::scene_renderer::deferred) const DEFERRED_LIGHTING_SHADER: &str = concat!(
    include_str!("../../mesh/shaders/zr_gpu_scene.wgsl"),
    "\n",
    include_str!("../../lighting/shaders/zr_light_grid.wgsl"),
    "\n",
    include_str!("../../shadow/shaders/zr_shadow.wgsl"),
    "\n",
    include_str!("../shaders/deferred_lighting.wgsl")
);
