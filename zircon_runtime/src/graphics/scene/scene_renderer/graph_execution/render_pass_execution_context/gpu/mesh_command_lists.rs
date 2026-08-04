use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawCommand, MeshDrawCommandStream, MeshDrawReplayStatsAccumulator,
    MeshIndirectDrawExecution, MeshSceneDataBindHandle,
};

#[derive(Clone, Copy)]
pub(in crate::graphics::scene::scene_renderer) struct RenderPassMeshCommandLists<'a> {
    pub replay_stats: &'a MeshDrawReplayStatsAccumulator,
    pub gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'a>>,
    pub depth_prepass_commands: &'a [MeshDrawCommand],
    pub shadow_commands: &'a [MeshDrawCommand],
    pub opaque_commands: &'a [MeshDrawCommand],
    pub alpha_mask_commands: &'a [MeshDrawCommand],
    pub advanced_pbr_opaque_commands: &'a [MeshDrawCommand],
    pub transmission_commands: &'a [MeshDrawCommand],
    pub transmission_step_count: usize,
    pub transparent_commands: &'a [MeshDrawCommand],
    pub half_resolution_transparent_commands: &'a [MeshDrawCommand],
    pub velocity_commands: &'a [MeshDrawCommand],
    pub taa_reactive_mask_commands: &'a [MeshDrawCommand],
    pub depth_prepass_indirect: Option<&'a MeshIndirectDrawExecution>,
    pub shadow_indirect: Option<&'a MeshIndirectDrawExecution>,
    pub opaque_indirect: Option<&'a MeshIndirectDrawExecution>,
    pub alpha_mask_indirect: Option<&'a MeshIndirectDrawExecution>,
    pub advanced_pbr_opaque_indirect: Option<&'a MeshIndirectDrawExecution>,
    pub transparent_indirect: Option<&'a MeshIndirectDrawExecution>,
    pub half_resolution_transparent_indirect: Option<&'a MeshIndirectDrawExecution>,
    pub velocity_indirect: Option<&'a MeshIndirectDrawExecution>,
    pub taa_reactive_mask_indirect: Option<&'a MeshIndirectDrawExecution>,
}

impl<'a> RenderPassMeshCommandLists<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn with_replay_stats<'b>(
        self,
        replay_stats: &'b MeshDrawReplayStatsAccumulator,
    ) -> RenderPassMeshCommandLists<'b>
    where
        'a: 'b,
    {
        RenderPassMeshCommandLists {
            replay_stats,
            ..self
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn stream_for_stage(
        &self,
        stage: RenderPassStage,
    ) -> MeshDrawCommandStream<'a> {
        match stage {
            RenderPassStage::DepthPrepass => self.depth_prepass_stream(),
            RenderPassStage::Opaque3d => self.opaque_stream(),
            RenderPassStage::AlphaMask3d => self.alpha_mask_stream(),
            RenderPassStage::Transparent3d => self.transparent_stream(),
            RenderPassStage::Shadow => self.shadow_stream(),
            _ => MeshDrawCommandStream::empty(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn depth_prepass_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(self.depth_prepass_commands, self.depth_prepass_indirect)
    }

    pub(in crate::graphics::scene::scene_renderer) fn shadow_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(self.shadow_commands, self.shadow_indirect)
    }

    pub(in crate::graphics::scene::scene_renderer) fn opaque_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(self.opaque_commands, self.opaque_indirect)
    }

    pub(in crate::graphics::scene::scene_renderer) fn alpha_mask_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(self.alpha_mask_commands, self.alpha_mask_indirect)
    }

    pub(in crate::graphics::scene::scene_renderer) fn transparent_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(self.transparent_commands, self.transparent_indirect)
    }

    pub(in crate::graphics::scene::scene_renderer) fn half_resolution_transparent_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(
            self.half_resolution_transparent_commands,
            self.half_resolution_transparent_indirect,
        )
    }

    pub(in crate::graphics::scene::scene_renderer) fn advanced_pbr_opaque_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(
            self.advanced_pbr_opaque_commands,
            self.advanced_pbr_opaque_indirect,
        )
    }

    pub(in crate::graphics::scene::scene_renderer) fn transmission_step_stream(
        &self,
        step_index: usize,
    ) -> MeshDrawCommandStream<'a> {
        let Some(range) = crate::graphics::scene::scene_renderer::advanced_lighting::transmission::transmission_step_range(
            self.transmission_commands.len(),
            self.transmission_step_count,
            step_index,
        ) else {
            return MeshDrawCommandStream::empty();
        };
        // Transmission ranges intentionally disable cross-command indirect batching;
        // command-local indirect draws remain valid for the selected slice.
        MeshDrawCommandStream::new(&self.transmission_commands[range], None)
    }

    pub(in crate::graphics::scene::scene_renderer) fn velocity_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(self.velocity_commands, self.velocity_indirect)
    }

    pub(in crate::graphics::scene::scene_renderer) fn taa_reactive_mask_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(
            self.taa_reactive_mask_commands,
            self.taa_reactive_mask_indirect,
        )
    }

    pub(in crate::graphics::scene::scene_renderer) fn occlusion_cull_candidate_arg_count(
        &self,
    ) -> u32 {
        self.hzb_occlusion_indirect_executions()
            .into_iter()
            .flatten()
            .map(|execution| execution.args_count())
            .sum()
    }

    pub(in crate::graphics::scene::scene_renderer) fn occlusion_cull_candidate_instance_count(
        &self,
    ) -> u32 {
        self.hzb_occlusion_indirect_executions()
            .into_iter()
            .flatten()
            .map(|execution| execution.total_instances())
            .sum()
    }

    pub(in crate::graphics::scene::scene_renderer) fn hzb_occlusion_indirect_executions(
        &self,
    ) -> [Option<&'a MeshIndirectDrawExecution>; 4] {
        [
            self.opaque_indirect,
            self.alpha_mask_indirect,
            self.advanced_pbr_opaque_indirect,
            self.velocity_indirect,
        ]
    }
}
