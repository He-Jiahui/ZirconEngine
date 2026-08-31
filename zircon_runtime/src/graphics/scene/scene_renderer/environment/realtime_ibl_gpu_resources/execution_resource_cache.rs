use std::collections::HashMap;
use std::time::Instant;

use crate::core::framework::render::IblBakeArtifactRequest;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::render_graph::CompiledRenderGraph;

use super::super::realtime_ibl_graph_plan::{RealtimeIblGraphPlan, RealtimeIblGraphTopologyKey};
use super::super::realtime_ibl_time_slice::RealtimeIblFrameBatch;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RealtimeIblExecutionResourceCacheStats {
    pub cache_hit_count: u64,
    pub cache_miss_count: u64,
    pub validation_count: u64,
    pub entry_count: usize,
}

pub(in crate::graphics) struct RealtimeIblExecutionResourceResolution<'a> {
    resources: &'a RenderGraphExecutionResources,
    execution_resource_binding_micros: u64,
    validation_micros: u64,
    execution_resource_cache_hits: u64,
    execution_resource_cache_misses: u64,
    execution_resource_cache_entry_count: usize,
    execution_resource_cache_topology_capacity: usize,
    texture_view_binding_count: usize,
    buffer_binding_count: usize,
}

impl RealtimeIblExecutionResourceResolution<'_> {
    pub(in crate::graphics) fn resources(&self) -> &RenderGraphExecutionResources {
        self.resources
    }

    pub(in crate::graphics) const fn execution_resource_binding_micros(&self) -> u64 {
        self.execution_resource_binding_micros
    }

    pub(in crate::graphics) const fn validation_micros(&self) -> u64 {
        self.validation_micros
    }

    pub(in crate::graphics) const fn execution_resource_cache_hits(&self) -> u64 {
        self.execution_resource_cache_hits
    }

    pub(in crate::graphics) const fn execution_resource_cache_misses(&self) -> u64 {
        self.execution_resource_cache_misses
    }

    pub(in crate::graphics) const fn execution_resource_cache_entry_count(&self) -> usize {
        self.execution_resource_cache_entry_count
    }

    pub(in crate::graphics) const fn execution_resource_cache_topology_capacity(&self) -> usize {
        self.execution_resource_cache_topology_capacity
    }

    pub(in crate::graphics) const fn texture_view_binding_count(&self) -> usize {
        self.texture_view_binding_count
    }

    pub(in crate::graphics) const fn buffer_binding_count(&self) -> usize {
        self.buffer_binding_count
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RealtimeIblExecutionResourceLayout {
    source_face_size: u32,
    source_mip_count: u32,
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
}

impl RealtimeIblExecutionResourceLayout {
    fn from_request(request: &IblBakeArtifactRequest) -> Self {
        Self {
            source_face_size: request.source_face_size(),
            source_mip_count: request.source_mip_count(),
            pmrem_face_size: request.pmrem_face_size(),
            pmrem_mip_count: request.pmrem_mip_count(),
        }
    }
}

#[derive(Default)]
pub(super) struct RealtimeIblExecutionResourceCache {
    resource_layout: Option<RealtimeIblExecutionResourceLayout>,
    topology_capacity: usize,
    entries: HashMap<RealtimeIblGraphTopologyKey, RenderGraphExecutionResources>,
    cache_hit_count: u64,
    cache_miss_count: u64,
    validation_count: u64,
}

impl RealtimeIblExecutionResourceCache {
    pub(super) fn resolve(
        &mut self,
        request: &IblBakeArtifactRequest,
        batch: &RealtimeIblFrameBatch,
        plan: &RealtimeIblGraphPlan,
        graph: &CompiledRenderGraph,
        required_resource_names: &[String],
        cpu_timing_enabled: bool,
        bind: impl FnOnce(&mut RenderGraphExecutionResources) -> Result<(), String>,
    ) -> Result<RealtimeIblExecutionResourceResolution<'_>, String> {
        self.reset_for_layout_if_needed(
            RealtimeIblExecutionResourceLayout::from_request(request),
            batch.topology_cache_capacity(),
        );
        let key = RealtimeIblGraphTopologyKey::from_batch(batch);
        if self.entries.contains_key(&key) {
            self.cache_hit_count = self.cache_hit_count.saturating_add(1);
            let entry_count = self.entries.len();
            return Ok(RealtimeIblExecutionResourceResolution {
                resources: self
                    .entries
                    .get(&key)
                    .expect("realtime IBL execution resource key was checked before lookup"),
                execution_resource_binding_micros: 0,
                validation_micros: 0,
                execution_resource_cache_hits: 1,
                execution_resource_cache_misses: 0,
                execution_resource_cache_entry_count: entry_count,
                execution_resource_cache_topology_capacity: self.topology_capacity,
                texture_view_binding_count: 0,
                buffer_binding_count: 0,
            });
        }

        self.cache_miss_count = self.cache_miss_count.saturating_add(1);
        let binding_started = cpu_timing_enabled.then(Instant::now);
        let mut resources = RenderGraphExecutionResources::new();
        bind(&mut resources)?;
        let resource_report = resources.resource_report();
        let execution_resource_binding_micros = elapsed_micros(binding_started);
        let validation_started = cpu_timing_enabled.then(Instant::now);
        resources.validate_materialized_graph_resources(graph)?;
        let validation_micros = elapsed_micros(validation_started);
        self.validation_count = self.validation_count.saturating_add(1);
        self.entries.insert(key, resources);
        let entry_count = self.entries.len();

        debug_assert!(
            self.entries.len() <= self.topology_capacity,
            "realtime IBL execution resource cache exceeded scheduler topology capacity"
        );
        debug_assert!(
            required_resource_names
                .windows(2)
                .all(|names| names[0] <= names[1]),
            "compiled realtime IBL resource names must remain sorted for binding lookup"
        );
        debug_assert_eq!(
            plan.ready.slot,
            batch.ready_slot(),
            "execution resource key must match the compiled ready slot"
        );
        debug_assert_eq!(
            plan.work.slot,
            batch.work_slot(),
            "execution resource key must match the compiled work slot"
        );
        Ok(RealtimeIblExecutionResourceResolution {
            resources: self
                .entries
                .get(&key)
                .expect("inserted realtime IBL execution resources must be retrievable"),
            execution_resource_binding_micros,
            validation_micros,
            execution_resource_cache_hits: 0,
            execution_resource_cache_misses: 1,
            execution_resource_cache_entry_count: entry_count,
            execution_resource_cache_topology_capacity: self.topology_capacity,
            texture_view_binding_count: resource_report.texture_view_count,
            buffer_binding_count: resource_report.buffer_count,
        })
    }

    pub(super) fn stats(&self) -> RealtimeIblExecutionResourceCacheStats {
        RealtimeIblExecutionResourceCacheStats {
            cache_hit_count: self.cache_hit_count,
            cache_miss_count: self.cache_miss_count,
            validation_count: self.validation_count,
            entry_count: self.entries.len(),
        }
    }

    fn reset_for_layout_if_needed(
        &mut self,
        layout: RealtimeIblExecutionResourceLayout,
        topology_capacity: usize,
    ) {
        let topology_capacity = topology_capacity.max(1);
        if self.resource_layout == Some(layout) && self.topology_capacity == topology_capacity {
            return;
        }
        self.resource_layout = Some(layout);
        self.topology_capacity = topology_capacity;
        self.entries.clear();
    }
}

fn elapsed_micros(started: Option<Instant>) -> u64 {
    started
        .map(|started| u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX))
        .unwrap_or_default()
}
