use crate::render_graph::{QueueLane, RenderGraphPassResourceAccess, RenderPassId};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphExecutionRecord {
    executed_passes: Vec<String>,
    executed_executor_ids: Vec<String>,
    executed_queue_lanes: Vec<QueueLane>,
    executed_declared_queue_lanes: Vec<QueueLane>,
    executed_pass_dependencies: Vec<Vec<RenderPassId>>,
    executed_pass_resources: Vec<Vec<RenderGraphPassResourceAccess>>,
}

impl RenderGraphExecutionRecord {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn push_executed_pass(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        queue: QueueLane,
    ) {
        self.push_executed_pass_with_resources(pass_name, executor_id, queue, Vec::new());
    }

    pub fn push_executed_pass_with_resources(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        queue: QueueLane,
        resources: Vec<RenderGraphPassResourceAccess>,
    ) {
        self.push_executed_pass_with_declared_queue_dependencies_and_resources(
            pass_name,
            executor_id,
            queue,
            queue,
            Vec::new(),
            resources,
        );
    }

    pub fn push_executed_pass_with_declared_queue_and_resources(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        queue: QueueLane,
        declared_queue: QueueLane,
        resources: Vec<RenderGraphPassResourceAccess>,
    ) {
        self.push_executed_pass_with_declared_queue_dependencies_and_resources(
            pass_name,
            executor_id,
            queue,
            declared_queue,
            Vec::new(),
            resources,
        );
    }

    pub fn push_executed_pass_with_declared_queue_dependencies_and_resources(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        queue: QueueLane,
        declared_queue: QueueLane,
        dependencies: Vec<RenderPassId>,
        resources: Vec<RenderGraphPassResourceAccess>,
    ) {
        self.executed_passes.push(pass_name.into());
        self.executed_executor_ids.push(executor_id.into());
        self.executed_queue_lanes.push(queue);
        self.executed_declared_queue_lanes.push(declared_queue);
        self.executed_pass_dependencies.push(dependencies);
        self.executed_pass_resources.push(resources);
    }

    pub fn executed_passes(&self) -> &[String] {
        &self.executed_passes
    }

    pub fn executed_executor_ids(&self) -> &[String] {
        &self.executed_executor_ids
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn executed_pass_resources(&self) -> &[Vec<RenderGraphPassResourceAccess>] {
        &self.executed_pass_resources
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn executed_pass_dependencies(&self) -> &[Vec<RenderPassId>] {
        &self.executed_pass_dependencies
    }

    pub fn executed_resource_access_count(&self) -> usize {
        self.executed_pass_resources.iter().map(Vec::len).sum()
    }

    pub fn executed_dependency_count(&self) -> usize {
        self.executed_pass_dependencies.iter().map(Vec::len).sum()
    }

    pub fn executed_queue_fallback_count(&self) -> usize {
        self.executed_queue_lanes
            .iter()
            .zip(&self.executed_declared_queue_lanes)
            .filter(|(queue, declared_queue)| queue != declared_queue)
            .count()
    }

    pub fn executed_queue_lane_count(&self, queue: QueueLane) -> usize {
        self.executed_queue_lanes
            .iter()
            .filter(|executed_queue| **executed_queue == queue)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use crate::render_graph::RenderPassId;
    use crate::render_graph::{
        QueueLane, RenderGraphPassResourceAccess, RenderGraphResourceAccessKind,
        RenderGraphResourceKind,
    };

    use super::RenderGraphExecutionRecord;

    #[test]
    fn execution_record_counts_queue_lanes_from_executed_passes() {
        let mut record = RenderGraphExecutionRecord::default();

        record.push_executed_pass_with_declared_queue_and_resources(
            "cull",
            "virtual-geometry.node-cluster-cull",
            QueueLane::Graphics,
            QueueLane::AsyncCompute,
            Vec::new(),
        );
        record.push_executed_pass("main", "mesh.opaque", QueueLane::Graphics);

        assert_eq!(record.executed_queue_lane_count(QueueLane::AsyncCompute), 0);
        assert_eq!(record.executed_queue_lane_count(QueueLane::Graphics), 2);
        assert_eq!(record.executed_queue_lane_count(QueueLane::AsyncCopy), 0);
        assert_eq!(record.executed_queue_fallback_count(), 1);
    }

    #[test]
    fn execution_record_preserves_executed_pass_resource_accesses() {
        let mut record = RenderGraphExecutionRecord::default();
        let resources = vec![
            RenderGraphPassResourceAccess {
                name: "scene-depth".to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
            },
            RenderGraphPassResourceAccess {
                name: "scene-color".to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Write,
            },
        ];

        record.push_executed_pass_with_resources(
            "opaque",
            "mesh.opaque",
            QueueLane::Graphics,
            resources.clone(),
        );

        assert_eq!(record.executed_pass_resources(), &[resources]);
        assert_eq!(record.executed_resource_access_count(), 2);
    }

    #[test]
    fn execution_record_preserves_executed_pass_dependencies() {
        let mut record = RenderGraphExecutionRecord::default();
        let dependencies = vec![RenderPassId(2), RenderPassId(5)];

        record.push_executed_pass_with_declared_queue_dependencies_and_resources(
            "lighting",
            "lighting.clustered-cull",
            QueueLane::Graphics,
            QueueLane::Graphics,
            dependencies.clone(),
            Vec::new(),
        );

        assert_eq!(record.executed_pass_dependencies(), &[dependencies]);
        assert_eq!(record.executed_dependency_count(), 2);
    }
}
