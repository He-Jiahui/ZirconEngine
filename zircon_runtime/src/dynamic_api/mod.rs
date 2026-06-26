//! Dynamic runtime library API exported through `zircon_runtime_interface`.

mod camera_controller;
mod exports;
mod frame;
mod runtime_loop;
mod session;
mod shader_prewarm;
mod surface;

pub use exports::zircon_runtime_get_api_v1;
pub use shader_prewarm::{
    builtin_fallback_shader_prewarm_manifest, builtin_standard_material_shader_prewarm_manifest,
    builtin_standard_material_shader_prewarm_manifest_for_geometry,
    default_shader_variant_cache_root_for_project,
    default_staged_shader_variant_cache_root_for_project, prewarm_shader_variants,
};

#[cfg(test)]
mod tests;
