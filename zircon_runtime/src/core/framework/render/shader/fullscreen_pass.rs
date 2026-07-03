use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::resource::{AssetReference, ResourceLocator, ResourceLocatorError};

use super::compute_dispatch::{
    validate_named_resource_bindings, validate_shader_entry_point, ShaderAbiBinding,
    ShaderDispatchBuildDiagnostic, ShaderNamedResourceBinding, ShaderParameterValue,
    ShaderResourceBindingRequest,
};
use super::{
    RenderShaderEntryPointDescriptor, RenderShaderStage, ShaderAssetKind, ShaderResourceAccess,
    ShaderResourceDescriptor, ShaderResourceKind,
};

pub const FULLSCREEN_FRAME_GROUP: u32 = 0;
pub const FULLSCREEN_PASS_INPUT_GROUP: u32 = 1;
pub const FULLSCREEN_PARAMS_BINDING: ShaderAbiBinding = ShaderAbiBinding {
    group: 2,
    binding: 0,
};
pub const FULLSCREEN_FIRST_PASS_INPUT_BINDING: u32 = 0;
pub const FULLSCREEN_TRIANGLE_VERTEX_ENTRY: &str = "zr_fullscreen_triangle_vs";

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FullscreenShaderRef {
    pub shader: AssetReference,
    pub fragment_entry: String,
}

impl FullscreenShaderRef {
    pub fn new(shader: AssetReference, fragment_entry: impl Into<String>) -> Self {
        Self {
            shader,
            fragment_entry: fragment_entry.into(),
        }
    }

    pub fn from_locator_str(
        shader: &str,
        fragment_entry: impl Into<String>,
    ) -> Result<Self, ResourceLocatorError> {
        Ok(Self::new(
            AssetReference::from_locator(ResourceLocator::parse(shader)?),
            fragment_entry,
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FullscreenPipelineCacheKey {
    pub shader: AssetReference,
    pub fragment_entry: String,
    pub option_bits: u32,
    pub content_hash: u64,
}

impl FullscreenPipelineCacheKey {
    pub fn canonical_string(&self) -> String {
        format!(
            "shader_fullscreen_pipeline_v1|shader={}|fragment={}|options={:#010x}|content={:#018x}",
            self.shader, self.fragment_entry, self.option_bits, self.content_hash
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FullscreenPassPlan {
    pub shader: FullscreenShaderRef,
    pub vertex_entry: String,
    pub parameters: BTreeMap<String, ShaderParameterValue>,
    pub resources: Vec<ShaderNamedResourceBinding>,
    pub pipeline_key: FullscreenPipelineCacheKey,
    pub pipeline_label: String,
}

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

    pub fn with_option_bits(&mut self, option_bits: u32) -> &mut Self {
        self.option_bits = option_bits;
        self
    }

    pub fn with_content_hash(&mut self, content_hash: u64) -> &mut Self {
        self.content_hash = content_hash;
        self
    }

    pub fn with_pipeline_label(&mut self, pipeline_label: impl Into<String>) -> &mut Self {
        self.pipeline_label = Some(pipeline_label.into());
        self
    }

    pub fn set_bool(&mut self, name: impl Into<String>, value: bool) -> &mut Self {
        self.set_parameter(name, ShaderParameterValue::Bool { value })
    }

    pub fn set_f32(&mut self, name: impl Into<String>, value: f32) -> &mut Self {
        self.set_parameter(name, ShaderParameterValue::F32 { value })
    }

    pub fn set_vec4(&mut self, name: impl Into<String>, value: [f32; 4]) -> &mut Self {
        self.set_parameter(name, ShaderParameterValue::Vec4 { value })
    }

    pub fn bind_texture(&mut self, name: impl Into<String>) -> &mut Self {
        self.bind_resource(
            name,
            ShaderResourceKind::Texture,
            ShaderResourceAccess::Read,
        )
    }

    pub fn bind_sampler(&mut self, name: impl Into<String>) -> &mut Self {
        self.bind_resource(
            name,
            ShaderResourceKind::Sampler,
            ShaderResourceAccess::Read,
        )
    }

    pub fn bind_uniform(&mut self, name: impl Into<String>) -> &mut Self {
        self.bind_resource(
            name,
            ShaderResourceKind::UniformBuffer,
            ShaderResourceAccess::Read,
        )
    }

    pub fn bind_storage_read(&mut self, name: impl Into<String>) -> &mut Self {
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

    fn set_parameter(&mut self, name: impl Into<String>, value: ShaderParameterValue) -> &mut Self {
        self.parameters.insert(name.into(), value);
        self
    }

    fn bind_resource(
        &mut self,
        name: impl Into<String>,
        kind: ShaderResourceKind,
        access: ShaderResourceAccess,
    ) -> &mut Self {
        let name = name.into();
        self.resource_bindings.insert(
            name.clone(),
            ShaderResourceBindingRequest { name, kind, access },
        );
        self
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::resource::{AssetReference, ResourceLocator};

    use super::*;

    fn shader_ref() -> AssetReference {
        AssetReference::from_locator(
            ResourceLocator::parse("builtin://shaders/fullscreen/tonemap").unwrap(),
        )
    }

    fn entry(name: &str, stage: RenderShaderStage) -> RenderShaderEntryPointDescriptor {
        RenderShaderEntryPointDescriptor {
            name: name.to_string(),
            stage,
        }
    }

    fn resource(
        name: &str,
        kind: ShaderResourceKind,
        access: ShaderResourceAccess,
    ) -> ShaderResourceDescriptor {
        ShaderResourceDescriptor {
            name: name.to_string(),
            kind,
            access: Some(access),
        }
    }

    #[test]
    fn render_fullscreen_pass_builder_emits_pass_input_and_params_abi() {
        let shader = FullscreenShaderRef::new(shader_ref(), "fs_main");
        let mut builder = FullscreenPassBuilder::new(shader.clone());
        builder
            .with_option_bits(0x8)
            .with_content_hash(0xf00d)
            .set_f32("exposure", 1.25)
            .bind_texture("source_color")
            .bind_sampler("linear_sampler");

        let plan = builder
            .build(
                ShaderAssetKind::Fullscreen,
                &[entry("fs_main", RenderShaderStage::Fragment)],
                &[
                    resource(
                        "source_color",
                        ShaderResourceKind::Texture,
                        ShaderResourceAccess::Read,
                    ),
                    resource(
                        "linear_sampler",
                        ShaderResourceKind::Sampler,
                        ShaderResourceAccess::Read,
                    ),
                ],
            )
            .unwrap();

        assert_eq!(plan.shader, shader);
        assert_eq!(plan.vertex_entry, FULLSCREEN_TRIANGLE_VERTEX_ENTRY);
        assert_eq!(
            plan.parameters.get("exposure"),
            Some(&ShaderParameterValue::F32 { value: 1.25 })
        );
        assert_eq!(FULLSCREEN_FRAME_GROUP, 0);
        assert_eq!(FULLSCREEN_PASS_INPUT_GROUP, 1);
        assert_eq!(
            FULLSCREEN_PARAMS_BINDING,
            ShaderAbiBinding {
                group: 2,
                binding: 0
            }
        );
        assert_eq!(
            plan.resources[0].abi,
            ShaderAbiBinding {
                group: 1,
                binding: 0
            }
        );
        assert_eq!(
            plan.resources[1].abi,
            ShaderAbiBinding {
                group: 1,
                binding: 1
            }
        );
        assert_eq!(
            plan.pipeline_key.canonical_string(),
            format!(
                "shader_fullscreen_pipeline_v1|shader={}|fragment=fs_main|options=0x00000008|content=0x000000000000f00d",
                shader_ref()
            )
        );
    }

    #[test]
    fn render_fullscreen_pass_builder_reports_stage_and_resource_errors() {
        let mut builder =
            FullscreenPassBuilder::new(FullscreenShaderRef::new(shader_ref(), "fs_main"));
        builder.bind_storage_read("source_color");

        let diagnostics = builder
            .build(
                ShaderAssetKind::Compute,
                &[entry("fs_main", RenderShaderStage::Compute)],
                &[resource(
                    "source_color",
                    ShaderResourceKind::Texture,
                    ShaderResourceAccess::Read,
                )],
            )
            .unwrap_err();

        assert!(
            diagnostics.contains(&ShaderDispatchBuildDiagnostic::InvalidShaderKind {
                expected: ShaderAssetKind::Fullscreen,
                actual: ShaderAssetKind::Compute,
            })
        );
        assert!(
            diagnostics.contains(&ShaderDispatchBuildDiagnostic::InvalidEntryPointStage {
                entry_point: "fs_main".to_string(),
                stage: RenderShaderStage::Compute,
                expected_stage: RenderShaderStage::Fragment,
            })
        );
        assert!(
            diagnostics.contains(&ShaderDispatchBuildDiagnostic::ResourceKindMismatch {
                name: "source_color".to_string(),
                expected: ShaderResourceKind::Texture,
                actual: ShaderResourceKind::StorageBuffer,
            })
        );
    }
}
