use std::collections::HashMap;

use crate::graphics::shader::invocation::ComputePipelineCacheKey;

use super::ibl_bake_shader_plan::IBL_BAKE_COMPUTE_ENTRY_POINT;
use super::ibl_bake_wgpu_binding::{
    create_ibl_bake_wgpu_source_sampler, IblBakeWgpuBindGroupLayouts,
};
use super::ibl_bake_wgpu_command_plan::{IblBakeWgpuCommandPlan, IblBakeWgpuOutputBindingKind};

#[cfg(test)]
#[path = "ibl_bake_wgpu_pipeline_cache/tests/fast_hit_tests.rs"]
mod fast_hit_tests;

pub(in crate::graphics::scene::scene_renderer) struct IblBakeWgpuPipelineCache {
    bind_group_layouts: IblBakeWgpuBindGroupLayouts,
    source_sampler: wgpu::Sampler,
    shader_modules: HashMap<ComputePipelineCacheKey, wgpu::ShaderModule>,
    pipeline_layouts: HashMap<IblBakeWgpuOutputBindingKind, wgpu::PipelineLayout>,
    compute_pipelines: HashMap<IblBakeWgpuComputePipelineCacheKey, wgpu::ComputePipeline>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer) struct IblBakeWgpuPipelineCacheStats {
    pub shader_module_count: usize,
    pub pipeline_layout_count: usize,
    pub compute_pipeline_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct IblBakeWgpuComputePipelineCacheKey {
    pipeline: ComputePipelineCacheKey,
    output_kind: IblBakeWgpuOutputBindingKind,
}

impl IblBakeWgpuPipelineCache {
    pub(in crate::graphics::scene::scene_renderer) fn new(device: &wgpu::Device) -> Self {
        Self {
            bind_group_layouts: IblBakeWgpuBindGroupLayouts::new(device),
            source_sampler: create_ibl_bake_wgpu_source_sampler(device),
            shader_modules: HashMap::new(),
            pipeline_layouts: HashMap::new(),
            compute_pipelines: HashMap::new(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn bind_group_layouts(
        &self,
    ) -> &IblBakeWgpuBindGroupLayouts {
        &self.bind_group_layouts
    }

    pub(in crate::graphics::scene::scene_renderer) fn source_sampler(&self) -> &wgpu::Sampler {
        &self.source_sampler
    }

    pub(in crate::graphics::scene::scene_renderer) fn ensure_compute_pipeline(
        &mut self,
        device: &wgpu::Device,
        command: &IblBakeWgpuCommandPlan,
    ) -> wgpu::ComputePipeline {
        let pipeline_key = IblBakeWgpuComputePipelineCacheKey {
            pipeline: command.pipeline_key.clone(),
            output_kind: command.bind_group_layout_kind,
        };
        if let Some(pipeline) = self.compute_pipelines.get(&pipeline_key) {
            return pipeline.clone();
        }

        let shader_key = pipeline_key.pipeline.clone();
        if !self.shader_modules.contains_key(&shader_key) {
            let shader_label = format!("{}-shader", command.pipeline_label);
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(shader_label.as_str()),
                source: wgpu::ShaderSource::Wgsl(command.wgsl_source.into()),
            });
            self.shader_modules.insert(shader_key.clone(), shader);
        }

        let layout_kind = command.bind_group_layout_kind;
        if !self.pipeline_layouts.contains_key(&layout_kind) {
            let layout_label = pipeline_layout_label(layout_kind);
            let bind_group_layout = self.bind_group_layouts.layout(layout_kind);
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(layout_label),
                bind_group_layouts: &[Some(bind_group_layout)],
                immediate_size: 0,
            });
            self.pipeline_layouts.insert(layout_kind, pipeline_layout);
        }

        let shader = self
            .shader_modules
            .get(&pipeline_key.pipeline)
            .expect("IBL bake shader module must be cached before pipeline creation");
        let layout = self
            .pipeline_layouts
            .get(&pipeline_key.output_kind)
            .expect("IBL bake pipeline layout must be cached before pipeline creation");
        let pipeline = create_ibl_bake_wgpu_compute_pipeline_from_cached_parts(
            device, command, layout, shader,
        );
        self.compute_pipelines
            .insert(pipeline_key, pipeline.clone());
        pipeline
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn stats(
        &self,
    ) -> IblBakeWgpuPipelineCacheStats {
        IblBakeWgpuPipelineCacheStats {
            shader_module_count: self.shader_modules.len(),
            pipeline_layout_count: self.pipeline_layouts.len(),
            compute_pipeline_count: self.compute_pipelines.len(),
        }
    }
}

pub(in crate::graphics::scene::scene_renderer) fn create_ibl_bake_wgpu_compute_pipeline_from_cached_parts(
    device: &wgpu::Device,
    command: &IblBakeWgpuCommandPlan,
    pipeline_layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(command.pipeline_label.as_str()),
        layout: Some(pipeline_layout),
        module: shader,
        entry_point: Some(IBL_BAKE_COMPUTE_ENTRY_POINT),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn pipeline_layout_label(output_kind: IblBakeWgpuOutputBindingKind) -> &'static str {
    match output_kind {
        IblBakeWgpuOutputBindingKind::StorageTexture2DArray => {
            "zircon-env-ibl-bake-storage-texture-pipeline-layout"
        }
        IblBakeWgpuOutputBindingKind::StorageBuffer => {
            "zircon-env-ibl-bake-storage-buffer-pipeline-layout"
        }
    }
}

#[cfg(test)]
#[path = "tests/ibl_bake_wgpu_pipeline_cache.rs"]
mod tests;
