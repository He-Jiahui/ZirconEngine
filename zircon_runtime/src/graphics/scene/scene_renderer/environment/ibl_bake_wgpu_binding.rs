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

/// 把烘焙命令的输出种类落实为 GPU 绑定，提前拒绝纹理/缓冲区契约不匹配。
/// 图执行应传编译器分配的缓冲区窗口；捕获自有 SH9 可绑定整个缓冲区。
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
#[path = "tests/ibl_bake_wgpu_binding.rs"]
mod tests;
