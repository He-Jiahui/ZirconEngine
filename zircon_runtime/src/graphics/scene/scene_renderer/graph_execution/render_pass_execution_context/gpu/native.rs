use std::cell::Cell;

use wgpu::util::DeviceExt;

use crate::core::framework::render::RenderPassNativeResourceCreateMetrics;

/// Resource creation surface allowed while an external graph pass is being recorded.
///
/// The production implementation counts every create in the pass profile. A raw `wgpu::Device`
/// implements the same surface for isolated shader tests without manufacturing a graph context.
pub trait RenderPassGpuResourceFactory {
    fn create_buffer_init(&self, descriptor: &wgpu::util::BufferInitDescriptor<'_>)
        -> wgpu::Buffer;

    fn create_bind_group(&self, descriptor: &wgpu::BindGroupDescriptor<'_>) -> wgpu::BindGroup;

    fn create_bind_group_layout(
        &self,
        descriptor: &wgpu::BindGroupLayoutDescriptor<'_>,
    ) -> wgpu::BindGroupLayout;

    fn create_shader_module(
        &self,
        descriptor: wgpu::ShaderModuleDescriptor<'_>,
    ) -> wgpu::ShaderModule;

    fn create_pipeline_layout(
        &self,
        descriptor: &wgpu::PipelineLayoutDescriptor<'_>,
    ) -> wgpu::PipelineLayout;

    fn create_compute_pipeline(
        &self,
        descriptor: &wgpu::ComputePipelineDescriptor<'_>,
    ) -> wgpu::ComputePipeline;

    fn create_render_pipeline(
        &self,
        descriptor: &wgpu::RenderPipelineDescriptor<'_>,
    ) -> wgpu::RenderPipeline;
}

/// Paired resource-creation and command-encoding capability for one graph pass operation.
pub trait RenderPassGpuRecordingContext {
    type ResourceFactory: RenderPassGpuResourceFactory + ?Sized;

    fn resource_factory(&self) -> &Self::ResourceFactory;
    fn command_encoder(&mut self) -> &mut wgpu::CommandEncoder;
}

impl RenderPassGpuResourceFactory for wgpu::Device {
    fn create_buffer_init(
        &self,
        descriptor: &wgpu::util::BufferInitDescriptor<'_>,
    ) -> wgpu::Buffer {
        DeviceExt::create_buffer_init(self, descriptor)
    }

    fn create_bind_group(&self, descriptor: &wgpu::BindGroupDescriptor<'_>) -> wgpu::BindGroup {
        wgpu::Device::create_bind_group(self, descriptor)
    }

    fn create_bind_group_layout(
        &self,
        descriptor: &wgpu::BindGroupLayoutDescriptor<'_>,
    ) -> wgpu::BindGroupLayout {
        wgpu::Device::create_bind_group_layout(self, descriptor)
    }

    fn create_shader_module(
        &self,
        descriptor: wgpu::ShaderModuleDescriptor<'_>,
    ) -> wgpu::ShaderModule {
        wgpu::Device::create_shader_module(self, descriptor)
    }

    fn create_pipeline_layout(
        &self,
        descriptor: &wgpu::PipelineLayoutDescriptor<'_>,
    ) -> wgpu::PipelineLayout {
        wgpu::Device::create_pipeline_layout(self, descriptor)
    }

    fn create_compute_pipeline(
        &self,
        descriptor: &wgpu::ComputePipelineDescriptor<'_>,
    ) -> wgpu::ComputePipeline {
        wgpu::Device::create_compute_pipeline(self, descriptor)
    }

    fn create_render_pipeline(
        &self,
        descriptor: &wgpu::RenderPipelineDescriptor<'_>,
    ) -> wgpu::RenderPipeline {
        wgpu::Device::create_render_pipeline(self, descriptor)
    }
}

/// Short-lived native recording capability for a graph pass.
///
/// The parent GPU context owns the frame and graph resources. External render features may borrow
/// this capability while encoding commands, but they cannot retain the parent context's resource
/// table or output mailbox through public fields.
pub struct RenderPassGpuNativeContext<'a, 'encoder> {
    device: &'a wgpu::Device,
    pub encoder: &'encoder mut wgpu::CommandEncoder,
    pub scene_bind_group: &'a wgpu::BindGroup,
    pub scene_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub(super) native_resource_creates: &'encoder Cell<RenderPassNativeResourceCreateMetrics>,
}

impl<'device, 'encoder> RenderPassGpuRecordingContext
    for RenderPassGpuNativeContext<'device, 'encoder>
{
    type ResourceFactory = RenderPassGpuNativeContext<'device, 'encoder>;

    fn resource_factory(&self) -> &Self::ResourceFactory {
        self
    }

    fn command_encoder(&mut self) -> &mut wgpu::CommandEncoder {
        self.encoder
    }
}

#[cfg(test)]
impl<'device, 'encoder> RenderPassGpuRecordingContext
    for (&'device wgpu::Device, &'encoder mut wgpu::CommandEncoder)
{
    type ResourceFactory = wgpu::Device;

    fn resource_factory(&self) -> &Self::ResourceFactory {
        self.0
    }

    fn command_encoder(&mut self) -> &mut wgpu::CommandEncoder {
        self.1
    }
}

impl RenderPassGpuNativeContext<'_, '_> {
    fn record_create(&self, record: impl FnOnce(&mut RenderPassNativeResourceCreateMetrics)) {
        let mut metrics = self.native_resource_creates.get();
        record(&mut metrics);
        self.native_resource_creates.set(metrics);
    }
}

impl RenderPassGpuResourceFactory for RenderPassGpuNativeContext<'_, '_> {
    fn create_buffer_init(
        &self,
        descriptor: &wgpu::util::BufferInitDescriptor<'_>,
    ) -> wgpu::Buffer {
        self.record_create(RenderPassNativeResourceCreateMetrics::record_buffer);
        self.device.create_buffer_init(descriptor)
    }

    fn create_bind_group(&self, descriptor: &wgpu::BindGroupDescriptor<'_>) -> wgpu::BindGroup {
        self.record_create(RenderPassNativeResourceCreateMetrics::record_bind_group);
        self.device.create_bind_group(descriptor)
    }

    fn create_bind_group_layout(
        &self,
        descriptor: &wgpu::BindGroupLayoutDescriptor<'_>,
    ) -> wgpu::BindGroupLayout {
        self.record_create(RenderPassNativeResourceCreateMetrics::record_bind_group_layout);
        self.device.create_bind_group_layout(descriptor)
    }

    fn create_shader_module(
        &self,
        descriptor: wgpu::ShaderModuleDescriptor<'_>,
    ) -> wgpu::ShaderModule {
        self.record_create(RenderPassNativeResourceCreateMetrics::record_shader_module);
        self.device.create_shader_module(descriptor)
    }

    fn create_pipeline_layout(
        &self,
        descriptor: &wgpu::PipelineLayoutDescriptor<'_>,
    ) -> wgpu::PipelineLayout {
        self.record_create(RenderPassNativeResourceCreateMetrics::record_pipeline_layout);
        self.device.create_pipeline_layout(descriptor)
    }

    fn create_compute_pipeline(
        &self,
        descriptor: &wgpu::ComputePipelineDescriptor<'_>,
    ) -> wgpu::ComputePipeline {
        self.record_create(RenderPassNativeResourceCreateMetrics::record_compute_pipeline);
        self.device.create_compute_pipeline(descriptor)
    }

    fn create_render_pipeline(
        &self,
        descriptor: &wgpu::RenderPipelineDescriptor<'_>,
    ) -> wgpu::RenderPipeline {
        self.record_create(RenderPassNativeResourceCreateMetrics::record_render_pipeline);
        self.device.create_render_pipeline(descriptor)
    }
}
