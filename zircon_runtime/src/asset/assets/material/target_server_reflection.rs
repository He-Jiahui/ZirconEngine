use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ShaderTextureViewDimension {
    D1,
    D2,
    D2Array,
    Cube,
    CubeArray,
    D3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ShaderTextureSampleType {
    Float,
    Depth,
    Sint,
    Uint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ShaderBindingResourceType {
    UniformBuffer,
    StorageBuffer {
        read_only: bool,
    },
    SampledTexture {
        view_dimension: ShaderTextureViewDimension,
        sample_type: ShaderTextureSampleType,
        multisampled: bool,
    },
    Sampler {
        comparison: bool,
    },
    Unsupported,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShaderStageVisibility(u16);

impl ShaderStageVisibility {
    fn insert(&mut self, stage: naga::ShaderStage) {
        self.0 |= stage_bit(stage);
    }

    pub(crate) fn contains(self, stage: naga::ShaderStage) -> bool {
        self.0 & stage_bit(stage) != 0
    }

    fn from_stages(stages: impl IntoIterator<Item = naga::ShaderStage>) -> Self {
        let mut visibility = Self::default();
        for stage in stages {
            visibility.0 |= stage_bit(stage);
        }
        visibility
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ShaderResourceBindingIdentity {
    pub(crate) group: u32,
    pub(crate) binding: u32,
    pub(crate) resource_type: ShaderBindingResourceType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderResourceBindingReflection {
    pub(crate) identity: ShaderResourceBindingIdentity,
    pub(crate) name: Option<String>,
    pub(crate) visibility: ShaderStageVisibility,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderTemplateReflection {
    pub(crate) resource_bindings: Vec<ShaderResourceBindingReflection>,
}

pub(crate) fn reflect_validated_shader_module(
    module: &naga::Module,
    module_info: &naga::valid::ModuleInfo,
) -> ShaderTemplateReflection {
    let mut resource_bindings = BTreeMap::new();
    for (entry_index, entry_point) in module.entry_points.iter().enumerate() {
        let entry_info = module_info.get_entry_point(entry_index);
        for (handle, global) in module.global_variables.iter() {
            let Some(binding) = global.binding else {
                continue;
            };
            if entry_info[handle].is_empty() {
                continue;
            }
            let key = (binding.group, binding.binding);
            let reflected =
                resource_bindings
                    .entry(key)
                    .or_insert_with(|| ShaderResourceBindingReflection {
                        identity: ShaderResourceBindingIdentity {
                            group: binding.group,
                            binding: binding.binding,
                            resource_type: shader_binding_resource_type(module, global),
                        },
                        name: global.name.clone(),
                        visibility: ShaderStageVisibility::default(),
                    });
            reflected.visibility.insert(entry_point.stage);
        }
    }
    ShaderTemplateReflection {
        resource_bindings: resource_bindings.into_values().collect(),
    }
}

pub(crate) fn reflect_declared_shader_resources(
    module: &naga::Module,
    expected_visibility: &[naga::ShaderStage],
) -> Vec<ShaderResourceBindingReflection> {
    reflect_declared_shader_resources_with_visibility(
        module,
        ShaderStageVisibility::from_stages(expected_visibility.iter().copied()),
    )
}

fn reflect_declared_shader_resources_with_visibility(
    module: &naga::Module,
    visibility: ShaderStageVisibility,
) -> Vec<ShaderResourceBindingReflection> {
    let mut reflected = BTreeMap::new();
    for (_, global) in module.global_variables.iter() {
        let Some(binding) = global.binding else {
            continue;
        };
        reflected.insert(
            (binding.group, binding.binding),
            ShaderResourceBindingReflection {
                identity: ShaderResourceBindingIdentity {
                    group: binding.group,
                    binding: binding.binding,
                    resource_type: shader_binding_resource_type(module, global),
                },
                name: global.name.clone(),
                visibility,
            },
        );
    }
    reflected.into_values().collect()
}

fn shader_binding_resource_type(
    module: &naga::Module,
    global: &naga::GlobalVariable,
) -> ShaderBindingResourceType {
    match global.space {
        naga::AddressSpace::Uniform => ShaderBindingResourceType::UniformBuffer,
        naga::AddressSpace::Storage { access } => ShaderBindingResourceType::StorageBuffer {
            read_only: !access.contains(naga::StorageAccess::STORE),
        },
        naga::AddressSpace::Handle => match &module.types[global.ty].inner {
            naga::TypeInner::Image {
                dim,
                arrayed,
                class: naga::ImageClass::Sampled { kind, multi },
            } => match (
                shader_texture_view_dimension(*dim, *arrayed),
                shader_texture_sample_type(*kind),
            ) {
                (Some(view_dimension), Some(sample_type)) => {
                    ShaderBindingResourceType::SampledTexture {
                        view_dimension,
                        sample_type,
                        multisampled: *multi,
                    }
                }
                _ => ShaderBindingResourceType::Unsupported,
            },
            naga::TypeInner::Image {
                dim,
                arrayed,
                class: naga::ImageClass::Depth { multi },
            } => shader_texture_view_dimension(*dim, *arrayed).map_or(
                ShaderBindingResourceType::Unsupported,
                |view_dimension| ShaderBindingResourceType::SampledTexture {
                    view_dimension,
                    sample_type: ShaderTextureSampleType::Depth,
                    multisampled: *multi,
                },
            ),
            naga::TypeInner::Sampler { comparison } => ShaderBindingResourceType::Sampler {
                comparison: *comparison,
            },
            _ => ShaderBindingResourceType::Unsupported,
        },
        _ => ShaderBindingResourceType::Unsupported,
    }
}

const fn shader_texture_view_dimension(
    dimension: naga::ImageDimension,
    arrayed: bool,
) -> Option<ShaderTextureViewDimension> {
    match (dimension, arrayed) {
        (naga::ImageDimension::D1, false) => Some(ShaderTextureViewDimension::D1),
        (naga::ImageDimension::D2, false) => Some(ShaderTextureViewDimension::D2),
        (naga::ImageDimension::D2, true) => Some(ShaderTextureViewDimension::D2Array),
        (naga::ImageDimension::Cube, false) => Some(ShaderTextureViewDimension::Cube),
        (naga::ImageDimension::Cube, true) => Some(ShaderTextureViewDimension::CubeArray),
        (naga::ImageDimension::D3, false) => Some(ShaderTextureViewDimension::D3),
        (naga::ImageDimension::D1 | naga::ImageDimension::D3, true) => None,
    }
}

const fn shader_texture_sample_type(kind: naga::ScalarKind) -> Option<ShaderTextureSampleType> {
    match kind {
        naga::ScalarKind::Float => Some(ShaderTextureSampleType::Float),
        naga::ScalarKind::Sint => Some(ShaderTextureSampleType::Sint),
        naga::ScalarKind::Uint => Some(ShaderTextureSampleType::Uint),
        naga::ScalarKind::Bool
        | naga::ScalarKind::AbstractInt
        | naga::ScalarKind::AbstractFloat => None,
    }
}

const fn stage_bit(stage: naga::ShaderStage) -> u16 {
    match stage {
        naga::ShaderStage::Vertex => 1 << 0,
        naga::ShaderStage::Task => 1 << 1,
        naga::ShaderStage::Mesh => 1 << 2,
        naga::ShaderStage::Fragment => 1 << 3,
        naga::ShaderStage::Compute => 1 << 4,
        naga::ShaderStage::RayGeneration => 1 << 5,
        naga::ShaderStage::Miss => 1 << 6,
        naga::ShaderStage::AnyHit => 1 << 7,
        naga::ShaderStage::ClosestHit => 1 << 8,
    }
}

#[cfg(test)]
#[path = "tests/target_server_reflection.rs"]
mod tests;
