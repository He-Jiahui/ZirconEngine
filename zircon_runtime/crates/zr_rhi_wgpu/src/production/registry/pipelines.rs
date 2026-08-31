use zr_rhi::{
    BindGroupLayoutHandle, PipelineDesc, PipelineHandle, PipelineKind, PipelineLayoutDesc,
    PipelineLayoutHandle, RhiError, ShaderModuleDesc, ShaderModuleHandle,
};

use crate::pipeline_validation::{
    validate_pipeline_desc, validate_pipeline_layout_desc, validate_shader_module_desc,
    PipelineResourceLookup,
};

use super::{
    WgpuPipelineLayoutResource, WgpuPipelineResource, WgpuResourceRegistry, WgpuRetiredResource,
    WgpuShaderModuleResource,
};

impl WgpuResourceRegistry {
    pub(crate) fn create_shader_module(
        &mut self,
        device: &wgpu::Device,
        desc: &ShaderModuleDesc,
    ) -> Result<ShaderModuleHandle, RhiError> {
        validate_shader_module_desc(desc)?;
        let handle = self.handles.allocate_shader_module()?;
        let native = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: desc.label.as_deref(),
            source: wgpu::ShaderSource::Wgsl(desc.source.clone().into()),
        });
        self.shader_modules.insert(
            handle,
            WgpuShaderModuleResource {
                desc: desc.clone(),
                native,
                last_uses: Default::default(),
            },
        );
        Ok(handle)
    }

    pub(crate) fn shader_module_desc(
        &self,
        handle: ShaderModuleHandle,
    ) -> Result<ShaderModuleDesc, RhiError> {
        self.handles.validate_shader_module(handle)?;
        self.shader_modules
            .get(&handle)
            .map(|resource| resource.desc.clone())
            .ok_or(RhiError::UnknownShaderModule(handle.diagnostic_id()))
    }

    pub(crate) fn shader_module(
        &self,
        handle: ShaderModuleHandle,
    ) -> Result<&wgpu::ShaderModule, RhiError> {
        self.handles.validate_shader_module(handle)?;
        self.shader_modules
            .get(&handle)
            .map(|resource| &resource.native)
            .ok_or(RhiError::UnknownShaderModule(handle.diagnostic_id()))
    }

    pub(crate) fn destroy_shader_module(
        &mut self,
        handle: ShaderModuleHandle,
    ) -> Result<(), RhiError> {
        self.handles.validate_shader_module(handle)?;
        let mut resource = self
            .shader_modules
            .remove(&handle)
            .ok_or(RhiError::UnknownShaderModule(handle.diagnostic_id()))?;
        self.handles.release_shader_module(handle)?;
        let last_uses = std::mem::take(&mut resource.last_uses);
        self.retire_native(WgpuRetiredResource::ShaderModule(resource), last_uses);
        Ok(())
    }

    pub(crate) fn create_pipeline_layout(
        &mut self,
        device: &wgpu::Device,
        desc: &PipelineLayoutDesc,
    ) -> Result<PipelineLayoutHandle, RhiError> {
        for handle in &desc.bind_group_layouts {
            self.handles.validate_bind_group_layout(*handle)?;
        }
        validate_pipeline_layout_desc(self, desc)?;
        let native = {
            let layouts = desc
                .bind_group_layouts
                .iter()
                .map(|handle| self.bind_group_layout(*handle))
                .collect::<Result<Vec<_>, _>>()?;
            let layouts = layouts.into_iter().map(Some).collect::<Vec<_>>();
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: desc.label.as_deref(),
                bind_group_layouts: &layouts,
                immediate_size: 0,
            })
        };
        let handle = self.handles.allocate_pipeline_layout()?;
        self.pipeline_layouts.insert(
            handle,
            WgpuPipelineLayoutResource {
                desc: desc.clone(),
                native,
                last_uses: Default::default(),
            },
        );
        Ok(handle)
    }

    pub(crate) fn pipeline_layout_desc(
        &self,
        handle: PipelineLayoutHandle,
    ) -> Result<PipelineLayoutDesc, RhiError> {
        self.handles.validate_pipeline_layout(handle)?;
        self.pipeline_layouts
            .get(&handle)
            .map(|resource| resource.desc.clone())
            .ok_or(RhiError::UnknownPipelineLayout(handle.diagnostic_id()))
    }

    pub(crate) fn pipeline_layout(
        &self,
        handle: PipelineLayoutHandle,
    ) -> Result<&wgpu::PipelineLayout, RhiError> {
        self.handles.validate_pipeline_layout(handle)?;
        self.pipeline_layouts
            .get(&handle)
            .map(|resource| &resource.native)
            .ok_or(RhiError::UnknownPipelineLayout(handle.diagnostic_id()))
    }

    pub(crate) fn destroy_pipeline_layout(
        &mut self,
        handle: PipelineLayoutHandle,
    ) -> Result<(), RhiError> {
        self.handles.validate_pipeline_layout(handle)?;
        let mut resource = self
            .pipeline_layouts
            .remove(&handle)
            .ok_or(RhiError::UnknownPipelineLayout(handle.diagnostic_id()))?;
        self.handles.release_pipeline_layout(handle)?;
        let last_uses = std::mem::take(&mut resource.last_uses);
        self.retire_native(WgpuRetiredResource::PipelineLayout(resource), last_uses);
        Ok(())
    }

    pub(crate) fn create_pipeline(
        &mut self,
        device: &wgpu::Device,
        desc: &PipelineDesc,
    ) -> Result<PipelineHandle, RhiError> {
        validate_pipeline_desc(self, desc)?;
        let native = match desc.kind {
            PipelineKind::Raster => WgpuPipelineResource::Raster {
                desc: desc.clone(),
                native: super::super::pipeline::create_raster_pipeline(device, self, desc)?,
                last_uses: Default::default(),
            },
            PipelineKind::Compute => WgpuPipelineResource::Compute {
                desc: desc.clone(),
                native: super::super::pipeline::create_compute_pipeline(device, self, desc)?,
                last_uses: Default::default(),
            },
            PipelineKind::RayTracing => {
                return Err(RhiError::InvalidPipelineDescriptor {
                    label: desc.label.clone(),
                    reason: "ray tracing is not implemented by the WGPU production backend"
                        .to_string(),
                });
            }
        };
        let handle = self.handles.allocate_pipeline()?;
        self.pipelines.insert(handle, native);
        Ok(handle)
    }

    pub(crate) fn pipeline_desc(&self, handle: PipelineHandle) -> Result<PipelineDesc, RhiError> {
        self.handles.validate_pipeline(handle)?;
        self.pipelines
            .get(&handle)
            .map(WgpuPipelineResource::desc)
            .cloned()
            .ok_or(RhiError::UnknownPipeline(handle.diagnostic_id()))
    }

    pub(crate) fn render_pipeline(
        &self,
        handle: PipelineHandle,
    ) -> Result<&wgpu::RenderPipeline, RhiError> {
        self.handles.validate_pipeline(handle)?;
        match self.pipelines.get(&handle) {
            Some(WgpuPipelineResource::Raster { native, .. }) => Ok(native),
            Some(WgpuPipelineResource::Compute { desc, .. }) => {
                Err(RhiError::InvalidPipelineUsage {
                    pipeline: handle.diagnostic_id(),
                    required: PipelineKind::Raster,
                    actual: desc.kind,
                })
            }
            None => Err(RhiError::UnknownPipeline(handle.diagnostic_id())),
        }
    }

    pub(crate) fn compute_pipeline(
        &self,
        handle: PipelineHandle,
    ) -> Result<&wgpu::ComputePipeline, RhiError> {
        self.handles.validate_pipeline(handle)?;
        match self.pipelines.get(&handle) {
            Some(WgpuPipelineResource::Compute { native, .. }) => Ok(native),
            Some(WgpuPipelineResource::Raster { desc, .. }) => {
                Err(RhiError::InvalidPipelineUsage {
                    pipeline: handle.diagnostic_id(),
                    required: PipelineKind::Compute,
                    actual: desc.kind,
                })
            }
            None => Err(RhiError::UnknownPipeline(handle.diagnostic_id())),
        }
    }

    pub(crate) fn destroy_pipeline(&mut self, handle: PipelineHandle) -> Result<(), RhiError> {
        self.handles.validate_pipeline(handle)?;
        let mut resource = self
            .pipelines
            .remove(&handle)
            .ok_or(RhiError::UnknownPipeline(handle.diagnostic_id()))?;
        self.handles.release_pipeline(handle)?;
        let last_uses = std::mem::take(resource.last_uses_mut());
        self.retire_native(WgpuRetiredResource::Pipeline(resource), last_uses);
        Ok(())
    }
}

impl PipelineResourceLookup for WgpuResourceRegistry {
    fn bind_group_layout_exists(&self, handle: BindGroupLayoutHandle) -> bool {
        self.handles.validate_bind_group_layout(handle).is_ok()
            && self.bind_group_layouts.contains_key(&handle)
    }

    fn pipeline_layout_desc(
        &self,
        handle: PipelineLayoutHandle,
    ) -> Result<&PipelineLayoutDesc, RhiError> {
        self.handles.validate_pipeline_layout(handle)?;
        self.pipeline_layouts
            .get(&handle)
            .map(|resource| &resource.desc)
            .ok_or(RhiError::UnknownPipelineLayout(handle.diagnostic_id()))
    }

    fn shader_module_desc(
        &self,
        handle: ShaderModuleHandle,
    ) -> Result<&ShaderModuleDesc, RhiError> {
        self.handles.validate_shader_module(handle)?;
        self.shader_modules
            .get(&handle)
            .map(|resource| &resource.desc)
            .ok_or(RhiError::UnknownShaderModule(handle.diagnostic_id()))
    }
}
