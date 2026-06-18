use std::path::{Path, PathBuf};

use crate::core::framework::render::{
    ShaderPassType, ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport,
    ShaderVariantPrewarmRequest,
};
use crate::graphics::scene::{default_pipeline_key, FALLBACK_MESH_SHADER};
use crate::graphics::shader::{prewarm_shader_variants_to_disk, ShaderVariantCacheDisk};

const MESH_SHADER_TEMPLATE_REVISION: &str = "mesh-template-v1";
const MESH_SHADER_NAGA_VERSION: &str = "naga-29.0.1";
const MESH_SHADER_WGPU_VERSION: &str = "wgpu-29.0.1";
const MESH_SHADER_PLATFORM_TOKEN: &str = "wgpu-runtime";

pub fn prewarm_shader_variants(
    manifest: &ShaderVariantPrewarmManifest,
    cache_dir: impl AsRef<Path>,
) -> ShaderVariantPrewarmReport {
    prewarm_shader_variants_to_disk(manifest, cache_dir)
}

pub fn builtin_fallback_shader_prewarm_manifest() -> ShaderVariantPrewarmManifest {
    let pipeline_key = default_pipeline_key();
    ShaderVariantPrewarmManifest::new(vec![ShaderVariantPrewarmRequest {
        key: pipeline_key.shader_variant_key(ShaderPassType::Forward, MESH_SHADER_PLATFORM_TOKEN),
        wgsl_source: FALLBACK_MESH_SHADER.to_string(),
        include_content_hashes: vec![blake3::hash(FALLBACK_MESH_SHADER.as_bytes())
            .to_hex()
            .to_string()],
        template_revision: MESH_SHADER_TEMPLATE_REVISION.to_string(),
        naga_version: MESH_SHADER_NAGA_VERSION.to_string(),
        wgpu_version: MESH_SHADER_WGPU_VERSION.to_string(),
    }])
}

pub fn default_shader_variant_cache_root_for_project(project_root: impl AsRef<Path>) -> PathBuf {
    ShaderVariantCacheDisk::default_project_root(project_root.as_ref())
}

pub fn default_staged_shader_variant_cache_root_for_project(
    project_root: impl AsRef<Path>,
) -> PathBuf {
    ShaderVariantCacheDisk::default_staged_project_root(project_root.as_ref())
}
