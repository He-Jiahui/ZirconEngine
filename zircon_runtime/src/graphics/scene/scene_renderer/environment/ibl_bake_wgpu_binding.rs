use wgpu::util::DeviceExt;

use super::ibl_bake_wgpu_command_plan::{
    ibl_bake_wgpu_bind_group_layout_entries, IblBakeWgpuCommandPlan, IblBakeWgpuOutputBindingKind,
    IblBakeWgpuOutputPlan, IBL_BAKE_BINDING_OUTPUT, IBL_BAKE_BINDING_PARAMS,
    IBL_BAKE_BINDING_SOURCE_CUBEMAP, IBL_BAKE_BINDING_SOURCE_SAMPLER,
};

pub(in crate::graphics::scene::scene_renderer) struct IblBakeWgpuBindGroupLayouts {
    storage_texture: wgpu::BindGroupLayout,
    storage_buffer: wgpu::BindGroupLayout,
}

impl IblBakeWgpuBindGroupLayouts {
    pub(in crate::graphics::scene::scene_renderer) fn new(device: &wgpu::Device) -> Self {
        Self {
            storage_texture: create_ibl_bake_wgpu_bind_group_layout(
                device,
                IblBakeWgpuOutputBindingKind::StorageTexture2DArray,
            ),
            storage_buffer: create_ibl_bake_wgpu_bind_group_layout(
                device,
                IblBakeWgpuOutputBindingKind::StorageBuffer,
            ),
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn layout(
        &self,
        output_kind: IblBakeWgpuOutputBindingKind,
    ) -> &wgpu::BindGroupLayout {
        match output_kind {
            IblBakeWgpuOutputBindingKind::StorageTexture2DArray => &self.storage_texture,
            IblBakeWgpuOutputBindingKind::StorageBuffer => &self.storage_buffer,
        }
    }
}

pub(in crate::graphics::scene::scene_renderer) enum IblBakeWgpuOutputBindingResource<'a> {
    StorageTexture2DArray(&'a wgpu::TextureView),
    StorageBuffer(&'a wgpu::Buffer),
    /// Graph-backed output with the compiler-proven byte window.
    ///
    /// The legacy `StorageBuffer` variant remains for direct environment
    /// capture targets, which are not owned by a compiled render graph.
    StorageBufferRange {
        buffer: &'a wgpu::Buffer,
        offset: wgpu::BufferAddress,
        size: Option<std::num::NonZeroU64>,
    },
}

impl IblBakeWgpuOutputBindingResource<'_> {
    fn kind(&self) -> IblBakeWgpuOutputBindingKind {
        match self {
            Self::StorageTexture2DArray(_) => IblBakeWgpuOutputBindingKind::StorageTexture2DArray,
            Self::StorageBuffer(_) | Self::StorageBufferRange { .. } => {
                IblBakeWgpuOutputBindingKind::StorageBuffer
            }
        }
    }

    fn as_binding_resource(&self) -> wgpu::BindingResource<'_> {
        match self {
            Self::StorageTexture2DArray(view) => wgpu::BindingResource::TextureView(view),
            Self::StorageBuffer(buffer) => buffer.as_entire_binding(),
            Self::StorageBufferRange {
                buffer,
                offset,
                size,
            } => wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: *offset,
                size: *size,
            }),
        }
    }
}

pub(in crate::graphics::scene::scene_renderer) fn create_ibl_bake_wgpu_params_buffer(
    device: &wgpu::Device,
    command: &IblBakeWgpuCommandPlan,
) -> wgpu::Buffer {
    let label = format!("{}-params", command.pipeline_label);
    let contents = command.params.little_endian_bytes();
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label.as_str()),
        contents: &contents,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

pub(in crate::graphics::scene::scene_renderer) fn create_ibl_bake_wgpu_source_sampler(
    device: &wgpu::Device,
) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("zircon-env-ibl-bake-source-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    })
}

pub(in crate::graphics::scene::scene_renderer) fn create_ibl_bake_wgpu_bind_group(
    device: &wgpu::Device,
    layouts: &IblBakeWgpuBindGroupLayouts,
    command: &IblBakeWgpuCommandPlan,
    params_buffer: &wgpu::Buffer,
    source_cubemap_view: &wgpu::TextureView,
    source_sampler: &wgpu::Sampler,
    output: IblBakeWgpuOutputBindingResource<'_>,
) -> Result<wgpu::BindGroup, String> {
    let planned_kind = output_kind_from_plan(&command.output)?;
    if command.bind_group_layout_kind != planned_kind {
        return Err(format!(
            "IBL bake command `{}` declares {:?} layout but output plan requires {:?}",
            command.pipeline_label, command.bind_group_layout_kind, planned_kind
        ));
    }
    if output.kind() != planned_kind {
        return Err(format!(
            "IBL bake command `{}` expects {:?} output binding, got {:?}",
            command.pipeline_label,
            planned_kind,
            output.kind()
        ));
    }

    let label = format!("{}-bind-group", command.pipeline_label);
    Ok(device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label.as_str()),
        layout: layouts.layout(planned_kind),
        entries: &[
            wgpu::BindGroupEntry {
                binding: IBL_BAKE_BINDING_PARAMS,
                resource: params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: IBL_BAKE_BINDING_SOURCE_CUBEMAP,
                resource: wgpu::BindingResource::TextureView(source_cubemap_view),
            },
            wgpu::BindGroupEntry {
                binding: IBL_BAKE_BINDING_SOURCE_SAMPLER,
                resource: wgpu::BindingResource::Sampler(source_sampler),
            },
            wgpu::BindGroupEntry {
                binding: IBL_BAKE_BINDING_OUTPUT,
                resource: output.as_binding_resource(),
            },
        ],
    }))
}

fn create_ibl_bake_wgpu_bind_group_layout(
    device: &wgpu::Device,
    output_kind: IblBakeWgpuOutputBindingKind,
) -> wgpu::BindGroupLayout {
    let label = match output_kind {
        IblBakeWgpuOutputBindingKind::StorageTexture2DArray => {
            "zircon-env-ibl-bake-storage-texture-bind-group-layout"
        }
        IblBakeWgpuOutputBindingKind::StorageBuffer => {
            "zircon-env-ibl-bake-storage-buffer-bind-group-layout"
        }
    };
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &ibl_bake_wgpu_bind_group_layout_entries(output_kind),
    })
}

fn output_kind_from_plan(
    output: &IblBakeWgpuOutputPlan,
) -> Result<IblBakeWgpuOutputBindingKind, String> {
    match output {
        IblBakeWgpuOutputPlan::StorageTexture { view, .. } => {
            if view.dimension != wgpu::TextureViewDimension::D2Array {
                return Err(format!(
                    "IBL bake storage texture output must use D2Array view, got {:?}",
                    view.dimension
                ));
            }
            Ok(IblBakeWgpuOutputBindingKind::StorageTexture2DArray)
        }
        IblBakeWgpuOutputPlan::StorageBuffer { .. } => {
            Ok(IblBakeWgpuOutputBindingKind::StorageBuffer)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        IblBakeArtifactContents, IblBakeArtifactRequest, ProceduralSkyParams,
        IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES,
    };
    use crate::graphics::backend::RenderBackend;

    use super::super::ibl_bake_shader_plan::IblBakeComputeKernelKind;
    use super::super::ibl_bake_wgpu_command_plan::{
        ibl_bake_wgpu_command_plan_for_request, IblBakeWgpuOutputPlan,
    };
    use super::*;

    #[test]
    fn bind_groups_create_for_storage_texture_and_storage_buffer_outputs() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let device = &backend.device;
        let layouts = IblBakeWgpuBindGroupLayouts::new(device);
        let sampler = create_ibl_bake_wgpu_source_sampler(device);
        let source_texture = create_source_cubemap_texture(device);
        let source_view = source_texture.create_view(&source_cubemap_view_descriptor());

        let request = request(64, 7, IblBakeArtifactContents::PMREM_SH9_IEM);
        let plan = ibl_bake_wgpu_command_plan_for_request(&request);

        let pmrem_command = command_for_kind(
            &plan.commands,
            IblBakeComputeKernelKind::Pmrem { mip_level: 2 },
        );
        let params = create_ibl_bake_wgpu_params_buffer(device, pmrem_command);
        let output_texture = create_storage_output_texture(device, 64, 7);
        let output_view = output_texture.create_view(&storage_texture_descriptor(pmrem_command));
        let texture_bind_group = create_ibl_bake_wgpu_bind_group(
            device,
            &layouts,
            pmrem_command,
            &params,
            &source_view,
            &sampler,
            IblBakeWgpuOutputBindingResource::StorageTexture2DArray(&output_view),
        );
        assert!(texture_bind_group.is_ok());

        let sh9_command = command_for_kind(&plan.commands, IblBakeComputeKernelKind::IrradianceSh9);
        let params = create_ibl_bake_wgpu_params_buffer(device, sh9_command);
        let sh9_output = create_sh9_output_buffer(device);
        let buffer_bind_group = create_ibl_bake_wgpu_bind_group(
            device,
            &layouts,
            sh9_command,
            &params,
            &source_view,
            &sampler,
            IblBakeWgpuOutputBindingResource::StorageBuffer(&sh9_output),
        );
        assert!(buffer_bind_group.is_ok());
    }

    #[test]
    fn bind_group_creation_rejects_output_kind_mismatches_before_wgpu_validation() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let device = &backend.device;
        let layouts = IblBakeWgpuBindGroupLayouts::new(device);
        let sampler = create_ibl_bake_wgpu_source_sampler(device);
        let source_texture = create_source_cubemap_texture(device);
        let source_view = source_texture.create_view(&source_cubemap_view_descriptor());
        let sh9_output = create_sh9_output_buffer(device);

        let request = request(64, 7, IblBakeArtifactContents::PMREM_SH9);
        let plan = ibl_bake_wgpu_command_plan_for_request(&request);
        let pmrem_command = command_for_kind(
            &plan.commands,
            IblBakeComputeKernelKind::Pmrem { mip_level: 0 },
        );
        let params = create_ibl_bake_wgpu_params_buffer(device, pmrem_command);

        let result = create_ibl_bake_wgpu_bind_group(
            device,
            &layouts,
            pmrem_command,
            &params,
            &source_view,
            &sampler,
            IblBakeWgpuOutputBindingResource::StorageBuffer(&sh9_output),
        );

        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .contains("expects StorageTexture2DArray output binding"));
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

    fn command_for_kind(
        commands: &[IblBakeWgpuCommandPlan],
        kind: IblBakeComputeKernelKind,
    ) -> &IblBakeWgpuCommandPlan {
        commands
            .iter()
            .find(|command| command.kind == kind)
            .expect("requested command should be present")
    }

    fn create_source_cubemap_texture(device: &wgpu::Device) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ibl-bake-test-source-cubemap"),
            size: wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }

    fn source_cubemap_view_descriptor() -> wgpu::TextureViewDescriptor<'static> {
        wgpu::TextureViewDescriptor {
            label: Some("ibl-bake-test-source-cubemap-view"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(6),
        }
    }

    fn create_storage_output_texture(
        device: &wgpu::Device,
        face_size: u32,
        mip_count: u32,
    ) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ibl-bake-test-storage-output"),
            size: wgpu::Extent3d {
                width: face_size,
                height: face_size,
                depth_or_array_layers: 6,
            },
            mip_level_count: mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    fn storage_texture_descriptor(
        command: &IblBakeWgpuCommandPlan,
    ) -> wgpu::TextureViewDescriptor<'static> {
        let IblBakeWgpuOutputPlan::StorageTexture { view, .. } = &command.output else {
            panic!("command should write a storage texture")
        };
        (*view).to_wgpu_descriptor()
    }

    fn create_sh9_output_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ibl-bake-test-sh9-output"),
            size: IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    }
}
