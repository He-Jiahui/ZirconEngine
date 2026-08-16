use bytemuck::Zeroable;
use wgpu::util::DeviceExt;

use crate::hybrid_gi::renderer::gpu_resources::HybridGiGpuResources;
use crate::hybrid_gi::scene_representation::{
    HybridGiGlobalSdfPageBuildRequest, HybridGiGlobalSdfSceneState, HybridGiMeshSdfObject,
};

use super::super::buffer_helpers::{create_pod_storage_buffer, create_u32_storage_buffer};
use super::packing::{
    pack_global_sdf_build_inputs, GlobalSdfGpuMeshPayload, GlobalSdfGpuObject,
    GlobalSdfPageBuildDispositionKind, GLOBAL_SDF_PAGE_VOXEL_COUNT,
};
use super::{
    GlobalSdfGpuBuildDispatch, GlobalSdfGpuPendingBuild, GlobalSdfGpuResources, GlobalSdfGpuState,
};

const GLOBAL_SDF_BUILD_WORKGROUP_SIZE: u32 = 64;

impl GlobalSdfGpuResources {
    pub(in crate::hybrid_gi::renderer) fn dispatch_pages(
        &self,
        state: &GlobalSdfGpuState,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        scene: &mut HybridGiGlobalSdfSceneState,
        objects: &[HybridGiMeshSdfObject],
        requests: &[HybridGiGlobalSdfPageBuildRequest],
        page_budget: usize,
    ) -> GlobalSdfGpuBuildDispatch {
        let inputs = pack_global_sdf_build_inputs(scene, objects, requests, page_budget);
        let terminal_fallbacks = inputs
            .dispositions
            .iter()
            .filter(|disposition| {
                disposition.kind == GlobalSdfPageBuildDispositionKind::TerminalFallback
            })
            .map(|disposition| disposition.request)
            .collect::<Vec<_>>();
        scene.resolve_pages_to_fallback(&terminal_fallbacks);
        if inputs.pages.is_empty() {
            return GlobalSdfGpuBuildDispatch::without_pending(inputs.stats);
        }
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-hybrid-gi-global-sdf-params"),
            contents: bytemuck::bytes_of(&inputs.params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let page_buffer = create_pod_storage_buffer(
            device,
            "zircon-hybrid-gi-global-sdf-pages",
            &inputs.pages,
            wgpu::BufferUsages::STORAGE,
        );
        let fallback_object = [GlobalSdfGpuObject::zeroed()];
        let object_upload = if inputs.objects.is_empty() {
            &fallback_object[..]
        } else {
            &inputs.objects
        };
        let object_buffer = create_pod_storage_buffer(
            device,
            "zircon-hybrid-gi-global-sdf-objects",
            object_upload,
            wgpu::BufferUsages::STORAGE,
        );
        let fallback_payload = [GlobalSdfGpuMeshPayload::zeroed()];
        let payload_upload = if inputs.payloads.is_empty() {
            &fallback_payload[..]
        } else {
            &inputs.payloads
        };
        let payload_buffer = create_pod_storage_buffer(
            device,
            "zircon-hybrid-gi-global-sdf-mesh-payloads",
            payload_upload,
            wgpu::BufferUsages::STORAGE,
        );
        let voxel_buffer = create_u32_storage_buffer(
            device,
            "zircon-hybrid-gi-global-sdf-mesh-voxels",
            &inputs.voxels,
            wgpu::BufferUsages::STORAGE,
        );
        let candidate_buffer = create_u32_storage_buffer(
            device,
            "zircon-hybrid-gi-global-sdf-page-candidates",
            &inputs.candidates,
            wgpu::BufferUsages::STORAGE,
        );
        let completion_buffer = create_u32_storage_buffer(
            device,
            "zircon-hybrid-gi-global-sdf-page-completions",
            &vec![0; inputs.pages.len()],
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-hybrid-gi-global-sdf-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                binding(0, &params_buffer),
                binding(1, &page_buffer),
                binding(2, &object_buffer),
                binding(3, &payload_buffer),
                binding(4, &voxel_buffer),
                binding(5, &candidate_buffer),
                binding(6, &state.atlas_buffer),
                binding(7, &completion_buffer),
            ],
        });
        let invocation_count = inputs.pages.len() as u32 * GLOBAL_SDF_PAGE_VOXEL_COUNT as u32;
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("HybridGiGlobalSdfPageBuildPass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(
            invocation_count.div_ceil(GLOBAL_SDF_BUILD_WORKGROUP_SIZE),
            1,
            1,
        );
        drop(pass);

        GlobalSdfGpuBuildDispatch::with_pending(GlobalSdfGpuPendingBuild {
            requests: inputs.requests,
            completion_buffer,
            stats: inputs.stats,
        })
    }
}

impl HybridGiGpuResources {
    pub(in crate::hybrid_gi::renderer) fn dispatch_global_sdf_pages(
        &self,
        state: &GlobalSdfGpuState,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        scene: &mut HybridGiGlobalSdfSceneState,
        objects: &[HybridGiMeshSdfObject],
        requests: &[HybridGiGlobalSdfPageBuildRequest],
        page_budget: usize,
    ) -> GlobalSdfGpuBuildDispatch {
        self.global_sdf.dispatch_pages(
            state,
            device,
            encoder,
            scene,
            objects,
            requests,
            page_budget,
        )
    }
}

fn binding(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}
