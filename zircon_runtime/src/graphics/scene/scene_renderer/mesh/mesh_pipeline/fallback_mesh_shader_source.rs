//! 组合共享 mesh WGSL 片段，供本模块测试检查入口、资源绑定与光照约定。
pub(crate) const FALLBACK_MESH_SHADER: &str = concat!(
    include_str!("../shaders/zr_gpu_scene.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_light_cookie.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_irradiance_volume.wgsl"),
    "\n",
    include_str!("../../lighting/shaders/zr_light_grid.wgsl"),
    "\n",
    include_str!("../../shadow/shaders/zr_shadow.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_volumetric.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_lightmap.wgsl"),
    "\n",
    include_str!("../../../../shader/includes/zr_pbr_common.wgsl"),
    "\n",
    include_str!("../../../../shader/includes/zr_pbr_extras_core.wgsl"),
    "\n",
    include_str!("../../../../shader/includes/zr_normal.wgsl"),
    "\n",
    include_str!("../shaders/fallback_mesh.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_procedural_sky.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_environment_core.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_environment_generic_api.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_environment.wgsl")
);

#[cfg(test)]
#[path = "tests/fallback_mesh_shader_source.rs"]
mod tests;

#[cfg(test)]
include!("tests/fallback_mesh_shader_source_cases.rs");
