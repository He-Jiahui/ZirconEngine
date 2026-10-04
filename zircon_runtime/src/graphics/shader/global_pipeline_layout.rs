use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::core::framework::render::{ShaderResourceAccess, ShaderResourceKind};

use super::invocation::{
    ComputeDispatchPlan, FullscreenPassPlan, ShaderNamedResourceBinding,
    COMPUTE_SHADER_PARAMS_BINDING, COMPUTE_SHADER_RESOURCE_GROUP, FULLSCREEN_PASS_INPUT_GROUP,
};

#[derive(Clone, Debug)]
pub(crate) struct ShaderWgpuResourceDescriptor {
    name: String,
    binding_type: ShaderWgpuResourceBindingType,
}

#[derive(Clone, Copy, Debug)]
enum ShaderWgpuResourceBindingType {
    Texture {
        sample_type: wgpu::TextureSampleType,
        view_dimension: wgpu::TextureViewDimension,
        multisampled: bool,
    },
    StorageTexture {
        format: wgpu::TextureFormat,
        view_dimension: wgpu::TextureViewDimension,
    },
}

impl ShaderWgpuResourceDescriptor {
    pub(crate) fn texture(
        name: impl Into<String>,
        sample_type: wgpu::TextureSampleType,
        view_dimension: wgpu::TextureViewDimension,
        multisampled: bool,
    ) -> Self {
        Self::new(
            name,
            ShaderWgpuResourceBindingType::Texture {
                sample_type,
                view_dimension,
                multisampled,
            },
        )
    }

    pub(crate) fn storage_texture(
        name: impl Into<String>,
        format: wgpu::TextureFormat,
        view_dimension: wgpu::TextureViewDimension,
    ) -> Self {
        Self::new(
            name,
            ShaderWgpuResourceBindingType::StorageTexture {
                format,
                view_dimension,
            },
        )
    }

    fn new(name: impl Into<String>, binding_type: ShaderWgpuResourceBindingType) -> Self {
        Self {
            name: name.into(),
            binding_type,
        }
    }

    fn shader_kind(&self) -> ShaderResourceKind {
        match self.binding_type {
            ShaderWgpuResourceBindingType::Texture { .. } => ShaderResourceKind::Texture,
            ShaderWgpuResourceBindingType::StorageTexture { .. } => {
                ShaderResourceKind::StorageTexture
            }
        }
    }

    fn wgpu_binding_type(&self, access: ShaderResourceAccess) -> wgpu::BindingType {
        match self.binding_type {
            ShaderWgpuResourceBindingType::Texture {
                sample_type,
                view_dimension,
                multisampled,
            } => wgpu::BindingType::Texture {
                sample_type,
                view_dimension,
                multisampled,
            },
            ShaderWgpuResourceBindingType::StorageTexture {
                format,
                view_dimension,
            } => wgpu::BindingType::StorageTexture {
                access: match access {
                    ShaderResourceAccess::Read => wgpu::StorageTextureAccess::ReadOnly,
                    ShaderResourceAccess::Write => wgpu::StorageTextureAccess::WriteOnly,
                    ShaderResourceAccess::ReadWrite => wgpu::StorageTextureAccess::ReadWrite,
                },
                format,
                view_dimension,
            },
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum GlobalShaderPipelineLayoutError {
    #[error("shader resource `{name}` has no WGPU resource type")]
    MissingResourceType { name: String },
    #[error("WGPU resource type `{name}` is not declared by the shader contract")]
    UnknownResourceType { name: String },
    #[error("WGPU resource type `{name}` was declared more than once")]
    DuplicateResourceType { name: String },
    #[error("shader resource `{name}` expects {expected:?}, got {actual:?}")]
    ResourceKindMismatch {
        name: String,
        expected: ShaderResourceKind,
        actual: ShaderResourceKind,
    },
    #[error("shader resource `{name}` expects ABI group {expected}, got {actual}")]
    AbiGroupMismatch {
        name: String,
        expected: u32,
        actual: u32,
    },
}

pub(crate) fn compute_shader_bind_group_layout_entries(
    plan: &ComputeDispatchPlan,
    resource_types: &[ShaderWgpuResourceDescriptor],
) -> Result<Vec<wgpu::BindGroupLayoutEntry>, GlobalShaderPipelineLayoutError> {
    let mut entries = vec![wgpu::BindGroupLayoutEntry {
        binding: COMPUTE_SHADER_PARAMS_BINDING.binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }];
    entries.extend(project_resource_entries(
        &plan.resources,
        resource_types,
        COMPUTE_SHADER_RESOURCE_GROUP,
        wgpu::ShaderStages::COMPUTE,
    )?);
    entries.sort_by_key(|entry| entry.binding);
    Ok(entries)
}

pub(crate) fn fullscreen_pass_input_layout_entries(
    plan: &FullscreenPassPlan,
    resource_types: &[ShaderWgpuResourceDescriptor],
) -> Result<Vec<wgpu::BindGroupLayoutEntry>, GlobalShaderPipelineLayoutError> {
    let mut entries = project_resource_entries(
        &plan.resources,
        resource_types,
        FULLSCREEN_PASS_INPUT_GROUP,
        wgpu::ShaderStages::FRAGMENT,
    )?;
    entries.sort_by_key(|entry| entry.binding);
    Ok(entries)
}

pub(crate) fn create_compute_shader_bind_group_layout(
    device: &wgpu::Device,
    plan: &ComputeDispatchPlan,
    resource_types: &[ShaderWgpuResourceDescriptor],
) -> Result<wgpu::BindGroupLayout, GlobalShaderPipelineLayoutError> {
    let label = format!("{}-bind-group-layout", plan.pipeline_label);
    let entries = compute_shader_bind_group_layout_entries(plan, resource_types)?;
    Ok(
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&label),
            entries: &entries,
        }),
    )
}

pub(crate) fn create_fullscreen_pass_input_bind_group_layout(
    device: &wgpu::Device,
    plan: &FullscreenPassPlan,
    resource_types: &[ShaderWgpuResourceDescriptor],
) -> Result<wgpu::BindGroupLayout, GlobalShaderPipelineLayoutError> {
    let label = format!("{}-pass-input-layout", plan.pipeline_label);
    let entries = fullscreen_pass_input_layout_entries(plan, resource_types)?;
    Ok(
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&label),
            entries: &entries,
        }),
    )
}

fn project_resource_entries(
    bindings: &[ShaderNamedResourceBinding],
    resource_types: &[ShaderWgpuResourceDescriptor],
    expected_group: u32,
    visibility: wgpu::ShaderStages,
) -> Result<Vec<wgpu::BindGroupLayoutEntry>, GlobalShaderPipelineLayoutError> {
    let mut by_name = HashMap::with_capacity(resource_types.len());
    for descriptor in resource_types {
        if by_name
            .insert(descriptor.name.as_str(), descriptor)
            .is_some()
        {
            return Err(GlobalShaderPipelineLayoutError::DuplicateResourceType {
                name: descriptor.name.clone(),
            });
        }
    }

    let mut declared_names = HashSet::with_capacity(bindings.len());
    declared_names.extend(bindings.iter().map(|binding| binding.name.as_str()));
    if let Some(unknown) = resource_types
        .iter()
        .find(|descriptor| !declared_names.contains(descriptor.name.as_str()))
    {
        return Err(GlobalShaderPipelineLayoutError::UnknownResourceType {
            name: unknown.name.clone(),
        });
    }

    bindings
        .iter()
        .map(|binding| {
            if binding.abi.group != expected_group {
                return Err(GlobalShaderPipelineLayoutError::AbiGroupMismatch {
                    name: binding.name.clone(),
                    expected: expected_group,
                    actual: binding.abi.group,
                });
            }
            let descriptor = by_name.get(binding.name.as_str()).ok_or_else(|| {
                GlobalShaderPipelineLayoutError::MissingResourceType {
                    name: binding.name.clone(),
                }
            })?;
            let actual = descriptor.shader_kind();
            if actual != binding.kind {
                return Err(GlobalShaderPipelineLayoutError::ResourceKindMismatch {
                    name: binding.name.clone(),
                    expected: binding.kind,
                    actual,
                });
            }
            Ok(wgpu::BindGroupLayoutEntry {
                binding: binding.abi.binding,
                visibility,
                ty: descriptor.wgpu_binding_type(binding.access),
                count: None,
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/global_pipeline_layout.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/global_pipeline_layout_optimization_batch_20260830cm_runtime390_tests.rs"]
mod optimization_batch_20260830cm_runtime390_tests;
