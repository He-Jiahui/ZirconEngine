use std::collections::HashMap;

use crate::graphics::scene::scene_renderer::environment::realtime_ibl_graph_plan::{
    RealtimeIblGraphPass, RealtimeIblGraphPlan,
};
use crate::render_graph::{CompiledRenderGraph, RenderGraphError};

/// 同时保存作者计划、编译顺序和已排序资源名，供回放与物理绑定缓存共享一个拓扑身份。
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
        // TODO: [CR-SCENE-ENV-0004] 确认 IBL 回放保留 culled 通道的产品契约；
        // 图资源寿命只覆盖存活访问，需用裁剪后的 IBL 图验证回放与绑定集合仍一致。
        let mut recording_passes = Vec::with_capacity(graph.passes().len());
        for pass in graph.passes() {
            recording_passes.push(authored_passes.get(&pass.id).cloned().ok_or(
                RenderGraphError::UnknownPass {
                    pass: pass.id.index(),
                },
            )?);
        }
        let mut required_resource_names = Vec::with_capacity(graph.resource_lifetimes().len());
        required_resource_names.extend(
            graph
                .resource_lifetimes()
                .iter()
                .map(|lifetime| lifetime.name.clone()),
        );
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

#[cfg(test)]
#[path = "tests/variant.rs"]
mod tests;
