use crate::graphics::shader::{
    ShaderBindingResourceType, ShaderTextureSampleType, ShaderTextureViewDimension,
};

pub(super) fn shader_binding_resource_type(
    module: &naga::Module,
    global: &naga::GlobalVariable,
) -> ShaderBindingResourceType {
    match global.space {
        naga::AddressSpace::Uniform => ShaderBindingResourceType::UniformBuffer,
        naga::AddressSpace::Storage { access } => ShaderBindingResourceType::StorageBuffer {
            read_only: !access.contains(naga::StorageAccess::STORE),
        },
        naga::AddressSpace::Handle => shader_handle_binding_resource_type(module, global.ty),
        _ => ShaderBindingResourceType::Unsupported,
    }
}

pub(super) fn shader_min_buffer_binding_size(
    module: &naga::Module,
    global: &naga::GlobalVariable,
) -> Option<u64> {
    matches!(
        global.space,
        naga::AddressSpace::Uniform | naga::AddressSpace::Storage { .. }
    )
    .then(|| module.types[global.ty].inner.size(module.to_ctx()) as u64)
}

fn shader_handle_binding_resource_type(
    module: &naga::Module,
    handle: naga::Handle<naga::Type>,
) -> ShaderBindingResourceType {
    match &module.types[handle].inner {
        naga::TypeInner::Image {
            dim,
            arrayed,
            class: naga::ImageClass::Sampled { kind, multi },
        } => shader_texture_view_dimension(*dim, *arrayed).map_or(
            ShaderBindingResourceType::Unsupported,
            |view_dimension| {
                shader_texture_sample_type(*kind).map_or(
                    ShaderBindingResourceType::Unsupported,
                    |sample_type| ShaderBindingResourceType::SampledTexture {
                        view_dimension,
                        sample_type,
                        multisampled: *multi,
                    },
                )
            },
        ),
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
        naga::TypeInner::Image { .. }
        | naga::TypeInner::AccelerationStructure { .. }
        | naga::TypeInner::BindingArray { .. } => ShaderBindingResourceType::Unsupported,
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
