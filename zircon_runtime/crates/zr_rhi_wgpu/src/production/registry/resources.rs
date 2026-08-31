use zr_rhi::{
    BufferDesc, BufferHandle, BufferUsage, RhiError, SamplerDesc, SamplerHandle, SubmissionTicket,
    TextureDesc, TextureHandle, TextureUsage, TextureViewDesc, TextureViewHandle,
    TransientAllocatorStats,
};

use crate::resource_validation::{
    texture_storage_size, validate_buffer_desc, validate_sampler_desc, validate_texture_desc,
};

use super::super::translate::{
    wgpu_address_mode, wgpu_buffer_usage, wgpu_compare_function, wgpu_filter_mode,
    wgpu_mipmap_filter_mode, wgpu_texture_dimension, wgpu_texture_format, wgpu_texture_usage,
    wgpu_texture_view_aspect, wgpu_texture_view_dimension,
};
use super::{
    WgpuBufferResource, WgpuResourceRegistry, WgpuRetiredResource, WgpuSamplerResource,
    WgpuTextureResource, WgpuTextureViewResource,
};
use crate::texture_view::validate_texture_view_desc;

impl WgpuResourceRegistry {
    pub(crate) fn create_buffer(
        &mut self,
        device: &wgpu::Device,
        desc: &BufferDesc,
    ) -> Result<BufferHandle, RhiError> {
        validate_buffer_desc(desc)?;
        validate_wgpu_buffer_usage(desc)?;
        self.ensure_buffer_capacity(desc.size_bytes)?;

        let handle = self.handles.allocate_buffer()?;
        let native = device.create_buffer(&wgpu::BufferDescriptor {
            label: desc.label.as_deref(),
            size: desc.size_bytes,
            usage: wgpu_buffer_usage(desc.usage),
            mapped_at_creation: false,
        });
        self.buffers.insert(
            handle,
            WgpuBufferResource {
                desc: desc.clone(),
                native,
                last_uses: Default::default(),
            },
        );
        Ok(handle)
    }

    pub(crate) fn buffer_desc(&self, handle: BufferHandle) -> Result<BufferDesc, RhiError> {
        self.handles.validate_buffer(handle)?;
        self.buffers
            .get(&handle)
            .map(|resource| resource.desc.clone())
            .ok_or(RhiError::UnknownBuffer(handle.diagnostic_id()))
    }

    pub(crate) fn buffer(&self, handle: BufferHandle) -> Result<&wgpu::Buffer, RhiError> {
        self.handles.validate_buffer(handle)?;
        self.buffers
            .get(&handle)
            .map(|resource| &resource.native)
            .ok_or(RhiError::UnknownBuffer(handle.diagnostic_id()))
    }

    pub(crate) fn destroy_buffer(&mut self, handle: BufferHandle) -> Result<(), RhiError> {
        self.handles.validate_buffer(handle)?;
        let mut resource = self
            .buffers
            .remove(&handle)
            .ok_or(RhiError::UnknownBuffer(handle.diagnostic_id()))?;
        self.handles.release_buffer(handle)?;
        let last_uses = std::mem::take(&mut resource.last_uses);
        self.retire_native(WgpuRetiredResource::Buffer(resource), last_uses);
        Ok(())
    }

    pub(crate) fn create_texture(
        &mut self,
        device: &wgpu::Device,
        desc: &TextureDesc,
    ) -> Result<TextureHandle, RhiError> {
        validate_texture_desc(desc, false)?;
        if desc.usage.has_unknown_bits() {
            return Err(RhiError::InvalidTextureDescriptor {
                label: desc.label.clone(),
                reason: "usage contains unknown bits".to_string(),
            });
        }
        if desc.usage.contains(TextureUsage::PRESENT) {
            return Err(RhiError::InvalidTextureDescriptor {
                label: desc.label.clone(),
                reason: "PRESENT textures must be created by the surface owner".to_string(),
            });
        }
        self.ensure_texture_capacity(texture_storage_size(desc))?;

        let handle = self.handles.allocate_texture()?;
        let view_formats: Vec<_> = desc
            .view_formats
            .iter()
            .copied()
            .map(wgpu_texture_format)
            .collect();
        let native = device.create_texture(&wgpu::TextureDescriptor {
            label: desc.label.as_deref(),
            size: wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: desc.depth,
            },
            mip_level_count: desc.mip_levels,
            sample_count: desc.sample_count,
            dimension: wgpu_texture_dimension(desc.dimension),
            format: wgpu_texture_format(desc.format),
            usage: wgpu_texture_usage(desc.usage),
            view_formats: &view_formats,
        });
        self.textures.insert(
            handle,
            WgpuTextureResource {
                desc: desc.clone(),
                native,
                last_uses: Default::default(),
            },
        );
        Ok(handle)
    }

    pub(crate) fn texture_desc(&self, handle: TextureHandle) -> Result<TextureDesc, RhiError> {
        self.handles.validate_texture(handle)?;
        self.textures
            .get(&handle)
            .map(|resource| resource.desc.clone())
            .ok_or(RhiError::UnknownTexture(handle.diagnostic_id()))
    }

    pub(crate) fn texture(&self, handle: TextureHandle) -> Result<&wgpu::Texture, RhiError> {
        self.handles.validate_texture(handle)?;
        self.textures
            .get(&handle)
            .map(|resource| &resource.native)
            .ok_or(RhiError::UnknownTexture(handle.diagnostic_id()))
    }

    pub(crate) fn destroy_texture(&mut self, handle: TextureHandle) -> Result<(), RhiError> {
        self.handles.validate_texture(handle)?;
        if self.surface_owned_textures.contains(&handle) {
            return Err(RhiError::SurfaceOwnedTexture {
                texture: handle.diagnostic_id(),
            });
        }
        let live_views = self.texture_view_counts.get(&handle).copied().unwrap_or(0);
        if live_views != 0 {
            return Err(RhiError::TextureHasLiveViews {
                texture: handle.diagnostic_id(),
                live_views,
            });
        }
        let mut resource = self
            .textures
            .remove(&handle)
            .ok_or(RhiError::UnknownTexture(handle.diagnostic_id()))?;
        self.handles.release_texture(handle)?;
        let last_uses = std::mem::take(&mut resource.last_uses);
        self.retire_native(WgpuRetiredResource::Texture(resource), last_uses);
        Ok(())
    }

    pub(crate) fn create_texture_view(
        &mut self,
        desc: &TextureViewDesc,
    ) -> Result<TextureViewHandle, RhiError> {
        self.handles.validate_texture(desc.texture)?;
        if self.surface_owned_textures.contains(&desc.texture) {
            return Err(RhiError::SurfaceOwnedTexture {
                texture: desc.texture.diagnostic_id(),
            });
        }
        let texture_desc = self.texture_desc(desc.texture)?;
        validate_texture_view_desc(&texture_desc, desc)?;
        let native = self
            .texture(desc.texture)?
            .create_view(&wgpu::TextureViewDescriptor {
                label: desc.label.as_deref(),
                format: desc.format.map(wgpu_texture_format),
                aspect: wgpu_texture_view_aspect(desc.aspect),
                dimension: Some(wgpu_texture_view_dimension(desc.dimension)),
                base_mip_level: desc.base_mip_level,
                mip_level_count: desc.mip_level_count,
                base_array_layer: desc.base_array_layer,
                array_layer_count: desc.array_layer_count,
                ..Default::default()
            });
        let handle = self.handles.allocate_texture_view()?;
        self.texture_views.insert(
            handle,
            WgpuTextureViewResource {
                desc: desc.clone(),
                native,
                last_uses: Default::default(),
            },
        );
        let count = self.texture_view_counts.entry(desc.texture).or_insert(0);
        *count = count.saturating_add(1);
        Ok(handle)
    }

    pub(crate) fn texture_view_desc(
        &self,
        handle: TextureViewHandle,
    ) -> Result<TextureViewDesc, RhiError> {
        self.handles.validate_texture_view(handle)?;
        self.texture_views
            .get(&handle)
            .map(|resource| resource.desc.clone())
            .ok_or(RhiError::UnknownTextureView(handle.diagnostic_id()))
    }

    pub(crate) fn texture_view(
        &self,
        handle: TextureViewHandle,
    ) -> Result<&wgpu::TextureView, RhiError> {
        self.handles.validate_texture_view(handle)?;
        self.texture_views
            .get(&handle)
            .map(|resource| &resource.native)
            .ok_or(RhiError::UnknownTextureView(handle.diagnostic_id()))
    }

    pub(crate) fn destroy_texture_view(
        &mut self,
        handle: TextureViewHandle,
    ) -> Result<(), RhiError> {
        self.handles.validate_texture_view(handle)?;
        if self.surface_owned_texture_views.contains(&handle) {
            return Err(RhiError::SurfaceOwnedTextureView {
                view: handle.diagnostic_id(),
            });
        }
        let mut resource = self
            .texture_views
            .remove(&handle)
            .ok_or(RhiError::UnknownTextureView(handle.diagnostic_id()))?;
        decrement_texture_view_count(&mut self.texture_view_counts, resource.desc.texture);
        self.handles.release_texture_view(handle)?;
        let last_uses = std::mem::take(&mut resource.last_uses);
        self.retire_native(WgpuRetiredResource::TextureView(resource), last_uses);
        Ok(())
    }

    pub(crate) fn create_sampler(
        &mut self,
        device: &wgpu::Device,
        desc: &SamplerDesc,
    ) -> Result<SamplerHandle, RhiError> {
        validate_sampler_desc(desc)?;

        let handle = self.handles.allocate_sampler()?;
        let native = device.create_sampler(&wgpu::SamplerDescriptor {
            label: desc.label.as_deref(),
            address_mode_u: wgpu_address_mode(desc.address_mode_u),
            address_mode_v: wgpu_address_mode(desc.address_mode_v),
            address_mode_w: wgpu_address_mode(desc.address_mode_w),
            mag_filter: wgpu_filter_mode(desc.mag_filter),
            min_filter: wgpu_filter_mode(desc.min_filter),
            mipmap_filter: wgpu_mipmap_filter_mode(desc.mipmap_filter),
            lod_min_clamp: desc.lod_min_clamp,
            lod_max_clamp: desc.lod_max_clamp,
            compare: desc.compare.map(wgpu_compare_function),
            anisotropy_clamp: desc.anisotropy_clamp,
            border_color: None,
        });
        self.samplers.insert(
            handle,
            WgpuSamplerResource {
                desc: desc.clone(),
                native,
                last_uses: Default::default(),
            },
        );
        Ok(handle)
    }

    pub(crate) fn sampler_desc(&self, handle: SamplerHandle) -> Result<SamplerDesc, RhiError> {
        self.handles.validate_sampler(handle)?;
        self.samplers
            .get(&handle)
            .map(|resource| resource.desc.clone())
            .ok_or(RhiError::UnknownSampler(handle.diagnostic_id()))
    }

    pub(crate) fn sampler(&self, handle: SamplerHandle) -> Result<&wgpu::Sampler, RhiError> {
        self.handles.validate_sampler(handle)?;
        self.samplers
            .get(&handle)
            .map(|resource| &resource.native)
            .ok_or(RhiError::UnknownSampler(handle.diagnostic_id()))
    }

    pub(crate) fn destroy_sampler(&mut self, handle: SamplerHandle) -> Result<(), RhiError> {
        self.handles.validate_sampler(handle)?;
        let mut resource = self
            .samplers
            .remove(&handle)
            .ok_or(RhiError::UnknownSampler(handle.diagnostic_id()))?;
        self.handles.release_sampler(handle)?;
        let last_uses = std::mem::take(&mut resource.last_uses);
        self.retire_native(WgpuRetiredResource::Sampler(resource), last_uses);
        Ok(())
    }

    pub(crate) fn transient_allocator_stats(&self) -> TransientAllocatorStats {
        self.physical_allocator_stats()
    }

    /// Registers one acquired surface texture and its default view under the
    /// normal generational resource table. The surface owner alone may later
    /// release these handles through present or discard.
    pub(crate) fn register_surface_frame(
        &mut self,
        desc: TextureDesc,
        native: wgpu::Texture,
        default_view: wgpu::TextureView,
    ) -> Result<(TextureHandle, TextureViewHandle), RhiError> {
        validate_texture_desc(&desc, false)?;
        if !desc.usage.contains(TextureUsage::PRESENT)
            || !desc.usage.contains(TextureUsage::RENDER_ATTACHMENT)
        {
            return Err(RhiError::InvalidTextureDescriptor {
                label: desc.label.clone(),
                reason: "surface frames require PRESENT and RENDER_ATTACHMENT usage".to_string(),
            });
        }
        self.ensure_texture_capacity(texture_storage_size(&desc))?;
        let texture = self.handles.allocate_texture()?;
        let view = match self.handles.allocate_texture_view() {
            Ok(view) => view,
            Err(error) => {
                let _ = self.handles.release_texture(texture);
                return Err(error.into());
            }
        };
        let view_desc = TextureViewDesc::new(
            "zircon-surface-frame-default-view",
            texture,
            zr_rhi::TextureViewDimension::D2,
        );
        self.textures.insert(
            texture,
            WgpuTextureResource {
                desc,
                native,
                last_uses: Vec::new(),
            },
        );
        self.texture_views.insert(
            view,
            WgpuTextureViewResource {
                desc: view_desc,
                native: default_view,
                last_uses: Vec::new(),
            },
        );
        self.texture_view_counts.insert(texture, 1);
        self.surface_owned_textures.insert(texture);
        self.surface_owned_texture_views.insert(view);
        self.surface_frame_submissions
            .insert(texture, Default::default());
        Ok((texture, view))
    }

    pub(crate) fn surface_frame_has_submission(
        &self,
        texture: TextureHandle,
        submission: SubmissionTicket,
    ) -> Result<bool, RhiError> {
        self.handles.validate_texture(texture)?;
        if !self.surface_owned_textures.contains(&texture) {
            return Err(RhiError::SurfaceOwnedTexture {
                texture: texture.diagnostic_id(),
            });
        }
        Ok(self
            .surface_frame_submissions
            .get(&texture)
            .is_some_and(|submissions| submissions.contains(&submission)))
    }

    pub(crate) fn surface_frame_submission_tickets(
        &self,
        texture: TextureHandle,
    ) -> Result<Vec<SubmissionTicket>, RhiError> {
        self.handles.validate_texture(texture)?;
        if !self.surface_owned_textures.contains(&texture) {
            return Err(RhiError::SurfaceOwnedTexture {
                texture: texture.diagnostic_id(),
            });
        }
        let mut tickets: Vec<_> = self
            .surface_frame_submissions
            .get(&texture)
            .map(|submissions| submissions.iter().copied().collect())
            .unwrap_or_default();
        tickets.sort_by_key(|ticket| ticket.sequence());
        Ok(tickets)
    }

    /// Retires an acquired frame's registry entries. Native references remain
    /// retained until every command packet that mentioned the target reaches a
    /// terminal state, while the public handles become stale immediately.
    pub(crate) fn release_surface_frame(
        &mut self,
        texture_handle: TextureHandle,
        default_view: TextureViewHandle,
    ) -> Result<(), RhiError> {
        self.handles.validate_texture(texture_handle)?;
        self.handles.validate_texture_view(default_view)?;
        if !self.surface_owned_textures.remove(&texture_handle) {
            return Err(RhiError::SurfaceOwnedTexture {
                texture: texture_handle.diagnostic_id(),
            });
        }
        if !self.surface_owned_texture_views.remove(&default_view) {
            return Err(RhiError::SurfaceOwnedTextureView {
                view: default_view.diagnostic_id(),
            });
        }
        self.surface_frame_submissions.remove(&texture_handle);
        let mut view = self
            .texture_views
            .remove(&default_view)
            .ok_or(RhiError::UnknownTextureView(default_view.diagnostic_id()))?;
        decrement_texture_view_count(&mut self.texture_view_counts, texture_handle);
        self.handles.release_texture_view(default_view)?;
        let view_last_uses = std::mem::take(&mut view.last_uses);
        self.retire_native(WgpuRetiredResource::TextureView(view), view_last_uses);
        let mut texture_resource = self
            .textures
            .remove(&texture_handle)
            .ok_or(RhiError::UnknownTexture(texture_handle.diagnostic_id()))?;
        self.handles.release_texture(texture_handle)?;
        let texture_last_uses = std::mem::take(&mut texture_resource.last_uses);
        self.retire_native(
            WgpuRetiredResource::Texture(texture_resource),
            texture_last_uses,
        );
        Ok(())
    }
}

fn decrement_texture_view_count(
    texture_view_counts: &mut std::collections::HashMap<TextureHandle, u32>,
    texture: TextureHandle,
) {
    let Some(count) = texture_view_counts.get_mut(&texture) else {
        return;
    };
    if *count <= 1 {
        texture_view_counts.remove(&texture);
    } else {
        *count -= 1;
    }
}

impl crate::render_pass_validation::RenderPassResourceLookup for WgpuResourceRegistry {
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
}

/// WGPU's mapped buffer modes are deliberately narrow: map-read buffers are
/// readback copies and map-write buffers are upload sources. Keeping this
/// check at the native boundary prevents the test-only CPU mirror from
/// accidentally defining a broader production contract.
pub(in crate::production) fn validate_wgpu_buffer_usage(desc: &BufferDesc) -> Result<(), RhiError> {
    if desc.usage.has_unknown_bits() {
        return Err(RhiError::InvalidBufferDescriptor {
            label: desc.label.clone(),
            reason: "usage contains unknown bits".to_string(),
        });
    }

    let usage_bits = desc.usage.bits();
    if desc.usage.contains(BufferUsage::STAGING_READ)
        && usage_bits != (BufferUsage::STAGING_READ | BufferUsage::COPY_DST).bits()
    {
        return Err(RhiError::InvalidBufferDescriptor {
            label: desc.label.clone(),
            reason: "STAGING_READ may only be combined with COPY_DST on WGPU".to_string(),
        });
    }
    if desc.usage.contains(BufferUsage::STAGING_WRITE)
        && usage_bits != (BufferUsage::STAGING_WRITE | BufferUsage::COPY_SRC).bits()
    {
        return Err(RhiError::InvalidBufferDescriptor {
            label: desc.label.clone(),
            reason: "STAGING_WRITE may only be combined with COPY_SRC on WGPU".to_string(),
        });
    }

    Ok(())
}
