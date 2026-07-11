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
    let descriptor = IblBakeArtifactDescriptor::current(
        request.bake_key(),
        request.pmrem_face_size(),
        request.pmrem_mip_count(),
        request.required_contents(),
    );
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
mod tests {
    use crate::core::framework::render::{
        source_cubemap_face_mip_offset, source_cubemap_sample_count, IblBakeArtifactContents,
        ProceduralSkyParams,
    };

    use super::*;

    #[test]
    fn bind_group_layout_entries_match_compute_shader_abi() {
        let entries = ibl_bake_wgpu_bind_group_layout_entries(
            IblBakeWgpuOutputBindingKind::StorageTexture2DArray,
        );

        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.binding)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert_uniform_entry(&entries[0]);
        assert_source_cubemap_entry(&entries[1]);
        assert_sampler_entry(&entries[2]);
        assert_storage_texture_output_entry(&entries[3]);

        let buffer_entries =
            ibl_bake_wgpu_bind_group_layout_entries(IblBakeWgpuOutputBindingKind::StorageBuffer);
        assert_storage_buffer_output_entry(&buffer_entries[3]);
    }

    #[test]
    fn command_plan_uses_per_mip_d2_array_storage_views() {
        let request = request(128, 8, IblBakeArtifactContents::PMREM_SH9_IEM);
        let plan = ibl_bake_wgpu_command_plan_for_request(&request);

        assert_eq!(plan.commands.len(), 10);
        let pmrem_commands = plan
            .commands
            .iter()
            .filter(|command| matches!(command.kind, IblBakeComputeKernelKind::Pmrem { .. }))
            .collect::<Vec<_>>();
        assert_eq!(pmrem_commands.len(), 8);

        for (mip_level, command) in pmrem_commands.iter().enumerate() {
            assert_eq!(
                command.bind_group_layout_kind,
                IblBakeWgpuOutputBindingKind::StorageTexture2DArray
            );
            let IblBakeWgpuOutputPlan::StorageTexture {
                resource_name,
                view,
            } = &command.output
            else {
                panic!("PMREM command should write a storage texture");
            };
            assert_eq!(*resource_name, IBL_BAKE_PMREM_RESOURCE);
            assert_eq!(*view, ibl_bake_storage_texture_view_plan(mip_level as u32));
            let descriptor = (*view).to_wgpu_descriptor();
            assert_eq!(
                descriptor.dimension,
                Some(wgpu::TextureViewDimension::D2Array)
            );
            assert_eq!(descriptor.base_mip_level, mip_level as u32);
            assert_eq!(descriptor.mip_level_count, Some(1));
            assert_eq!(descriptor.array_layer_count, Some(6));
            assert_eq!(descriptor.usage, Some(wgpu::TextureUsages::STORAGE_BINDING));
            assert_eq!(command.readback_copies.len(), 6);
        }

        assert_eq!(pmrem_commands[0].dispatch_groups, [16, 16, 6]);
        assert_eq!(pmrem_commands[7].dispatch_groups, [1, 1, 6]);
    }

    #[test]
    fn readback_plan_uses_face_major_artifact_offsets() {
        let request = request(4, 3, IblBakeArtifactContents::PMREM_SH9_IEM);
        let plan = ibl_bake_wgpu_command_plan_for_request(&request);
        let descriptor = plan.descriptor;

        let pmrem_mip0 = pmrem_command(&plan, 0);
        let pmrem_mip1 = pmrem_command(&plan, 1);
        assert_eq!(pmrem_mip0.readback_copies.len(), 6);
        assert_eq!(pmrem_mip1.readback_copies.len(), 6);
        assert_texture_copy(
            &pmrem_mip0.readback_copies[0],
            IblBakeArtifactReadbackSectionKind::Pmrem,
            IBL_BAKE_PMREM_RESOURCE,
            0,
            0,
            [128, 128, 1],
            0,
        );
        assert_texture_copy(
            &pmrem_mip1.readback_copies[0],
            IblBakeArtifactReadbackSectionKind::Pmrem,
            IBL_BAKE_PMREM_RESOURCE,
            1,
            0,
            [64, 64, 1],
            source_cubemap_face_mip_offset(128, 8, CubemapFace::PositiveX, 1) as u64 * 8,
        );
        assert_texture_copy(
            &pmrem_mip0.readback_copies[1],
            IblBakeArtifactReadbackSectionKind::Pmrem,
            IBL_BAKE_PMREM_RESOURCE,
            0,
            1,
            [128, 128, 1],
            source_cubemap_face_mip_offset(128, 8, CubemapFace::NegativeX, 0) as u64 * 8,
        );

        let pmrem_bytes = source_cubemap_sample_count(128, 8) as u64 * 8;
        let sh9 = plan
            .commands
            .iter()
            .find(|command| command.kind == IblBakeComputeKernelKind::IrradianceSh9)
            .expect("SH9 command should be present");
        assert_eq!(sh9.readback_copies.len(), 1);
        assert_eq!(sh9.readback_copies[0].artifact_byte_offset, pmrem_bytes);
        assert_eq!(
            sh9.readback_copies[0].unpadded_byte_len,
            IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64
        );

        let irradiance = plan
            .commands
            .iter()
            .find(|command| command.kind == IblBakeComputeKernelKind::IrradianceCube)
            .expect("IEM command should be present");
        let iem_base = pmrem_bytes + IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64;
        assert_eq!(irradiance.readback_copies.len(), 6);
        assert_texture_copy(
            &irradiance.readback_copies[0],
            IblBakeArtifactReadbackSectionKind::IrradianceCube,
            IBL_BAKE_IRRADIANCE_CUBE_RESOURCE,
            0,
            0,
            [
                SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
                SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
                1,
            ],
            iem_base,
        );
        assert_eq!(
            descriptor.expected_payload_size_bytes() as u64,
            iem_base
                + 6 * u64::from(SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE)
                    * u64::from(SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE)
                    * 8
        );
    }

    #[test]
    fn command_plan_serializes_wgsl_uniform_params_in_layout_order() {
        let request = request(128, 8, IblBakeArtifactContents::PMREM_SH9_IEM);
        let plan = ibl_bake_wgpu_command_plan_for_request(&request);

        let pmrem_mip0 = pmrem_command(&plan, 0);
        let pmrem_mip7 = pmrem_command(&plan, 7);
        assert_eq!(pmrem_mip0.params.byte_len(), 32);
        assert_eq!(
            pmrem_mip0.params.words(),
            &[128, 128, 0, 8, 32, 0, 0.0_f32.to_bits(), 0]
        );
        assert_eq!(
            pmrem_mip7.params.words(),
            &[128, 1, 7, 8, 128, 0, 1.0_f32.to_bits(), 0]
        );

        let sh9 = plan
            .commands
            .iter()
            .find(|command| command.kind == IblBakeComputeKernelKind::IrradianceSh9)
            .expect("SH9 command should be present");
        assert_eq!(sh9.params.byte_len(), 16);
        assert_eq!(sh9.params.words(), &[128, 32, 2.0_f32.to_bits(), 0]);

        let irradiance = plan
            .commands
            .iter()
            .find(|command| command.kind == IblBakeComputeKernelKind::IrradianceCube)
            .expect("IEM command should be present");
        assert_eq!(irradiance.params.byte_len(), 16);
        assert_eq!(irradiance.params.words(), &[128, 32, 64, 0]);
        assert_eq!(
            &pmrem_mip7.params.little_endian_bytes()[0..4],
            &128_u32.to_le_bytes()
        );
    }

    #[test]
    fn command_plan_omits_unrequested_outputs() {
        let request = request(64, 7, IblBakeArtifactContents::SH9);
        let plan = ibl_bake_wgpu_command_plan_for_request(&request);

        assert_eq!(plan.commands.len(), 1);
        assert_eq!(
            plan.commands[0].kind,
            IblBakeComputeKernelKind::IrradianceSh9
        );
        assert_eq!(
            plan.commands[0].params.words(),
            &[64, 32, 1.0_f32.to_bits(), 0]
        );
        assert_eq!(
            plan.commands[0].bind_group_layout_kind,
            IblBakeWgpuOutputBindingKind::StorageBuffer
        );
        assert_eq!(plan.commands[0].readback_copies[0].artifact_byte_offset, 0);
    }

    fn request(
        face_size: u32,
        mip_count: u32,
        contents: IblBakeArtifactContents,
    ) -> IblBakeArtifactRequest {
        IblBakeArtifactRequest::new(
            ProceduralSkyParams::default_gradient().ibl_bake_key(),
            face_size,
            mip_count,
        )
        .with_required_contents(contents)
    }

    fn pmrem_command(plan: &IblBakeWgpuCommandPlanSet, mip_level: u32) -> &IblBakeWgpuCommandPlan {
        plan.commands
            .iter()
            .find(|command| command.kind == IblBakeComputeKernelKind::Pmrem { mip_level })
            .expect("PMREM mip command should be present")
    }

    fn assert_texture_copy(
        copy: &IblBakeWgpuReadbackCopyPlan,
        section: IblBakeArtifactReadbackSectionKind,
        resource_name: &'static str,
        mip_level: u32,
        face_index: u32,
        extent: [u32; 3],
        artifact_byte_offset: u64,
    ) {
        assert_eq!(copy.section, section);
        assert_eq!(copy.artifact_byte_offset, artifact_byte_offset);
        let IblBakeWgpuReadbackSource::Texture {
            resource_name: actual_resource,
            mip_level: actual_mip,
            origin,
            extent: actual_extent,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
            rows_per_image,
        } = &copy.source
        else {
            panic!("expected texture readback copy");
        };
        assert_eq!(*actual_resource, resource_name);
        assert_eq!(*actual_mip, mip_level);
        assert_eq!(*origin, [0, 0, face_index]);
        assert_eq!(*actual_extent, extent);
        assert_eq!(*unpadded_bytes_per_row, extent[0] * 8);
        assert_eq!(
            *padded_bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT,
            0
        );
        assert_eq!(*rows_per_image, extent[1]);
    }

    fn assert_uniform_entry(entry: &wgpu::BindGroupLayoutEntry) {
        assert_eq!(entry.visibility, wgpu::ShaderStages::COMPUTE);
        let wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        } = &entry.ty
        else {
            panic!("binding {} should be a uniform buffer", entry.binding);
        };
    }

    fn assert_source_cubemap_entry(entry: &wgpu::BindGroupLayoutEntry) {
        let wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::Cube,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        } = &entry.ty
        else {
            panic!("binding {} should be a sampled cube texture", entry.binding);
        };
    }

    fn assert_sampler_entry(entry: &wgpu::BindGroupLayoutEntry) {
        let wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering) = &entry.ty else {
            panic!("binding {} should be a filtering sampler", entry.binding);
        };
    }

    fn assert_storage_texture_output_entry(entry: &wgpu::BindGroupLayoutEntry) {
        let wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::Rgba16Float,
            view_dimension: wgpu::TextureViewDimension::D2Array,
        } = &entry.ty
        else {
            panic!(
                "binding {} should be a write-only rgba16float D2Array storage texture",
                entry.binding
            );
        };
    }

    fn assert_storage_buffer_output_entry(entry: &wgpu::BindGroupLayoutEntry) {
        let wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        } = &entry.ty
        else {
            panic!(
                "binding {} should be a writable storage buffer",
                entry.binding
            );
        };
    }
}
