use zr_rhi::{
    BindGroupDesc, BindGroupHandle, BindGroupLayoutDesc, BindGroupLayoutEntryDesc,
    BindGroupLayoutHandle, BindingResourceType, BufferDesc, BufferHandle, RhiError, SamplerDesc,
    SamplerHandle, ShaderStage, TextureDesc, TextureHandle, TextureViewDesc, TextureViewHandle,
};

use super::super::translate::{
    wgpu_sampler_binding_type, wgpu_storage_texture_access, wgpu_texture_format,
    wgpu_texture_sample_type, wgpu_texture_view_dimension,
};
use crate::bind_group_validation::{validate_bind_group_desc, BindGroupResourceLookup};
use crate::resource_validation::validate_bind_group_layout_desc;

use super::{
    WgpuBindGroupLayoutResource, WgpuBindGroupResource, WgpuResourceRegistry, WgpuRetiredResource,
};

impl WgpuResourceRegistry {
    pub(crate) fn create_bind_group_layout(
        &mut self,
        device: &wgpu::Device,
        desc: &BindGroupLayoutDesc,
    ) -> Result<BindGroupLayoutHandle, RhiError> {
        validate_bind_group_layout_desc(desc)?;
        let entries = desc
            .entries
            .iter()
            .map(|entry| wgpu_bind_group_layout_entry(desc, entry))
            .collect::<Result<Vec<_>, _>>()?;
        let handle = self.handles.allocate_bind_group_layout()?;
        let native = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: desc.label.as_deref(),
            entries: &entries,
        });
        self.bind_group_layouts.insert(
            handle,
            WgpuBindGroupLayoutResource {
                desc: desc.clone(),
                native,
                last_uses: Default::default(),
            },
        );
        Ok(handle)
    }

    pub(crate) fn bind_group_layout_desc(
        &self,
        handle: BindGroupLayoutHandle,
    ) -> Result<BindGroupLayoutDesc, RhiError> {
        self.handles.validate_bind_group_layout(handle)?;
        self.bind_group_layouts
            .get(&handle)
            .map(|resource| resource.desc.clone())
            .ok_or(RhiError::UnknownBindGroupLayout(handle.diagnostic_id()))
    }

    pub(crate) fn bind_group_layout(
        &self,
        handle: BindGroupLayoutHandle,
    ) -> Result<&wgpu::BindGroupLayout, RhiError> {
        self.handles.validate_bind_group_layout(handle)?;
        self.bind_group_layouts
            .get(&handle)
            .map(|resource| &resource.native)
            .ok_or(RhiError::UnknownBindGroupLayout(handle.diagnostic_id()))
    }

    pub(crate) fn destroy_bind_group_layout(
        &mut self,
        handle: BindGroupLayoutHandle,
    ) -> Result<(), RhiError> {
        self.handles.validate_bind_group_layout(handle)?;
        let mut resource = self
            .bind_group_layouts
            .remove(&handle)
            .ok_or(RhiError::UnknownBindGroupLayout(handle.diagnostic_id()))?;
        self.handles.release_bind_group_layout(handle)?;
        let last_uses = std::mem::take(&mut resource.last_uses);
        self.retire_native(WgpuRetiredResource::BindGroupLayout(resource), last_uses);
        Ok(())
    }

    pub(crate) fn create_bind_group(
        &mut self,
        device: &wgpu::Device,
        desc: &BindGroupDesc,
    ) -> Result<BindGroupHandle, RhiError> {
        validate_bind_group_desc(self, desc)?;
        let native = {
            let layout = self.bind_group_layout(desc.layout)?;
            let entries = desc
                .entries
                .iter()
                .map(|entry| {
                    super::super::binding::wgpu_bind_group_entry(
                        self,
                        entry.resource.clone(),
                        entry.binding,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: desc.label.as_deref(),
                layout,
                entries: &entries,
            })
        };
        let handle = self.handles.allocate_bind_group()?;
        self.bind_groups.insert(
            handle,
            WgpuBindGroupResource {
                desc: desc.clone(),
                native,
                last_uses: Default::default(),
            },
        );
        Ok(handle)
    }

    pub(crate) fn bind_group_desc(
        &self,
        handle: BindGroupHandle,
    ) -> Result<BindGroupDesc, RhiError> {
        self.handles.validate_bind_group(handle)?;
        self.bind_groups
            .get(&handle)
            .map(|resource| resource.desc.clone())
            .ok_or(RhiError::UnknownBindGroup(handle.diagnostic_id()))
    }

    pub(crate) fn bind_group(&self, handle: BindGroupHandle) -> Result<&wgpu::BindGroup, RhiError> {
        self.handles.validate_bind_group(handle)?;
        self.bind_groups
            .get(&handle)
            .map(|resource| &resource.native)
            .ok_or(RhiError::UnknownBindGroup(handle.diagnostic_id()))
    }

    pub(crate) fn destroy_bind_group(&mut self, handle: BindGroupHandle) -> Result<(), RhiError> {
        self.handles.validate_bind_group(handle)?;
        let mut resource = self
            .bind_groups
            .remove(&handle)
            .ok_or(RhiError::UnknownBindGroup(handle.diagnostic_id()))?;
        self.handles.release_bind_group(handle)?;
        let last_uses = std::mem::take(&mut resource.last_uses);
        self.retire_native(WgpuRetiredResource::BindGroup(resource), last_uses);
        Ok(())
    }
}

impl BindGroupResourceLookup for WgpuResourceRegistry {
    fn layout_desc(&self, handle: BindGroupLayoutHandle) -> Result<&BindGroupLayoutDesc, RhiError> {
        self.handles.validate_bind_group_layout(handle)?;
        self.bind_group_layouts
            .get(&handle)
            .map(|resource| &resource.desc)
            .ok_or(RhiError::UnknownBindGroupLayout(handle.diagnostic_id()))
    }

    fn buffer_desc(&self, handle: BufferHandle) -> Result<&BufferDesc, RhiError> {
        self.handles.validate_buffer(handle)?;
        self.buffers
            .get(&handle)
            .map(|resource| &resource.desc)
            .ok_or(RhiError::UnknownBuffer(handle.diagnostic_id()))
    }

    fn texture_desc(&self, handle: TextureHandle) -> Result<&TextureDesc, RhiError> {
        self.handles.validate_texture(handle)?;
        self.textures
            .get(&handle)
            .map(|resource| &resource.desc)
            .ok_or(RhiError::UnknownTexture(handle.diagnostic_id()))
    }

    fn texture_view_desc(&self, handle: TextureViewHandle) -> Result<&TextureViewDesc, RhiError> {
        self.handles.validate_texture_view(handle)?;
        self.texture_views
            .get(&handle)
            .map(|resource| &resource.desc)
            .ok_or(RhiError::UnknownTextureView(handle.diagnostic_id()))
    }

    fn sampler_desc(&self, handle: SamplerHandle) -> Result<&SamplerDesc, RhiError> {
        self.handles.validate_sampler(handle)?;
        self.samplers
            .get(&handle)
            .map(|resource| &resource.desc)
            .ok_or(RhiError::UnknownSampler(handle.diagnostic_id()))
    }
}

fn wgpu_bind_group_layout_entry(
    desc: &BindGroupLayoutDesc,
    entry: &BindGroupLayoutEntryDesc,
) -> Result<wgpu::BindGroupLayoutEntry, RhiError> {
    let ty = match entry.resource_type {
        BindingResourceType::UniformBuffer => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: entry.has_dynamic_offset,
            min_binding_size: entry.min_binding_size.and_then(std::num::NonZeroU64::new),
        },
        BindingResourceType::StorageBuffer => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: entry.has_dynamic_offset,
            min_binding_size: entry.min_binding_size.and_then(std::num::NonZeroU64::new),
        },
        BindingResourceType::SampledTexture {
            sample_type,
            view_dimension,
            multisampled,
        } => wgpu::BindingType::Texture {
            sample_type: wgpu_texture_sample_type(sample_type),
            view_dimension: wgpu_texture_view_dimension(view_dimension),
            multisampled,
        },
        BindingResourceType::Sampler(binding_type) => {
            wgpu::BindingType::Sampler(wgpu_sampler_binding_type(binding_type))
        }
        BindingResourceType::StorageTexture(storage) => wgpu::BindingType::StorageTexture {
            access: wgpu_storage_texture_access(storage.access),
            format: wgpu_texture_format(storage.format),
            view_dimension: wgpu_texture_view_dimension(storage.view_dimension),
        },
    };
    let mut visibility = wgpu::ShaderStages::empty();
    for stage in &entry.visibility {
        visibility |= match stage {
            ShaderStage::Vertex => wgpu::ShaderStages::VERTEX,
            ShaderStage::Fragment => wgpu::ShaderStages::FRAGMENT,
            ShaderStage::Compute => wgpu::ShaderStages::COMPUTE,
        };
    }
    Ok(wgpu::BindGroupLayoutEntry {
        binding: entry.binding,
        visibility,
        ty,
        count: None,
    })
}
