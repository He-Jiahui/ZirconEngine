use crate::core::framework::render::{
    source_cubemap_face_mip_offset, source_cubemap_mip_size, ComputePipelineCacheKey, CubemapFace,
    IblBakeArtifactDescriptor, IblBakeArtifactReadbackSectionKind, IblBakeArtifactRequest,
    ShaderDispatchExtent, ShaderParameterValue, IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES,
    IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES, SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
};

use super::ibl_bake_graph_plan::{
    IBL_BAKE_IRRADIANCE_CUBE_RESOURCE, IBL_BAKE_IRRADIANCE_SH9_RESOURCE, IBL_BAKE_PMREM_RESOURCE,
};
use super::ibl_bake_shader_plan::{
    ibl_bake_compute_kernel_plans_for_request, IblBakeComputeKernelKind, IblBakeComputeKernelPlan,
};
mod realtime_slice;

pub(in crate::graphics::scene::scene_renderer) use realtime_slice::ibl_bake_wgpu_prefilter_command_for_slice;

pub(in crate::graphics::scene::scene_renderer) const IBL_BAKE_BINDING_PARAMS: u32 = 0;
pub(in crate::graphics::scene::scene_renderer) const IBL_BAKE_BINDING_SOURCE_CUBEMAP: u32 = 1;
pub(in crate::graphics::scene::scene_renderer) const IBL_BAKE_BINDING_SOURCE_SAMPLER: u32 = 2;
pub(in crate::graphics::scene::scene_renderer) const IBL_BAKE_BINDING_OUTPUT: u32 = 3;
const IBL_BAKE_CUBE_FACE_COUNT: u32 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::graphics::scene::scene_renderer) enum IblBakeWgpuOutputBindingKind {
    StorageTexture2DArray,
    StorageBuffer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer) struct IblBakeWgpuCommandPlanSet {
    pub descriptor: IblBakeArtifactDescriptor,
    pub commands: Vec<IblBakeWgpuCommandPlan>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer) struct IblBakeWgpuCommandPlan {
    pub kind: IblBakeComputeKernelKind,
    pub shader_locator: &'static str,
    pub wgsl_source: &'static str,
    pub pipeline_label: String,
    pub pipeline_key: ComputePipelineCacheKey,
    pub params: IblBakeWgpuParamsPlan,
    pub bind_group_layout_kind: IblBakeWgpuOutputBindingKind,
    pub output: IblBakeWgpuOutputPlan,
    pub dispatch_groups: [u32; 3],
    pub readback_copies: Vec<IblBakeWgpuReadbackCopyPlan>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer) struct IblBakeWgpuParamsPlan {
    words: Vec<u32>,
}

impl IblBakeWgpuParamsPlan {
    pub(in crate::graphics::scene::scene_renderer) fn byte_len(&self) -> u64 {
        self.words.len() as u64 * std::mem::size_of::<u32>() as u64
    }

    pub(in crate::graphics::scene::scene_renderer) fn words(&self) -> &[u32] {
        &self.words
    }

    pub(in crate::graphics::scene::scene_renderer) fn little_endian_bytes(&self) -> Vec<u8> {
        self.words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer) enum IblBakeWgpuOutputPlan {
    StorageTexture {
        resource_name: &'static str,
        view: IblBakeWgpuStorageTextureViewPlan,
    },
    StorageBuffer {
        resource_name: &'static str,
        byte_len: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer) struct IblBakeWgpuStorageTextureViewPlan {
    pub format: wgpu::TextureFormat,
    pub dimension: wgpu::TextureViewDimension,
    pub base_mip_level: u32,
    pub mip_level_count: u32,
    pub base_array_layer: u32,
    pub array_layer_count: u32,
    pub usage: wgpu::TextureUsages,
}

impl IblBakeWgpuStorageTextureViewPlan {
    pub(in crate::graphics::scene::scene_renderer) fn to_wgpu_descriptor(
        self,
    ) -> wgpu::TextureViewDescriptor<'static> {
        wgpu::TextureViewDescriptor {
            label: None,
            format: Some(self.format),
            dimension: Some(self.dimension),
            usage: Some(self.usage),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: self.base_mip_level,
            mip_level_count: Some(self.mip_level_count),
            base_array_layer: self.base_array_layer,
            array_layer_count: Some(self.array_layer_count),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer) struct IblBakeWgpuReadbackCopyPlan {
    pub section: IblBakeArtifactReadbackSectionKind,
    pub artifact_byte_offset: u64,
    pub unpadded_byte_len: u64,
    pub padded_byte_len: u64,
    pub source: IblBakeWgpuReadbackSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer) enum IblBakeWgpuReadbackSource {
    Texture {
        resource_name: &'static str,
        mip_level: u32,
        origin: [u32; 3],
        extent: [u32; 3],
        unpadded_bytes_per_row: u32,
        padded_bytes_per_row: u32,
        rows_per_image: u32,
    },
    Buffer {
        resource_name: &'static str,
        source_byte_offset: u64,
        byte_len: u64,
    },
}

pub(in crate::graphics::scene::scene_renderer) fn ibl_bake_wgpu_command_plan_for_request(
    request: &IblBakeArtifactRequest,
) -> IblBakeWgpuCommandPlanSet {
    let descriptor = IblBakeArtifactDescriptor::current_for_request(request);
    let commands = ibl_bake_compute_kernel_plans_for_request(request)
        .into_iter()
        .map(|kernel| ibl_bake_wgpu_command_plan_for_kernel(request, descriptor, kernel))
        .collect();

    IblBakeWgpuCommandPlanSet {
        descriptor,
        commands,
    }
}

pub(in crate::graphics::scene::scene_renderer) fn ibl_bake_wgpu_bind_group_layout_entries(
    output_kind: IblBakeWgpuOutputBindingKind,
) -> [wgpu::BindGroupLayoutEntry; 4] {
    [
        uniform_layout_entry(IBL_BAKE_BINDING_PARAMS),
        source_cubemap_layout_entry(IBL_BAKE_BINDING_SOURCE_CUBEMAP),
        source_sampler_layout_entry(IBL_BAKE_BINDING_SOURCE_SAMPLER),
        output_layout_entry(IBL_BAKE_BINDING_OUTPUT, output_kind),
    ]
}

pub(in crate::graphics::scene::scene_renderer) fn ibl_bake_storage_texture_view_plan(
    mip_level: u32,
) -> IblBakeWgpuStorageTextureViewPlan {
    IblBakeWgpuStorageTextureViewPlan {
        format: wgpu::TextureFormat::Rgba16Float,
        dimension: wgpu::TextureViewDimension::D2Array,
        base_mip_level: mip_level,
        mip_level_count: 1,
        base_array_layer: 0,
        array_layer_count: IBL_BAKE_CUBE_FACE_COUNT,
        usage: wgpu::TextureUsages::STORAGE_BINDING,
    }
}

fn ibl_bake_wgpu_command_plan_for_kernel(
    request: &IblBakeArtifactRequest,
    descriptor: IblBakeArtifactDescriptor,
    kernel: IblBakeComputeKernelPlan,
) -> IblBakeWgpuCommandPlan {
    let params = params_plan_for_kernel(&kernel);
    let (bind_group_layout_kind, output, readback_copies) = match kernel.kind {
        IblBakeComputeKernelKind::Pmrem { mip_level } => (
            IblBakeWgpuOutputBindingKind::StorageTexture2DArray,
            IblBakeWgpuOutputPlan::StorageTexture {
                resource_name: IBL_BAKE_PMREM_RESOURCE,
                view: ibl_bake_storage_texture_view_plan(mip_level),
            },
            pmrem_readback_copies(request, descriptor, mip_level),
        ),
        IblBakeComputeKernelKind::IrradianceSh9 => (
            IblBakeWgpuOutputBindingKind::StorageBuffer,
            IblBakeWgpuOutputPlan::StorageBuffer {
                resource_name: IBL_BAKE_IRRADIANCE_SH9_RESOURCE,
                byte_len: IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64,
            },
            vec![sh9_readback_copy(descriptor)],
        ),
        IblBakeComputeKernelKind::IrradianceCube => (
            IblBakeWgpuOutputBindingKind::StorageTexture2DArray,
            IblBakeWgpuOutputPlan::StorageTexture {
                resource_name: IBL_BAKE_IRRADIANCE_CUBE_RESOURCE,
                view: ibl_bake_storage_texture_view_plan(0),
            },
            irradiance_cube_readback_copies(descriptor),
        ),
    };

    IblBakeWgpuCommandPlan {
        kind: kernel.kind,
        shader_locator: kernel.shader_locator,
        wgsl_source: kernel.wgsl_source,
        pipeline_label: kernel.dispatch.pipeline_label,
        pipeline_key: kernel.dispatch.pipeline_key,
        params,
        bind_group_layout_kind,
        output,
        dispatch_groups: fixed_dispatch_groups(&kernel.dispatch.dispatch_extent),
        readback_copies,
    }
}

fn params_plan_for_kernel(kernel: &IblBakeComputeKernelPlan) -> IblBakeWgpuParamsPlan {
    let words = match kernel.kind {
        IblBakeComputeKernelKind::Pmrem { .. } => vec![
            u32_param(kernel, "face_size"),
            u32_param(kernel, "mip_face_size"),
            u32_param(kernel, "mip_level"),
            u32_param(kernel, "mip_count"),
            u32_param(kernel, "sample_count"),
            0,
            f32_param(kernel, "roughness").to_bits(),
            0,
        ],
        IblBakeComputeKernelKind::IrradianceSh9 => vec![
            u32_param(kernel, "source_face_size"),
            u32_param(kernel, "sample_face_size"),
            f32_param(kernel, "source_lod").to_bits(),
            0,
        ],
        IblBakeComputeKernelKind::IrradianceCube => vec![
            u32_param(kernel, "source_face_size"),
            u32_param(kernel, "irradiance_face_size"),
            u32_param(kernel, "sample_count"),
            0,
        ],
    };
    IblBakeWgpuParamsPlan { words }
}

fn u32_param(kernel: &IblBakeComputeKernelPlan, name: &str) -> u32 {
    match kernel.dispatch.parameters.get(name) {
        Some(ShaderParameterValue::U32 { value }) => *value,
        Some(other) => panic!("IBL bake parameter `{name}` should be u32, got {other:?}"),
        None => panic!("IBL bake parameter `{name}` is required"),
    }
}

fn f32_param(kernel: &IblBakeComputeKernelPlan, name: &str) -> f32 {
    match kernel.dispatch.parameters.get(name) {
        Some(ShaderParameterValue::F32 { value }) => *value,
        Some(other) => panic!("IBL bake parameter `{name}` should be f32, got {other:?}"),
        None => panic!("IBL bake parameter `{name}` is required"),
    }
}

fn fixed_dispatch_groups(extent: &ShaderDispatchExtent) -> [u32; 3] {
    match extent {
        ShaderDispatchExtent::Fixed(groups) => *groups,
        _ => [0, 0, 0],
    }
}

fn pmrem_readback_copies(
    request: &IblBakeArtifactRequest,
    descriptor: IblBakeArtifactDescriptor,
    mip_level: u32,
) -> Vec<IblBakeWgpuReadbackCopyPlan> {
    let Some(section_base) = pmrem_section_base_byte_offset(descriptor) else {
        return Vec::new();
    };
    let mip_size = source_cubemap_mip_size(request.pmrem_face_size(), mip_level);
    CubemapFace::ALL
        .into_iter()
        .map(|face| {
            let artifact_texel_offset = source_cubemap_face_mip_offset(
                request.pmrem_face_size(),
                request.pmrem_mip_count(),
                face,
                mip_level,
            );
            texture_readback_copy(
                IblBakeArtifactReadbackSectionKind::Pmrem,
                IBL_BAKE_PMREM_RESOURCE,
                mip_level,
                face.index() as u32,
                mip_size,
                section_base
                    + artifact_texel_offset as u64
                        * IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES as u64,
            )
        })
        .collect()
}

fn sh9_readback_copy(descriptor: IblBakeArtifactDescriptor) -> IblBakeWgpuReadbackCopyPlan {
    let artifact_byte_offset = sh9_section_base_byte_offset(descriptor).unwrap_or(0);
    IblBakeWgpuReadbackCopyPlan {
        section: IblBakeArtifactReadbackSectionKind::IrradianceSh9,
        artifact_byte_offset,
        unpadded_byte_len: IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64,
        padded_byte_len: IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64,
        source: IblBakeWgpuReadbackSource::Buffer {
            resource_name: IBL_BAKE_IRRADIANCE_SH9_RESOURCE,
            source_byte_offset: 0,
            byte_len: IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64,
        },
    }
}

fn irradiance_cube_readback_copies(
    descriptor: IblBakeArtifactDescriptor,
) -> Vec<IblBakeWgpuReadbackCopyPlan> {
    let Some(section_base) = irradiance_cube_section_base_byte_offset(descriptor) else {
        return Vec::new();
    };
    let face_size = SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE;
    let face_byte_len =
        face_size as u64 * face_size as u64 * IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES as u64;
    CubemapFace::ALL
        .into_iter()
        .map(|face| {
            texture_readback_copy(
                IblBakeArtifactReadbackSectionKind::IrradianceCube,
                IBL_BAKE_IRRADIANCE_CUBE_RESOURCE,
                0,
                face.index() as u32,
                face_size,
                section_base + face.index() as u64 * face_byte_len,
            )
        })
        .collect()
}

fn texture_readback_copy(
    section: IblBakeArtifactReadbackSectionKind,
    resource_name: &'static str,
    mip_level: u32,
    face_index: u32,
    face_size: u32,
    artifact_byte_offset: u64,
) -> IblBakeWgpuReadbackCopyPlan {
    let unpadded_bytes_per_row =
        face_size.saturating_mul(IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES as u32);
    let padded_bytes_per_row = padded_copy_bytes_per_row(unpadded_bytes_per_row);
    let rows_per_image = face_size.max(1);
    IblBakeWgpuReadbackCopyPlan {
        section,
        artifact_byte_offset,
        unpadded_byte_len: u64::from(unpadded_bytes_per_row) * u64::from(rows_per_image),
        padded_byte_len: u64::from(padded_bytes_per_row) * u64::from(rows_per_image),
        source: IblBakeWgpuReadbackSource::Texture {
            resource_name,
            mip_level,
            origin: [0, 0, face_index],
            extent: [face_size, face_size, 1],
            unpadded_bytes_per_row,
            padded_bytes_per_row,
            rows_per_image,
        },
    }
}

fn pmrem_section_base_byte_offset(descriptor: IblBakeArtifactDescriptor) -> Option<u64> {
    descriptor.expected_pmrem_rgba16f_size_bytes().map(|_| 0)
}

fn sh9_section_base_byte_offset(descriptor: IblBakeArtifactDescriptor) -> Option<u64> {
    let mut offset = 0;
    if let Some(bytes) = descriptor.expected_pmrem_rgba16f_size_bytes() {
        offset += bytes as u64;
    }
    descriptor
        .expected_irradiance_sh9_size_bytes()
        .map(|_| offset)
}

fn irradiance_cube_section_base_byte_offset(descriptor: IblBakeArtifactDescriptor) -> Option<u64> {
    let mut offset = 0;
    if let Some(bytes) = descriptor.expected_pmrem_rgba16f_size_bytes() {
        offset += bytes as u64;
    }
    if let Some(bytes) = descriptor.expected_irradiance_sh9_size_bytes() {
        offset += bytes as u64;
    }
    descriptor
        .expected_irradiance_cube_rgba16f_size_bytes()
        .map(|_| offset)
}

fn padded_copy_bytes_per_row(unpadded_bytes_per_row: u32) -> u32 {
    unpadded_bytes_per_row
        .max(1)
        .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
}

fn uniform_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn source_cubemap_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::Cube,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        },
        count: None,
    }
}

fn source_sampler_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

fn output_layout_entry(
    binding: u32,
    output_kind: IblBakeWgpuOutputBindingKind,
) -> wgpu::BindGroupLayoutEntry {
    let ty = match output_kind {
        IblBakeWgpuOutputBindingKind::StorageTexture2DArray => wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::Rgba16Float,
            view_dimension: wgpu::TextureViewDimension::D2Array,
        },
        IblBakeWgpuOutputBindingKind::StorageBuffer => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
    };

    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty,
        count: None,
    }
}

#[cfg(test)]
mod tests;
