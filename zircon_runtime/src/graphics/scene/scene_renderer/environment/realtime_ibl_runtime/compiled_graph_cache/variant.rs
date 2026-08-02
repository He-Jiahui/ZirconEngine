use crate::core::framework::render::IblBakeArtifactRequest;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_graph_plan::RealtimeIblGraphPlan;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_time_slice::{
    IblRealtimeBufferSlot, RealtimeIblFrameBatch, RealtimeIblOperation,
};
use crate::render_graph::CompiledRenderGraph;

pub(in crate::graphics) struct RealtimeIblCompiledGraphVariant {
    source_face_size: u32,
    source_mip_count: u32,
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
    ready_slot: IblRealtimeBufferSlot,
    work_slot: IblRealtimeBufferSlot,
    operations: Vec<RealtimeIblOperation>,
    plan: RealtimeIblGraphPlan,
    graph: CompiledRenderGraph,
    required_resource_names: Vec<String>,
}

impl RealtimeIblCompiledGraphVariant {
    pub(super) fn new(
        request: &IblBakeArtifactRequest,
        batch: &RealtimeIblFrameBatch,
        plan: RealtimeIblGraphPlan,
        graph: CompiledRenderGraph,
    ) -> Self {
        let mut required_resource_names = graph
            .resource_lifetimes()
            .iter()
            .map(|lifetime| lifetime.name.clone())
            .collect::<Vec<_>>();
        required_resource_names.sort();
        Self {
            source_face_size: request.source_face_size(),
            source_mip_count: request.source_mip_count(),
            pmrem_face_size: request.pmrem_face_size(),
            pmrem_mip_count: request.pmrem_mip_count(),
            ready_slot: batch.ready_slot(),
            work_slot: batch.work_slot(),
            operations: batch.operations().to_vec(),
            plan,
            graph,
            required_resource_names,
        }
    }

    pub(super) fn matches(
        &self,
        request: &IblBakeArtifactRequest,
        batch: &RealtimeIblFrameBatch,
    ) -> bool {
        self.source_face_size == request.source_face_size()
            && self.source_mip_count == request.source_mip_count()
            && self.pmrem_face_size == request.pmrem_face_size()
            && self.pmrem_mip_count == request.pmrem_mip_count()
            && self.ready_slot == batch.ready_slot()
            && self.work_slot == batch.work_slot()
            && self.operations == batch.operations()
    }

    pub(in crate::graphics) fn plan(&self) -> &RealtimeIblGraphPlan {
        &self.plan
    }

    pub(in crate::graphics) fn graph(&self) -> &CompiledRenderGraph {
        &self.graph
    }

    pub(in crate::graphics) fn required_resource_names(&self) -> &[String] {
        &self.required_resource_names
    }
}
