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
    assert_eq!(pmrem_commands[7].dispatch_groups, [1, 1, 1]);
    assert_eq!(pmrem_commands[7].params.words()[7], 1.0_f32.to_bits());
}

#[test]
fn one_texel_single_mip_pmrem_plan_writes_the_terminal_average_to_all_faces() {
    let request = request(16, 5, IblBakeArtifactContents::PMREM).with_pmrem_layout(1, 1);
    let plan = ibl_bake_wgpu_command_plan_for_request(&request);
    let command = pmrem_command(&plan, 0);

    assert_eq!(command.dispatch_groups, [1, 1, 1]);
    assert_eq!(
        command.params.words(),
        &[1, 1, 0, 1, 32, 0, 0.0_f32.to_bits(), 1.0_f32.to_bits()]
    );
    assert_eq!(command.readback_copies.len(), 6);
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
        &[128, 1, 7, 8, 128, 0, 1.0_f32.to_bits(), 1.0_f32.to_bits()]
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
    assert_eq!(irradiance.params.words(), &[128, 32, 64, 2]);
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
