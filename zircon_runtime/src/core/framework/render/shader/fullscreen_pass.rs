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

impl FullscreenPassPlan {
    pub fn resource_binding(&self, name: &str) -> Option<&ShaderNamedResourceBinding> {
        self.resources.iter().find(|resource| resource.name == name)
    }

    pub fn parameter_slot(&self, name: &str) -> Option<u32> {
        self.parameters
            .keys()
            .position(|parameter_name| parameter_name == name)
            .and_then(|slot| u32::try_from(slot).ok())
    }

    pub fn parameter_byte_len(&self) -> u64 {
        u64::try_from(self.parameters.len())
            .unwrap_or(u64::MAX / 16)
            .saturating_mul(16)
    }

    pub fn parameter_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.parameters.len().saturating_mul(16));
        self.write_parameter_bytes(&mut bytes);
        bytes
    }

    pub(crate) fn write_parameter_bytes(&self, bytes: &mut Vec<u8>) {
        let byte_len = self.parameters.len().saturating_mul(16);
        bytes.clear();
        if bytes.capacity() < byte_len {
            bytes.reserve_exact(byte_len);
        }
        for value in self.parameters.values() {
            for word in fullscreen_parameter_words(value) {
                bytes.extend_from_slice(&word.to_ne_bytes());
            }
        }
    }
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

fn fullscreen_parameter_words(value: &ShaderParameterValue) -> [u32; 4] {
    // A fixed vec4-sized slot keeps generated fullscreen parameter ABI stable.
    match value {
        ShaderParameterValue::Bool { value } => [u32::from(*value), 0, 0, 0],
        ShaderParameterValue::F32 { value } => [value.to_bits(), 0, 0, 0],
        ShaderParameterValue::I32 { value } => [*value as u32, 0, 0, 0],
        ShaderParameterValue::U32 { value } => [*value, 0, 0, 0],
        ShaderParameterValue::Vec2 { value } => [value[0].to_bits(), value[1].to_bits(), 0, 0],
        ShaderParameterValue::Vec3 { value } => [
            value[0].to_bits(),
            value[1].to_bits(),
            value[2].to_bits(),
            0,
        ],
        ShaderParameterValue::Vec4 { value } => [
            value[0].to_bits(),
            value[1].to_bits(),
            value[2].to_bits(),
            value[3].to_bits(),
        ],
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
        let builder = FullscreenPassBuilder::new(shader.clone())
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
    fn render_fullscreen_pass_parameters_use_stable_vec4_slots() {
        let plan = FullscreenPassBuilder::new(FullscreenShaderRef::new(shader_ref(), "fs_main"))
            .set_vec4("tint", [0.25, 0.5, 0.75, 1.0])
            .set_bool("enabled", true)
            .set_f32("exposure", 1.25)
            .build(
                ShaderAssetKind::Fullscreen,
                &[entry("fs_main", RenderShaderStage::Fragment)],
                &[],
            )
            .expect("fullscreen parameter-only plan should build");

        assert_eq!(plan.parameter_slot("enabled"), Some(0));
        assert_eq!(plan.parameter_slot("exposure"), Some(1));
        assert_eq!(plan.parameter_slot("tint"), Some(2));
        assert_eq!(plan.parameter_byte_len(), 48);
        assert_eq!(
            plan.parameter_bytes(),
            [
                1_u32,
                0,
                0,
                0,
                1.25_f32.to_bits(),
                0,
                0,
                0,
                0.25_f32.to_bits(),
                0.5_f32.to_bits(),
                0.75_f32.to_bits(),
                1.0_f32.to_bits(),
            ]
            .into_iter()
            .flat_map(u32::to_ne_bytes)
            .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn render_fullscreen_pass_reencodes_parameters_into_a_reused_buffer() {
        let plan = FullscreenPassBuilder::new(FullscreenShaderRef::new(shader_ref(), "fs_main"))
            .set_f32("exposure", 1.25)
            .set_vec4("tint", [0.25, 0.5, 0.75, 1.0])
            .build(
                ShaderAssetKind::Fullscreen,
                &[entry("fs_main", RenderShaderStage::Fragment)],
                &[],
            )
            .expect("fullscreen parameter-only plan should build");
        let mut bytes = Vec::with_capacity(64);
        bytes.extend_from_slice(&[0xff; 8]);
        let capacity = bytes.capacity();

        plan.write_parameter_bytes(&mut bytes);

        assert_eq!(bytes.capacity(), capacity);
        assert_eq!(bytes.len(), 32);
        assert_eq!(
            bytes,
            [
                1.25_f32.to_bits(),
                0,
                0,
                0,
                0.25_f32.to_bits(),
                0.5_f32.to_bits(),
                0.75_f32.to_bits(),
                1.0_f32.to_bits(),
            ]
            .into_iter()
            .flat_map(u32::to_ne_bytes)
            .collect::<Vec<_>>(),
        );

        plan.write_parameter_bytes(&mut bytes);
        assert_eq!(bytes.capacity(), capacity);
    }

    #[test]
    fn render_fullscreen_pass_builder_encodes_every_parameter_value_shape() {
        let plan = FullscreenPassBuilder::new(FullscreenShaderRef::new(shader_ref(), "fs_main"))
            .set_i32("signed", -2)
            .set_u32("unsigned", 7)
            .set_vec2("uv", [0.25, 0.5])
            .set_vec3("normal", [0.0, 1.0, 0.0])
            .build(
                ShaderAssetKind::Fullscreen,
                &[entry("fs_main", RenderShaderStage::Fragment)],
                &[],
            )
            .expect("all fullscreen parameter value shapes should build");

        assert_eq!(plan.parameter_slot("normal"), Some(0));
        assert_eq!(plan.parameter_slot("signed"), Some(1));
        assert_eq!(plan.parameter_slot("unsigned"), Some(2));
        assert_eq!(plan.parameter_slot("uv"), Some(3));
        assert_eq!(
            plan.parameter_bytes(),
            [
                0.0_f32.to_bits(),
                1.0_f32.to_bits(),
                0.0_f32.to_bits(),
                0,
                (-2_i32) as u32,
                0,
                0,
                0,
                7,
                0,
                0,
                0,
                0.25_f32.to_bits(),
                0.5_f32.to_bits(),
                0,
                0,
            ]
            .into_iter()
            .flat_map(u32::to_ne_bytes)
            .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn render_fullscreen_pass_builder_reports_stage_and_resource_errors() {
        let builder = FullscreenPassBuilder::new(FullscreenShaderRef::new(shader_ref(), "fs_main"))
            .bind_storage_read("source_color");

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
