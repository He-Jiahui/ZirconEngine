use std::collections::HashMap;

use crate::core::framework::render::ComputePipelineCacheKey;

use super::ibl_bake_shader_plan::IBL_BAKE_COMPUTE_ENTRY_POINT;
use super::ibl_bake_wgpu_binding::{
    create_ibl_bake_wgpu_source_sampler, IblBakeWgpuBindGroupLayouts,
};
use super::ibl_bake_wgpu_command_plan::{IblBakeWgpuCommandPlan, IblBakeWgpuOutputBindingKind};

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
        let shader_key = command.pipeline_key.clone();
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

        let pipeline_key = IblBakeWgpuComputePipelineCacheKey {
            pipeline: shader_key,
            output_kind: layout_kind,
        };
        if !self.compute_pipelines.contains_key(&pipeline_key) {
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
                .insert(pipeline_key.clone(), pipeline);
        }

        self.compute_pipelines
            .get(&pipeline_key)
            .expect("IBL bake compute pipeline must be cached")
            .clone()
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
mod tests {
    use crate::core::framework::render::{
        IblBakeArtifactContents, IblBakeArtifactRequest, ProceduralSkyParams,
    };
    use crate::graphics::backend::RenderBackend;

    use super::super::ibl_bake_shader_plan::IblBakeComputeKernelKind;
    use super::super::ibl_bake_wgpu_command_plan::{
        ibl_bake_wgpu_command_plan_for_request, IblBakeWgpuCommandPlan,
    };
    use super::*;

    #[test]
    fn pipeline_cache_reuses_pmrem_shader_and_pipeline_across_mips() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let request = request(16, 5, IblBakeArtifactContents::PMREM_SH9);
        let plan = ibl_bake_wgpu_command_plan_for_request(&request);
        let pmrem_mip0 = command_for_kind(
            &plan.commands,
            IblBakeComputeKernelKind::Pmrem { mip_level: 0 },
        );
        let pmrem_mip1 = command_for_kind(
            &plan.commands,
            IblBakeComputeKernelKind::Pmrem { mip_level: 1 },
        );
        let sh9 = command_for_kind(&plan.commands, IblBakeComputeKernelKind::IrradianceSh9);
        let mut cache = IblBakeWgpuPipelineCache::new(&backend.device);

        let _ = cache.ensure_compute_pipeline(&backend.device, pmrem_mip0);
        assert_eq!(
            cache.stats(),
            IblBakeWgpuPipelineCacheStats {
                shader_module_count: 1,
                pipeline_layout_count: 1,
                compute_pipeline_count: 1,
            }
        );

        let _ = cache.ensure_compute_pipeline(&backend.device, pmrem_mip0);
        let _ = cache.ensure_compute_pipeline(&backend.device, pmrem_mip1);
        assert_eq!(
            cache.stats(),
            IblBakeWgpuPipelineCacheStats {
                shader_module_count: 1,
                pipeline_layout_count: 1,
                compute_pipeline_count: 1,
            }
        );

        let _ = cache.ensure_compute_pipeline(&backend.device, sh9);
        assert_eq!(
            cache.stats(),
            IblBakeWgpuPipelineCacheStats {
                shader_module_count: 2,
                pipeline_layout_count: 2,
                compute_pipeline_count: 2,
            }
        );
    }

    #[test]
    fn pipeline_cache_owns_the_production_source_sampler() {
        let source = include_str!("ibl_bake_wgpu_dispatch.rs");
        let realtime = include_str!("realtime_ibl_wgpu_recorder.rs");

        assert!(!source.contains("create_ibl_bake_wgpu_source_sampler"));
        assert!(!realtime.contains("create_ibl_bake_wgpu_source_sampler"));
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
            .unwrap_or_else(|| panic!("IBL bake command {kind:?} should exist"))
    }
}
