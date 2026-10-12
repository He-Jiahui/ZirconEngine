//! Dynamic runtime library API exported through `zircon_runtime_interface`.

#[path = "bounded_json.rs"]
mod bounded_json;
#[path = "camera_controller.rs"]
mod camera_controller;
#[path = "exports.rs"]
mod exports;
#[path = "frame.rs"]
mod frame;
#[path = "runtime_loop.rs"]
mod runtime_loop;
#[path = "session.rs"]
mod session;
#[path = "shader_prewarm.rs"]
mod shader_prewarm;
#[path = "surface.rs"]
mod surface;

pub use exports::zircon_runtime_get_api_v8;
pub use session::{create_linked_runtime_session, RuntimeDynamicSessionError, RuntimeProjectError};
pub(crate) use shader_prewarm::builtin_standard_material_shader_prewarm_manifest_for_geometry_with_plugin_shading_models;
pub use shader_prewarm::{
    builtin_fallback_shader_prewarm_manifest, builtin_standard_material_shader_prewarm_manifest,
    builtin_standard_material_shader_prewarm_manifest_for_geometry,
    builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor,
    default_shader_variant_cache_root_for_project,
    default_staged_shader_variant_cache_root_for_project,
    material_surface_shader_prewarm_template_source, prewarm_shader_variants,
    prewarm_shader_variants_with_execution_budget,
    prewarm_shader_variants_with_wgpu_module_and_pipeline_validation,
    prewarm_shader_variants_with_wgpu_module_validation,
    prewarm_shader_variants_with_wgpu_pipeline_validation, ShaderPrewarmTemplateSource,
};

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
