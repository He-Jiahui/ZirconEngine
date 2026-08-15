use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::resource::{AssetReference, ResourceLocator, ResourceLocatorError};

use super::{
    RenderShaderEntryPointDescriptor, RenderShaderStage, ShaderAssetKind, ShaderResourceAccess,
    ShaderResourceDescriptor, ShaderResourceKind,
};

pub const COMPUTE_SHADER_PARAMS_BINDING: ShaderAbiBinding = ShaderAbiBinding {
    group: 0,
    binding: 0,
};
pub const COMPUTE_SHADER_RESOURCE_GROUP: u32 = 0;
pub const COMPUTE_SHADER_FIRST_RESOURCE_BINDING: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShaderAbiBinding {
    pub group: u32,
    pub binding: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComputeKernelRef {
    pub shader: AssetReference,
    pub kernel: String,
}

impl ComputeKernelRef {
    pub fn new(shader: AssetReference, kernel: impl Into<String>) -> Self {
        Self {
            shader,
            kernel: kernel.into(),
        }
    }

    pub fn from_locator_str(
        shader: &str,
        kernel: impl Into<String>,
    ) -> Result<Self, ResourceLocatorError> {
        Ok(Self::new(
            AssetReference::from_locator(ResourceLocator::parse(shader)?),
            kernel,
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ShaderParameterValue {
    Bool { value: bool },
    F32 { value: f32 },
    I32 { value: i32 },
    U32 { value: u32 },
    Vec2 { value: [f32; 2] },
    Vec3 { value: [f32; 3] },
    Vec4 { value: [f32; 4] },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderDispatchExtent {
    ClusterGrid,
    HzbFurthest,
    IndirectArgs,
    Fixed([u32; 3]),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComputePipelineCacheKey {
    pub shader: AssetReference,
    pub kernel: String,
    pub option_bits: u32,
    pub content_hash: u64,
}

impl ComputePipelineCacheKey {
    pub fn canonical_string(&self) -> String {
        format!(
            "shader_compute_pipeline_v1|shader={}|kernel={}|options={:#010x}|content={:#018x}",
            self.shader, self.kernel, self.option_bits, self.content_hash
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderNamedResourceBinding {
    pub name: String,
    pub kind: ShaderResourceKind,
    pub access: ShaderResourceAccess,
    pub abi: ShaderAbiBinding,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComputeDispatchPlan {
    pub kernel: ComputeKernelRef,
    pub parameters: BTreeMap<String, ShaderParameterValue>,
    pub resources: Vec<ShaderNamedResourceBinding>,
    pub dispatch_extent: ShaderDispatchExtent,
    pub workgroup_size: [u32; 3],
    pub pipeline_key: ComputePipelineCacheKey,
    pub pipeline_label: String,
}

impl ComputeDispatchPlan {
    pub fn resource_binding(&self, name: &str) -> Option<&ShaderNamedResourceBinding> {
        self.resources.iter().find(|resource| resource.name == name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShaderDispatchBuildDiagnostic {
    InvalidShaderKind {
        expected: ShaderAssetKind,
        actual: ShaderAssetKind,
    },
    MissingEntryPoint {
        entry_point: String,
        expected_stage: RenderShaderStage,
    },
    InvalidEntryPointStage {
        entry_point: String,
        stage: RenderShaderStage,
        expected_stage: RenderShaderStage,
    },
    MissingDispatchGroups {
        kernel: String,
    },
    MissingResource {
        name: String,
    },
    UnknownResource {
        name: String,
    },
    ResourceKindMismatch {
        name: String,
        expected: ShaderResourceKind,
        actual: ShaderResourceKind,
    },
    ResourceAccessMismatch {
        name: String,
        expected: ShaderResourceAccess,
        actual: ShaderResourceAccess,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComputeDispatchBuilder {
    kernel: ComputeKernelRef,
    parameters: BTreeMap<String, ShaderParameterValue>,
    resource_bindings: BTreeMap<String, ShaderResourceBindingRequest>,
    dispatch_extent: Option<ShaderDispatchExtent>,
    workgroup_size: [u32; 3],
    option_bits: u32,
    content_hash: u64,
    pipeline_label: Option<String>,
}

impl ComputeDispatchBuilder {
    pub fn new(kernel: ComputeKernelRef) -> Self {
        Self {
            kernel,
            parameters: BTreeMap::new(),
            resource_bindings: BTreeMap::new(),
            dispatch_extent: None,
            workgroup_size: [1, 1, 1],
            option_bits: 0,
            content_hash: 0,
            pipeline_label: None,
        }
    }

    pub fn with_workgroup_size(mut self, workgroup_size: [u32; 3]) -> Self {
        self.workgroup_size = normalize_workgroup_size(workgroup_size);
        self
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

    pub fn bind_uniform(self, name: impl Into<String>) -> Self {
        self.bind_resource(
            name,
            ShaderResourceKind::UniformBuffer,
            ShaderResourceAccess::Read,
        )
    }

    pub fn bind_storage(self, name: impl Into<String>) -> Self {
        self.bind_storage_read_write(name)
    }

    pub fn bind_storage_read(self, name: impl Into<String>) -> Self {
        self.bind_resource(
            name,
            ShaderResourceKind::StorageBuffer,
            ShaderResourceAccess::Read,
        )
    }

    pub fn bind_storage_write(self, name: impl Into<String>) -> Self {
        self.bind_resource(
            name,
            ShaderResourceKind::StorageBuffer,
            ShaderResourceAccess::Write,
        )
    }

    pub fn bind_storage_read_write(self, name: impl Into<String>) -> Self {
        self.bind_resource(
            name,
            ShaderResourceKind::StorageBuffer,
            ShaderResourceAccess::ReadWrite,
        )
    }

    pub fn bind_texture(self, name: impl Into<String>) -> Self {
        self.bind_resource(
            name,
            ShaderResourceKind::Texture,
            ShaderResourceAccess::Read,
        )
    }

    pub fn bind_storage_texture_write(self, name: impl Into<String>) -> Self {
        self.bind_resource(
            name,
            ShaderResourceKind::StorageTexture,
            ShaderResourceAccess::Write,
        )
    }

    pub fn bind_sampler(self, name: impl Into<String>) -> Self {
        self.bind_resource(
            name,
            ShaderResourceKind::Sampler,
            ShaderResourceAccess::Read,
        )
    }

    pub fn dispatch_extent(mut self, extent: ShaderDispatchExtent) -> Self {
        self.dispatch_extent = Some(extent);
        self
    }

    pub fn dispatch_groups(self, groups: [u32; 3]) -> Self {
        self.dispatch_extent(ShaderDispatchExtent::Fixed(groups))
    }

    pub fn build(
        self,
        shader_kind: ShaderAssetKind,
        entry_points: &[RenderShaderEntryPointDescriptor],
        declared_resources: &[ShaderResourceDescriptor],
    ) -> Result<ComputeDispatchPlan, Vec<ShaderDispatchBuildDiagnostic>> {
        let mut diagnostics = Vec::new();
        validate_shader_entry_point(
            &mut diagnostics,
            shader_kind,
            ShaderAssetKind::Compute,
            entry_points,
            &self.kernel.kernel,
            RenderShaderStage::Compute,
        );
        let dispatch_extent = match self.dispatch_extent {
            Some(extent) => extent,
            None => {
                diagnostics.push(ShaderDispatchBuildDiagnostic::MissingDispatchGroups {
                    kernel: self.kernel.kernel.clone(),
                });
                ShaderDispatchExtent::Fixed([0, 0, 0])
            }
        };
        let resources = validate_named_resource_bindings(
            &mut diagnostics,
            &self.resource_bindings,
            declared_resources,
            COMPUTE_SHADER_RESOURCE_GROUP,
            COMPUTE_SHADER_FIRST_RESOURCE_BINDING,
        );

        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        let pipeline_key = ComputePipelineCacheKey {
            shader: self.kernel.shader.clone(),
            kernel: self.kernel.kernel.clone(),
            option_bits: self.option_bits,
            content_hash: self.content_hash,
        };
        let pipeline_label = self
            .pipeline_label
            .unwrap_or_else(|| pipeline_key.canonical_string());

        Ok(ComputeDispatchPlan {
            kernel: self.kernel,
            parameters: self.parameters,
            resources,
            dispatch_extent,
            workgroup_size: self.workgroup_size,
            pipeline_key,
            pipeline_label,
        })
    }

    fn set_parameter(mut self, name: impl Into<String>, value: ShaderParameterValue) -> Self {
        self.parameters.insert(name.into(), value);
        self
    }

    pub(super) fn bind_resource(
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ShaderResourceBindingRequest {
    pub name: String,
    pub kind: ShaderResourceKind,
    pub access: ShaderResourceAccess,
}

pub(super) fn validate_shader_entry_point(
    diagnostics: &mut Vec<ShaderDispatchBuildDiagnostic>,
    actual_kind: ShaderAssetKind,
    expected_kind: ShaderAssetKind,
    entry_points: &[RenderShaderEntryPointDescriptor],
    entry_point: &str,
    expected_stage: RenderShaderStage,
) {
    if actual_kind != expected_kind {
        diagnostics.push(ShaderDispatchBuildDiagnostic::InvalidShaderKind {
            expected: expected_kind,
            actual: actual_kind,
        });
    }

    let Some(entry) = entry_points.iter().find(|entry| entry.name == entry_point) else {
        diagnostics.push(ShaderDispatchBuildDiagnostic::MissingEntryPoint {
            entry_point: entry_point.to_string(),
            expected_stage,
        });
        return;
    };

    if entry.stage != expected_stage {
        diagnostics.push(ShaderDispatchBuildDiagnostic::InvalidEntryPointStage {
            entry_point: entry_point.to_string(),
            stage: entry.stage,
            expected_stage,
        });
    }
}

pub(super) fn validate_named_resource_bindings(
    diagnostics: &mut Vec<ShaderDispatchBuildDiagnostic>,
    requested_bindings: &BTreeMap<String, ShaderResourceBindingRequest>,
    declared_resources: &[ShaderResourceDescriptor],
    abi_group: u32,
    first_binding: u32,
) -> Vec<ShaderNamedResourceBinding> {
    let mut output = Vec::new();
    let mut declared_names = BTreeSet::new();

    for (index, declared) in declared_resources.iter().enumerate() {
        declared_names.insert(declared.name.as_str());
        let Some(requested) = requested_bindings.get(&declared.name) else {
            diagnostics.push(ShaderDispatchBuildDiagnostic::MissingResource {
                name: declared.name.clone(),
            });
            continue;
        };

        if requested.kind != declared.kind {
            diagnostics.push(ShaderDispatchBuildDiagnostic::ResourceKindMismatch {
                name: declared.name.clone(),
                expected: declared.kind,
                actual: requested.kind,
            });
            continue;
        }

        let expected_access = declared
            .access
            .unwrap_or_else(|| default_resource_access(declared.kind));
        if !resource_access_satisfies(requested.access, expected_access) {
            diagnostics.push(ShaderDispatchBuildDiagnostic::ResourceAccessMismatch {
                name: declared.name.clone(),
                expected: expected_access,
                actual: requested.access,
            });
            continue;
        }

        output.push(ShaderNamedResourceBinding {
            name: declared.name.clone(),
            kind: declared.kind,
            access: requested.access,
            abi: ShaderAbiBinding {
                group: abi_group,
                binding: first_binding + index as u32,
            },
        });
    }

    for requested in requested_bindings.values() {
        if !declared_names.contains(requested.name.as_str()) {
            diagnostics.push(ShaderDispatchBuildDiagnostic::UnknownResource {
                name: requested.name.clone(),
            });
        }
    }

    output
}

fn normalize_workgroup_size(workgroup_size: [u32; 3]) -> [u32; 3] {
    [
        workgroup_size[0].max(1),
        workgroup_size[1].max(1),
        workgroup_size[2].max(1),
    ]
}

fn default_resource_access(kind: ShaderResourceKind) -> ShaderResourceAccess {
    match kind {
        ShaderResourceKind::UniformBuffer
        | ShaderResourceKind::Texture
        | ShaderResourceKind::Sampler => ShaderResourceAccess::Read,
        ShaderResourceKind::StorageBuffer | ShaderResourceKind::StorageTexture => {
            ShaderResourceAccess::ReadWrite
        }
    }
}

fn resource_access_satisfies(actual: ShaderResourceAccess, expected: ShaderResourceAccess) -> bool {
    match expected {
        ShaderResourceAccess::Read => {
            matches!(
                actual,
                ShaderResourceAccess::Read | ShaderResourceAccess::ReadWrite
            )
        }
        ShaderResourceAccess::ReadWrite => matches!(actual, ShaderResourceAccess::ReadWrite),
        ShaderResourceAccess::Write => {
            matches!(
                actual,
                ShaderResourceAccess::Write | ShaderResourceAccess::ReadWrite
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::resource::{AssetReference, ResourceLocator};

    use super::*;

    fn shader_ref() -> AssetReference {
        AssetReference::from_locator(
            ResourceLocator::parse("builtin://shaders/compute/particles").unwrap(),
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
    fn render_compute_dispatch_builder_emits_kernel_resource_abi_and_cache_key() {
        let kernel = ComputeKernelRef::new(shader_ref(), "cs_main");
        let builder = ComputeDispatchBuilder::new(kernel.clone())
            .with_workgroup_size([64, 0, 1])
            .with_option_bits(0x3)
            .with_content_hash(0x55aa)
            .set_f32("delta_time", 1.0 / 60.0)
            .bind_storage("particles")
            .bind_storage_read("alive_list")
            .dispatch_groups([32, 1, 1]);

        let plan = builder
            .build(
                ShaderAssetKind::Compute,
                &[entry("cs_main", RenderShaderStage::Compute)],
                &[
                    resource(
                        "particles",
                        ShaderResourceKind::StorageBuffer,
                        ShaderResourceAccess::ReadWrite,
                    ),
                    resource(
                        "alive_list",
                        ShaderResourceKind::StorageBuffer,
                        ShaderResourceAccess::Read,
                    ),
                ],
            )
            .unwrap();

        assert_eq!(plan.kernel, kernel);
        assert_eq!(plan.workgroup_size, [64, 1, 1]);
        assert_eq!(
            plan.dispatch_extent,
            ShaderDispatchExtent::Fixed([32, 1, 1])
        );
        assert_eq!(
            plan.parameters.get("delta_time"),
            Some(&ShaderParameterValue::F32 { value: 1.0 / 60.0 })
        );
        assert_eq!(plan.resources.len(), 2);
        assert_eq!(plan.resources[0].name, "particles");
        assert_eq!(
            plan.resources[0].abi,
            ShaderAbiBinding {
                group: 0,
                binding: 1
            }
        );
        assert_eq!(plan.resources[1].name, "alive_list");
        assert_eq!(
            plan.resources[1].abi,
            ShaderAbiBinding {
                group: 0,
                binding: 2
            }
        );
        assert_eq!(COMPUTE_SHADER_PARAMS_BINDING.group, 0);
        assert_eq!(COMPUTE_SHADER_PARAMS_BINDING.binding, 0);
        assert_eq!(
            plan.pipeline_key.canonical_string(),
            format!(
                "shader_compute_pipeline_v1|shader={}|kernel=cs_main|options=0x00000003|content=0x00000000000055aa",
                shader_ref()
            )
        );
        assert_eq!(plan.pipeline_label, plan.pipeline_key.canonical_string());
    }

    #[test]
    fn render_compute_dispatch_builder_reports_named_binding_diagnostics() {
        let builder = ComputeDispatchBuilder::new(ComputeKernelRef::new(shader_ref(), "main"))
            .bind_texture("particles")
            .bind_storage_read("unknown")
            .dispatch_groups([1, 1, 1]);

        let diagnostics = builder
            .build(
                ShaderAssetKind::Surface,
                &[entry("main", RenderShaderStage::Fragment)],
                &[
                    resource(
                        "particles",
                        ShaderResourceKind::StorageBuffer,
                        ShaderResourceAccess::Write,
                    ),
                    resource(
                        "params",
                        ShaderResourceKind::UniformBuffer,
                        ShaderResourceAccess::Read,
                    ),
                ],
            )
            .unwrap_err();

        assert!(
            diagnostics.contains(&ShaderDispatchBuildDiagnostic::InvalidShaderKind {
                expected: ShaderAssetKind::Compute,
                actual: ShaderAssetKind::Surface,
            })
        );
        assert!(
            diagnostics.contains(&ShaderDispatchBuildDiagnostic::InvalidEntryPointStage {
                entry_point: "main".to_string(),
                stage: RenderShaderStage::Fragment,
                expected_stage: RenderShaderStage::Compute,
            })
        );
        assert!(
            diagnostics.contains(&ShaderDispatchBuildDiagnostic::ResourceKindMismatch {
                name: "particles".to_string(),
                expected: ShaderResourceKind::StorageBuffer,
                actual: ShaderResourceKind::Texture,
            })
        );
        assert!(
            diagnostics.contains(&ShaderDispatchBuildDiagnostic::MissingResource {
                name: "params".to_string(),
            })
        );
        assert!(
            diagnostics.contains(&ShaderDispatchBuildDiagnostic::UnknownResource {
                name: "unknown".to_string(),
            })
        );
    }

    #[test]
    fn render_compute_dispatch_builder_requires_dispatch_groups() {
        let builder = ComputeDispatchBuilder::new(ComputeKernelRef::new(shader_ref(), "cs_main"));

        let diagnostics = builder
            .build(
                ShaderAssetKind::Compute,
                &[entry("cs_main", RenderShaderStage::Compute)],
                &[],
            )
            .unwrap_err();

        assert_eq!(
            diagnostics,
            vec![ShaderDispatchBuildDiagnostic::MissingDispatchGroups {
                kernel: "cs_main".to_string(),
            }]
        );
    }
}
