use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantPrewarmManifest,
    ShaderVariantPrewarmRequest, ShadingModelDescriptor, GEOMETRY_SOURCE_ID_STATIC_MESH,
    SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::dynamic_api::{
    builtin_fallback_shader_prewarm_manifest,
    builtin_standard_material_shader_prewarm_manifest_for_geometry,
    builtin_standard_material_shader_prewarm_manifest_for_geometry_with_plugin_shading_models,
};
use crate::graphics::shader::ShaderTemplateAssemblyError;

use super::case::RegistryShaderCase;

pub(super) const REGISTRY_MATERIAL_PASS_TYPES: [ShaderPassType; 6] = [
    ShaderPassType::Forward,
    ShaderPassType::GBuffer,
    ShaderPassType::DepthPrepass,
    ShaderPassType::Shadow,
    ShaderPassType::Velocity,
    ShaderPassType::TaaReactiveMask,
];

pub(super) fn registry_material_pass_product_prewarm_manifest(
    cases: &[RegistryShaderCase],
) -> ShaderVariantPrewarmManifest {
    let standard_material_variants =
        builtin_standard_material_shader_prewarm_manifest_for_geometry(
            ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
            SHADING_MODEL_ID_STANDARD_PBR,
            None,
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            &[ShaderQualityTier::Medium],
        )
        .variants;
    let mut variants = builtin_fallback_shader_prewarm_manifest().variants;
    for case in cases.iter().copied() {
        variants.extend(
            standard_material_variants
                .iter()
                .cloned()
                .map(|mut request| {
                    request.key.material_shader = case.shader_id();
                    request.key.material_revision = case.revision;
                    request.source_label = case.source_label_for_pass(request.key.pass_type);
                    request
                }),
        );
    }
    ShaderVariantPrewarmManifest::new(variants)
}

pub(super) fn registry_material_pass_live_source_label_prewarm_manifest(
    cases: &[RegistryShaderCase],
) -> ShaderVariantPrewarmManifest {
    let mut manifest = registry_material_pass_product_prewarm_manifest(cases);
    for request in &mut manifest.variants {
        if let Some(case) = cases
            .iter()
            .copied()
            .find(|case| request_belongs_to_case(request, *case))
        {
            request.source_label = case.locator.to_string();
        }
    }
    manifest
}

pub(super) fn registry_material_pass_product_prewarm_manifest_with_plugin_shading_models(
    asset_manager: &ProjectAssetManager,
    cases: &[RegistryShaderCase],
    plugin_shading_models: &[ShadingModelDescriptor],
) -> Result<ShaderVariantPrewarmManifest, ShaderTemplateAssemblyError> {
    let custom_shading_model = plugin_shading_models
        .first()
        .expect("registry material-pass custom shading model");
    let custom_material_variants =
        builtin_standard_material_shader_prewarm_manifest_for_geometry_with_plugin_shading_models(
            asset_manager,
            ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
            custom_shading_model.id,
            None,
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            &[ShaderQualityTier::Medium],
            plugin_shading_models,
        )?
        .variants;
    let mut variants = builtin_fallback_shader_prewarm_manifest().variants;
    for case in cases.iter().copied() {
        variants.extend(custom_material_variants.iter().cloned().map(|mut request| {
            request.key.material_shader = case.shader_id();
            request.key.material_revision = case.revision;
            request.source_label = case.source_label_for_pass(request.key.pass_type);
            request
        }));
    }
    Ok(ShaderVariantPrewarmManifest::new(variants))
}

pub(super) fn raw_wgsl_hash(source: &str) -> String {
    blake3::hash(source.as_bytes()).to_hex().to_string()
}

pub(super) fn registry_material_pass_runtime_surface_source() -> String {
    super::super::registry_staged_cache_runtime_surface_source()
}

fn request_belongs_to_case(
    request: &ShaderVariantPrewarmRequest,
    case: RegistryShaderCase,
) -> bool {
    request.key.material_shader == case.shader_id()
        && request.key.material_revision == case.revision
}

#[cfg(test)]
mod tests {
    use super::registry_material_pass_runtime_surface_source;

    #[test]
    fn registry_material_pass_runtime_surface_source_uses_surface_entry_contract() {
        let source = registry_material_pass_runtime_surface_source();

        assert!(source.contains("fn zr_material_surface("));
        assert!(!source.contains("fn standard_material_surface("));
        assert!(!source.contains("@fragment"));
        assert!(source.contains("standard_material_properties"));
        assert!(source.contains("ZR_STANDARD_MATERIAL_ALPHA_CUTOFF"));
    }
}
