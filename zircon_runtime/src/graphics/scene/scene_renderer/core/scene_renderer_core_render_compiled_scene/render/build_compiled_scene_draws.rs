use crate::core::framework::render::ShaderQualityTier;
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::gpu_scene::GpuSceneUploadReport;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::mesh::{
    BuiltMeshDraws, CachedMeshDrawCommands, MeshDraw, MeshPassCommandBuffers, MeshPipelineCache,
    PendingMeshCommandCacheExtractionStats, PendingMeshCommandCachePlanStats,
    PreparedMeshQueueStats, PreparedMeshVirtualGeometryExecutionStats,
    PreparedMeshVirtualGeometryIndirectStats,
};
use crate::graphics::scene::scene_renderer::shadow::ShadowLightSlotAssignments;
use crate::graphics::types::ViewportRenderFrame;

use super::super::super::scene_renderer_core::SceneRendererAdvancedPluginResources;

pub(super) struct CompiledSceneDraws {
    draws: Vec<MeshDraw>,
    prepared_mesh_queue_stats: PreparedMeshQueueStats,
    prebuilt_mesh_pass_command_buffers: MeshPassCommandBuffers,
    gpu_scene_upload_report: GpuSceneUploadReport,
    indirect_segment_count: u32,
    indirect_args_count: u32,
    indirect_args_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_submission_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_authority_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_draw_ref_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_segment_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    pending_command_cache_plan_stats: PendingMeshCommandCachePlanStats,
    pending_command_cache_extraction_stats: PendingMeshCommandCacheExtractionStats,
}

impl CompiledSceneDraws {
    fn from_built_mesh_draws(built_mesh_draws: BuiltMeshDraws) -> Self {
        let indirect_segment_count = built_mesh_draws.indirect_segment_count();
        let indirect_args_count = built_mesh_draws.indirect_args_count();
        let indirect_args_buffer = built_mesh_draws.indirect_args_buffer();
        let indirect_submission_buffer = built_mesh_draws.indirect_submission_buffer();
        let indirect_authority_buffer = built_mesh_draws.indirect_authority_buffer();
        let indirect_draw_ref_buffer = built_mesh_draws.indirect_draw_ref_buffer();
        let indirect_segment_buffer = built_mesh_draws.indirect_segment_buffer();
        let gpu_scene_upload_report = built_mesh_draws.gpu_scene_upload_report();
        let prepared_mesh_queue_stats = built_mesh_draws.prepared_mesh_queue_stats();
        let prebuilt_mesh_pass_command_buffers =
            built_mesh_draws.prebuilt_mesh_pass_command_buffers();
        let pending_command_cache_plan_stats = built_mesh_draws.pending_command_cache_plan_stats();
        let pending_command_cache_extraction_stats =
            built_mesh_draws.pending_command_cache_extraction_stats();
        Self {
            draws: built_mesh_draws.into_draws(),
            prepared_mesh_queue_stats,
            prebuilt_mesh_pass_command_buffers,
            gpu_scene_upload_report,
            indirect_segment_count,
            indirect_args_count,
            indirect_args_buffer,
            indirect_submission_buffer,
            indirect_authority_buffer,
            indirect_draw_ref_buffer,
            indirect_segment_buffer,
            pending_command_cache_plan_stats,
            pending_command_cache_extraction_stats,
        }
    }

    pub(super) fn draws(&self) -> &[MeshDraw] {
        &self.draws
    }

    pub(super) fn draws_mut(&mut self) -> &mut [MeshDraw] {
        &mut self.draws
    }

    pub(super) fn prepared_mesh_queue_stats(&self) -> PreparedMeshQueueStats {
        self.prepared_mesh_queue_stats
    }

    pub(super) fn prebuilt_mesh_pass_command_buffers(&self) -> MeshPassCommandBuffers {
        self.prebuilt_mesh_pass_command_buffers.clone()
    }

    pub(super) fn gpu_scene_upload_report(&self) -> GpuSceneUploadReport {
        self.gpu_scene_upload_report
    }

    pub(super) fn indirect_segment_count(&self) -> u32 {
        self.indirect_segment_count
    }

    pub(super) fn indirect_args_count(&self) -> u32 {
        self.indirect_args_count
    }

    pub(super) fn indirect_args_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_args_buffer.clone()
    }

    pub(super) fn indirect_submission_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_submission_buffer.clone()
    }

    pub(super) fn indirect_authority_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_authority_buffer.clone()
    }

    pub(super) fn indirect_draw_ref_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_draw_ref_buffer.clone()
    }

    pub(super) fn indirect_segment_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_segment_buffer.clone()
    }

    pub(super) fn virtual_geometry_indirect_stats(
        &self,
    ) -> PreparedMeshVirtualGeometryIndirectStats {
        PreparedMeshVirtualGeometryIndirectStats {
            draw_count: self.indirect_args_count() as usize,
            buffer_count: [
                self.indirect_args_buffer(),
                self.indirect_submission_buffer(),
                self.indirect_authority_buffer(),
                self.indirect_draw_ref_buffer(),
                self.indirect_segment_buffer(),
            ]
            .into_iter()
            .flatten()
            .count(),
            args_count: self.indirect_args_count() as usize,
            segment_count: self.indirect_segment_count() as usize,
        }
    }

    pub(super) fn virtual_geometry_execution_stats(
        &self,
    ) -> PreparedMeshVirtualGeometryExecutionStats {
        PreparedMeshVirtualGeometryExecutionStats::from_execution_draws(
            self.draws.iter().enumerate().map(|(draw_index, draw)| {
                draw.virtual_geometry_execution_draw(saturated_u32_index(draw_index), draw_index)
            }),
        )
    }

    pub(super) fn pending_command_cache_plan_stats(&self) -> PendingMeshCommandCachePlanStats {
        self.pending_command_cache_plan_stats
    }

    pub(super) fn pending_command_cache_extraction_stats(
        &self,
    ) -> PendingMeshCommandCacheExtractionStats {
        self.pending_command_cache_extraction_stats
    }
}

pub(super) fn build_compiled_scene_draws(
    advanced_plugin_resources: &SceneRendererAdvancedPluginResources,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    material_texture_bind_group_layout: &wgpu::BindGroupLayout,
    gpu_scene: &mut GpuScene,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    virtual_geometry_enabled: bool,
    shadow_light_slots: Option<&ShadowLightSlotAssignments>,
    command_cache: &mut CachedMeshDrawCommands,
    mesh_pipelines: &mut MeshPipelineCache,
    generation: u64,
    shader_quality: ShaderQualityTier,
) -> CompiledSceneDraws {
    let built_mesh_draws = advanced_plugin_resources.build_mesh_draws_with_command_cache(
        device,
        queue,
        encoder,
        material_texture_bind_group_layout,
        gpu_scene,
        streamer,
        frame,
        virtual_geometry_enabled,
        shadow_light_slots,
        command_cache,
        mesh_pipelines,
        generation,
        shader_quality,
    );

    CompiledSceneDraws::from_built_mesh_draws(built_mesh_draws)
}

fn saturated_u32_index(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_scene_draws_report_virtual_geometry_indirect_counts_without_buffers() {
        let draws = CompiledSceneDraws {
            draws: Vec::new(),
            prepared_mesh_queue_stats: PreparedMeshQueueStats::default(),
            prebuilt_mesh_pass_command_buffers: MeshPassCommandBuffers::default(),
            gpu_scene_upload_report: GpuSceneUploadReport::default(),
            indirect_segment_count: 2,
            indirect_args_count: 3,
            indirect_args_buffer: None,
            indirect_submission_buffer: None,
            indirect_authority_buffer: None,
            indirect_draw_ref_buffer: None,
            indirect_segment_buffer: None,
            pending_command_cache_plan_stats: PendingMeshCommandCachePlanStats::default(),
            pending_command_cache_extraction_stats: Default::default(),
        };

        let stats = draws.virtual_geometry_indirect_stats();

        assert_eq!(stats.draw_count, 3);
        assert_eq!(stats.args_count, 3);
        assert_eq!(stats.segment_count, 2);
        assert_eq!(stats.buffer_count, 0);
    }
}
