use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::core::framework::render::{
    COMPUTE_SHADER_PARAMS_BINDING, COMPUTE_SHADER_RESOURCE_GROUP, ComputeDispatchPlan,
    FULLSCREEN_PASS_INPUT_GROUP, FullscreenPassPlan, ShaderNamedResourceBinding,
    ShaderResourceAccess, ShaderResourceKind,
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
mod tests {
    use crate::core::framework::render::ShaderResourceKind;
    use crate::graphics::shader::builtin_global_shader_contracts::{
        HZB_SCENE_DEPTH_RESOURCE, HZB_SOURCE_RESOURCE, HZB_TARGET_RESOURCE,
        MOTION_VECTOR_SOURCE_RESOURCE, hzb_build_dispatch_plan, motion_vector_tile_max_pass_plan,
    };

    use super::*;

    #[test]
    fn hzb_compute_layout_is_projected_from_named_plan_bindings() {
        let entries = compute_shader_bind_group_layout_entries(
            hzb_build_dispatch_plan(),
            &[
                ShaderWgpuResourceDescriptor::texture(
                    HZB_SCENE_DEPTH_RESOURCE,
                    wgpu::TextureSampleType::Depth,
                    wgpu::TextureViewDimension::D2,
                    false,
                ),
                ShaderWgpuResourceDescriptor::texture(
                    HZB_SOURCE_RESOURCE,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::TextureViewDimension::D2,
                    false,
                ),
                ShaderWgpuResourceDescriptor::storage_texture(
                    HZB_TARGET_RESOURCE,
                    wgpu::TextureFormat::Rgba16Float,
                    wgpu::TextureViewDimension::D2,
                ),
            ],
        )
        .expect("HZB plan should project to WGPU entries");

        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.binding)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert!(matches!(
            &entries[0].ty,
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                ..
            }
        ));
        assert!(matches!(
            &entries[1].ty,
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Depth,
                ..
            }
        ));
        assert!(matches!(
            &entries[3].ty,
            wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba16Float,
                ..
            }
        ));
    }

    #[test]
    fn hzb_compute_layout_reports_missing_named_resource_before_wgpu_creation() {
        let error = compute_shader_bind_group_layout_entries(
            hzb_build_dispatch_plan(),
            &[
                ShaderWgpuResourceDescriptor::texture(
                    HZB_SCENE_DEPTH_RESOURCE,
                    wgpu::TextureSampleType::Depth,
                    wgpu::TextureViewDimension::D2,
                    false,
                ),
                ShaderWgpuResourceDescriptor::texture(
                    HZB_SOURCE_RESOURCE,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::TextureViewDimension::D2,
                    false,
                ),
            ],
        )
        .expect_err("missing HZB target type must fail before WGPU layout creation");

        assert_eq!(
            error,
            GlobalShaderPipelineLayoutError::MissingResourceType {
                name: HZB_TARGET_RESOURCE.to_string(),
            }
        );
    }

    #[test]
    fn fullscreen_layout_reports_named_type_mismatch_before_wgpu_creation() {
        let error = fullscreen_pass_input_layout_entries(
            motion_vector_tile_max_pass_plan(),
            &[ShaderWgpuResourceDescriptor::storage_texture(
                MOTION_VECTOR_SOURCE_RESOURCE,
                wgpu::TextureFormat::Rgba16Float,
                wgpu::TextureViewDimension::D2,
            )],
        )
        .expect_err("texture contract must reject a storage-texture projection");

        assert_eq!(
            error,
            GlobalShaderPipelineLayoutError::ResourceKindMismatch {
                name: MOTION_VECTOR_SOURCE_RESOURCE.to_string(),
                expected: ShaderResourceKind::Texture,
                actual: ShaderResourceKind::StorageTexture,
            }
        );
    }
}

#[cfg(test)]
mod optimization_batch_20260830cm_runtime390_tests {
    use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
    use std::hint::black_box;
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const RESOURCES_PER_SAMPLE: usize = 512;

    #[test]
    fn shader_layout_projection_uses_preallocated_hash_lookups() {
        let source = include_str!("global_pipeline_layout.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("global shader layout implementation");

        assert!(implementation.contains("use std::collections::{HashMap, HashSet}"));
        assert!(implementation.contains("HashMap::with_capacity(resource_types.len())"));
        assert!(implementation.contains("HashSet::with_capacity(bindings.len())"));
        assert!(!implementation.contains("BTreeMap::new()"));
        assert!(!implementation.contains("collect::<BTreeSet<_>>()"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cm_runtime390_shader_layout_hash_lookup_p95() {
        let names = (0..RESOURCES_PER_SAMPLE)
            .map(|index| format!("shader_resource_{index:04}_shared_prefix"))
            .collect::<Vec<_>>();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(&names, false));
                optimized.push(measure(&names, true));
            } else {
                optimized.push(measure(&names, true));
                legacy.push(measure(&names, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME390_SHADER_LAYOUT_HASH_LOOKUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} resources_per_sample={RESOURCES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(names: &[String], use_hash: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..64 {
            if use_hash {
                let mut by_name = HashMap::with_capacity(names.len());
                let mut declared = HashSet::with_capacity(names.len());
                for (index, name) in black_box(names).iter().enumerate() {
                    by_name.insert(name.as_str(), index);
                    declared.insert(name.as_str());
                }
                checksum ^= names
                    .iter()
                    .map(|name| {
                        by_name[name.as_str()] + usize::from(declared.contains(name.as_str()))
                    })
                    .sum::<usize>();
            } else {
                let mut by_name = BTreeMap::new();
                let mut declared = BTreeSet::new();
                for (index, name) in black_box(names).iter().enumerate() {
                    by_name.insert(name.as_str(), index);
                    declared.insert(name.as_str());
                }
                checksum ^= names
                    .iter()
                    .map(|name| {
                        by_name[name.as_str()] + usize::from(declared.contains(name.as_str()))
                    })
                    .sum::<usize>();
            }
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
