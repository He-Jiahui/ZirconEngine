use std::collections::HashMap;

use crate::graphics::scene::scene_renderer::environment::realtime_ibl_graph_plan::{
    RealtimeIblGraphPass, RealtimeIblGraphPlan,
};
use crate::render_graph::{CompiledRenderGraph, RenderGraphError};

pub(in crate::graphics) struct RealtimeIblCompiledGraphVariant {
    plan: RealtimeIblGraphPlan,
    graph: CompiledRenderGraph,
    recording_passes: Vec<RealtimeIblGraphPass>,
    required_resource_names: Vec<String>,
}

impl RealtimeIblCompiledGraphVariant {
    pub(super) fn new(
        plan: RealtimeIblGraphPlan,
        graph: CompiledRenderGraph,
    ) -> Result<Self, RenderGraphError> {
        let authored_passes = plan
            .passes
            .iter()
            .map(|pass| (pass.pass_id, pass.clone()))
            .collect::<HashMap<_, _>>();
        // The recorder consumes compiler order but still records culled passes
        // until IBL executor culling semantics have product evidence.
        let recording_passes = graph
            .passes()
            .iter()
            .map(|pass| {
                authored_passes
                    .get(&pass.id)
                    .cloned()
                    .ok_or(RenderGraphError::UnknownPass {
                        pass: pass.id.index(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut required_resource_names = graph
            .resource_lifetimes()
            .iter()
            .map(|lifetime| lifetime.name.clone())
            .collect::<Vec<_>>();
        required_resource_names.sort();
        Ok(Self {
            plan,
            graph,
            recording_passes,
            required_resource_names,
        })
    }

    pub(in crate::graphics) fn plan(&self) -> &RealtimeIblGraphPlan {
        &self.plan
    }

    pub(in crate::graphics) fn graph(&self) -> &CompiledRenderGraph {
        &self.graph
    }

    pub(in crate::graphics) fn recording_passes(&self) -> &[RealtimeIblGraphPass] {
        &self.recording_passes
    }

    pub(in crate::graphics) fn required_resource_names(&self) -> &[String] {
        &self.required_resource_names
    }
}
