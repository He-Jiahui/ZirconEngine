use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

#[cfg(test)]
use crate::asset::ProjectAssetManager;
use crate::asset::ProjectAssetManagerAccess;
use crate::core::framework::render::{
    RenderMaterialPropertyUniformPayload, ShadingModelDescriptor,
};
use crate::graphics::material::{
    builtin_shading_model_registry, shading_model_registry_with_plugin_descriptors,
    ShadingModelRegistry,
};
use crate::graphics::scene::scene_renderer::mip_gen::RuntimeMipGenPass;
use crate::graphics::GraphicsError;
use crate::plugin::ShaderModuleSourceBinding;

use super::super::fallback::{create_fallback_normal_texture, create_fallback_texture};
use super::super::{
    GpuMaterialUniformResource, OutputTargetWritebackConverter, TextureSamplerCache,
};
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn new(
        asset_manager: ProjectAssetManagerAccess,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self::new_with_shading_model_registry(
            asset_manager,
            device,
            queue,
            texture_layout,
            builtin_shading_model_registry(),
            BTreeMap::new(),
        )
    }

    pub(crate) fn new_with_plugin_shading_models(
        asset_manager: ProjectAssetManagerAccess,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
    ) -> Result<Self, GraphicsError> {
        let shading_model_registry =
            shading_model_registry_with_plugin_descriptors(plugin_shading_models).map_err(
                |error| GraphicsError::Asset(format!("shading model registration failed: {error}")),
            )?;
        Ok(Self::new_with_shading_model_registry(
            asset_manager,
            device,
            queue,
            texture_layout,
            shading_model_registry,
            BTreeMap::new(),
        ))
    }

    pub(crate) fn new_with_plugin_shading_models_and_shader_modules(
        asset_manager: ProjectAssetManagerAccess,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
        plugin_shader_module_sources: impl IntoIterator<Item = ShaderModuleSourceBinding>,
    ) -> Result<Self, GraphicsError> {
        let shading_model_registry =
            shading_model_registry_with_plugin_descriptors(plugin_shading_models).map_err(
                |error| GraphicsError::Asset(format!("shading model registration failed: {error}")),
            )?;
        let shader_module_sources = shader_module_source_map(plugin_shader_module_sources)?;
        Ok(Self::new_with_shading_model_registry(
            asset_manager,
            device,
            queue,
            texture_layout,
            shading_model_registry,
            shader_module_sources,
        ))
    }

    fn new_with_shading_model_registry(
        asset_manager: ProjectAssetManagerAccess,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        shading_model_registry: ShadingModelRegistry,
        shader_module_sources: BTreeMap<String, ShaderModuleSourceBinding>,
    ) -> Self {
        let texture_sampler_cache = Arc::new(TextureSamplerCache::new());
        let fallback_texture = Arc::new(create_fallback_texture(
            device,
            queue,
            texture_layout,
            Arc::clone(&texture_sampler_cache),
        ));
        let fallback_normal_texture = Arc::new(create_fallback_normal_texture(
            device,
            queue,
            texture_layout,
            Arc::clone(&texture_sampler_cache),
        ));
        Self {
            asset_manager_access: asset_manager,
            shading_model_registry,
            shader_module_sources,
            models: HashMap::new(),
            meshes: HashMap::new(),
            materials: HashMap::new(),
            textures: HashMap::new(),
            mip_streaming_states: HashMap::new(),
            mip_streaming_visible_instance_keys: HashSet::new(),
            mip_streaming_visibility: Vec::new(),
            mip_streaming_residency_budget_bytes: u64::MAX,
            output_target_textures: HashMap::new(),
            post_process_lut_textures: HashMap::new(),
            shaders: HashMap::new(),
            texture_sampler_cache,
            fallback_texture,
            fallback_normal_texture,
            fallback_material_uniform: Arc::new(GpuMaterialUniformResource::from_payload(
                device,
                &RenderMaterialPropertyUniformPayload::default(),
            )),
            fallback_standard_material_uniform: Arc::new(
                GpuMaterialUniformResource::fallback_standard_material(device),
            ),
            runtime_mip_gen_pass: RuntimeMipGenPass::new(device),
            output_target_writeback_converter: OutputTargetWritebackConverter::new(device),
            last_material_count: 0,
            last_material_ready_count: 0,
            last_material_fallback_count: 0,
            last_material_validation_error_count: 0,
            last_material_diagnostic_count: 0,
            last_sprite_count: 0,
            last_sprite_ready_count: 0,
            last_sprite_texture_fallback_count: 0,
            last_post_process_lut_request_count: 0,
            last_post_process_lut_ready_count: 0,
            last_post_process_lut_fallback_count: 0,
            last_post_process_lut_2d_strip_ready_count: 0,
            last_post_process_lut_3d_request_count: 0,
            last_post_process_lut_unsupported_shape_count: 0,
            last_output_target_graph_import_report: Default::default(),
            last_output_target_writeback_report: Default::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self::new(
            ProjectAssetManagerAccess::for_test(asset_manager),
            device,
            queue,
            texture_layout,
        )
    }

    #[cfg(test)]
    pub(crate) fn new_for_test_with_plugin_shading_models(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_shading_models(
            ProjectAssetManagerAccess::for_test(asset_manager),
            device,
            queue,
            texture_layout,
            plugin_shading_models,
        )
    }

    #[cfg(test)]
    pub(crate) fn new_for_test_with_plugin_shader_module_sources(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        plugin_shader_module_sources: impl IntoIterator<Item = ShaderModuleSourceBinding>,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_shading_models_and_shader_modules(
            ProjectAssetManagerAccess::for_test(asset_manager),
            device,
            queue,
            texture_layout,
            Vec::new(),
            plugin_shader_module_sources,
        )
    }
}

fn shader_module_source_map(
    sources: impl IntoIterator<Item = ShaderModuleSourceBinding>,
) -> Result<BTreeMap<String, ShaderModuleSourceBinding>, GraphicsError> {
    let mut sources_by_import_path: BTreeMap<String, ShaderModuleSourceBinding> = BTreeMap::new();
    for source in sources {
        let actual_content_hash = blake3::hash(source.source.as_bytes()).to_hex().to_string();
        if source.content_hash != actual_content_hash {
            return Err(GraphicsError::Asset(format!(
                "shader module {} content hash does not match its source body",
                source.diagnostic_origin
            )));
        }
        if let Some(existing) = sources_by_import_path.get(&source.import_path) {
            return Err(GraphicsError::Asset(format!(
                "shader module `{}` is declared by both {} and {}",
                source.import_path, existing.diagnostic_origin, source.diagnostic_origin
            )));
        }
        sources_by_import_path.insert(source.import_path.clone(), source);
    }
    Ok(sources_by_import_path)
}

#[cfg(test)]
mod tests {
    use super::shader_module_source_map;
    use crate::plugin::ShaderModuleSourceBinding;

    #[test]
    fn shader_module_source_map_reports_same_token_from_distinct_owners() {
        let project = ShaderModuleSourceBinding::new(
            "package:one",
            "zircon_fixture::lighting",
            "fn fixture_lighting() -> vec3f { return vec3f(0.5); }",
            "fixture package one",
        );
        let duplicate = ShaderModuleSourceBinding::new(
            "package:two",
            "zircon_fixture::lighting",
            project.source.clone(),
            "fixture package two",
        );

        let error = shader_module_source_map([project, duplicate])
            .expect_err("distinct shader-module owners must remain diagnosable");
        assert!(error.to_string().contains("fixture package one"));
        assert!(error.to_string().contains("fixture package two"));
    }
}
