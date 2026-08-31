use std::collections::BTreeMap;

use super::super::compiler::{
    ShaderDispatchBuildDiagnostic, ShaderParameterValue, ShaderResourceBindingRequest,
    validate_named_resource_bindings, validate_shader_entry_point,
};
use super::super::{
    RenderShaderEntryPointDescriptor, RenderShaderStage, ShaderAssetKind, ShaderResourceAccess,
    ShaderResourceDescriptor, ShaderResourceKind,
};
use super::abi::{
    FULLSCREEN_FIRST_PASS_INPUT_BINDING, FULLSCREEN_PASS_INPUT_GROUP,
    FULLSCREEN_TRIANGLE_VERTEX_ENTRY,
};
use super::pipeline_cache_key::FullscreenPipelineCacheKey;
use super::plan::FullscreenPassPlan;
use super::shader_ref::FullscreenShaderRef;

#[derive(Clone, Debug, PartialEq)]
pub struct FullscreenPassBuilder {
    shader: FullscreenShaderRef,
    parameters: BTreeMap<String, ShaderParameterValue>,
    resource_bindings: BTreeMap<String, ShaderResourceBindingRequest>,
    option_bits: u32,
    content_hash: u64,
    pipeline_label: Option<String>,
}

impl FullscreenPassBuilder {
    pub fn new(shader: FullscreenShaderRef) -> Self {
        Self {
            shader,
            parameters: BTreeMap::new(),
            resource_bindings: BTreeMap::new(),
            option_bits: 0,
            content_hash: 0,
            pipeline_label: None,
        }
    }

    pub fn with_option_bits(mut self, option_bits: u32) -> Self {
        self.option_bits = option_bits;
        self
    }

    pub fn with_content_hash(mut self, content_hash: u64) -> Self {
        self.content_hash = content_hash;
        self
    }

    pub fn with_pipeline_label(mut self, pipeline_label: impl Into<String>) -> Self {
        self.pipeline_label = Some(pipeline_label.into());
        self
    }

    pub fn set_bool(self, name: impl Into<String>, value: bool) -> Self {
        self.set_parameter(name, ShaderParameterValue::Bool { value })
    }

    pub fn set_f32(self, name: impl Into<String>, value: f32) -> Self {
        self.set_parameter(name, ShaderParameterValue::F32 { value })
    }

    pub fn set_i32(self, name: impl Into<String>, value: i32) -> Self {
        self.set_parameter(name, ShaderParameterValue::I32 { value })
    }

    pub fn set_u32(self, name: impl Into<String>, value: u32) -> Self {
        self.set_parameter(name, ShaderParameterValue::U32 { value })
    }

    pub fn set_vec2(self, name: impl Into<String>, value: [f32; 2]) -> Self {
        self.set_parameter(name, ShaderParameterValue::Vec2 { value })
    }

    pub fn set_vec3(self, name: impl Into<String>, value: [f32; 3]) -> Self {
        self.set_parameter(name, ShaderParameterValue::Vec3 { value })
    }

    pub fn set_vec4(self, name: impl Into<String>, value: [f32; 4]) -> Self {
        self.set_parameter(name, ShaderParameterValue::Vec4 { value })
    }

    pub fn bind_texture(self, name: impl Into<String>) -> Self {
        self.bind_resource(
            name,
            ShaderResourceKind::Texture,
            ShaderResourceAccess::Read,
        )
    }

    pub fn bind_sampler(self, name: impl Into<String>) -> Self {
        self.bind_resource(
            name,
            ShaderResourceKind::Sampler,
            ShaderResourceAccess::Read,
        )
    }

    pub fn bind_uniform(self, name: impl Into<String>) -> Self {
        self.bind_resource(
            name,
            ShaderResourceKind::UniformBuffer,
            ShaderResourceAccess::Read,
        )
    }

    pub fn bind_storage_read(self, name: impl Into<String>) -> Self {
        self.bind_resource(
            name,
            ShaderResourceKind::StorageBuffer,
            ShaderResourceAccess::Read,
        )
    }

    pub fn build(
        self,
        shader_kind: ShaderAssetKind,
        entry_points: &[RenderShaderEntryPointDescriptor],
        declared_resources: &[ShaderResourceDescriptor],
    ) -> Result<FullscreenPassPlan, Vec<ShaderDispatchBuildDiagnostic>> {
        let mut diagnostics = Vec::new();
        validate_shader_entry_point(
            &mut diagnostics,
            shader_kind,
            ShaderAssetKind::Fullscreen,
            entry_points,
            &self.shader.fragment_entry,
            RenderShaderStage::Fragment,
        );
        let resources = validate_named_resource_bindings(
            &mut diagnostics,
            &self.resource_bindings,
            declared_resources,
            FULLSCREEN_PASS_INPUT_GROUP,
            FULLSCREEN_FIRST_PASS_INPUT_BINDING,
        );

        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        let pipeline_key = FullscreenPipelineCacheKey {
            shader: self.shader.shader.clone(),
            fragment_entry: self.shader.fragment_entry.clone(),
            option_bits: self.option_bits,
            content_hash: self.content_hash,
        };
        let pipeline_label = self
            .pipeline_label
            .unwrap_or_else(|| pipeline_key.canonical_string());

        Ok(FullscreenPassPlan {
            shader: self.shader,
            vertex_entry: FULLSCREEN_TRIANGLE_VERTEX_ENTRY.to_string(),
            parameters: self.parameters,
            resources,
            pipeline_key,
            pipeline_label,
        })
    }

    fn set_parameter(mut self, name: impl Into<String>, value: ShaderParameterValue) -> Self {
        self.parameters.insert(name.into(), value);
        self
    }

    fn bind_resource(
        mut self,
        name: impl Into<String>,
        kind: ShaderResourceKind,
        access: ShaderResourceAccess,
    ) -> Self {
        let name = name.into();
        self.resource_bindings.insert(
            name.clone(),
            ShaderResourceBindingRequest { name, kind, access },
        );
        self
    }
}
