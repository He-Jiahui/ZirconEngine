use std::collections::{HashMap, VecDeque};

use crate::core::framework::render::IblBakeArtifactRequest;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_graph_plan::{
    append_realtime_ibl_graph_plan, RealtimeIblGraphTopologyKey,
};
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_time_slice::RealtimeIblFrameBatch;
use crate::render_graph::{RenderGraphBuilder, RenderGraphError};

use super::variant::RealtimeIblCompiledGraphVariant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RealtimeIblResourceLayout {
    source_face_size: u32,
    source_mip_count: u32,
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
}

impl RealtimeIblResourceLayout {
    fn from_request(request: &IblBakeArtifactRequest) -> Self {
        Self {
            source_face_size: request.source_face_size(),
            source_mip_count: request.source_mip_count(),
            pmrem_face_size: request.pmrem_face_size(),
            pmrem_mip_count: request.pmrem_mip_count(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblCompiledGraphCacheStats {
    pub cache_hit_count: u64,
    pub cache_miss_count: u64,
    pub compile_count: u64,
    pub eviction_count: u64,
    pub variant_count: usize,
}

pub(in crate::graphics) struct RealtimeIblCompiledGraphCache {
    resource_layout: Option<RealtimeIblResourceLayout>,
    variants: HashMap<RealtimeIblGraphTopologyKey, RealtimeIblCompiledGraphVariant>,
    insertion_order: VecDeque<RealtimeIblGraphTopologyKey>,
    capacity: usize,
    cache_hit_count: u64,
    cache_miss_count: u64,
    compile_count: u64,
    eviction_count: u64,
}

impl RealtimeIblCompiledGraphCache {
    pub(in crate::graphics) fn new() -> Self {
        Self {
            resource_layout: None,
            variants: HashMap::new(),
            insertion_order: VecDeque::new(),
            capacity: 0,
            cache_hit_count: 0,
            cache_miss_count: 0,
            compile_count: 0,
            eviction_count: 0,
        }
    }

    pub(in crate::graphics) fn resolve(
        &mut self,
        request: &IblBakeArtifactRequest,
        batch: &RealtimeIblFrameBatch,
    ) -> Result<&RealtimeIblCompiledGraphVariant, RenderGraphError> {
        self.reset_for_layout_if_needed(RealtimeIblResourceLayout::from_request(request), batch);
        let topology = RealtimeIblGraphTopologyKey::from_batch(batch);
        if self.variants.contains_key(&topology) {
            self.cache_hit_count = self.cache_hit_count.saturating_add(1);
            return Ok(self
                .variants
                .get(&topology)
                .expect("realtime IBL topology key was checked before lookup"));
        }

        self.cache_miss_count = self.cache_miss_count.saturating_add(1);
        while self.variants.len() >= self.capacity {
            let Some(evicted) = self.insertion_order.pop_front() else {
                break;
            };
            if self.variants.remove(&evicted).is_some() {
                self.eviction_count = self.eviction_count.saturating_add(1);
            }
        }

        let mut builder = RenderGraphBuilder::new("realtime-ibl-frame");
        let plan = append_realtime_ibl_graph_plan(&mut builder, request, batch)?;
        let graph = builder.compile()?;
        let variant = RealtimeIblCompiledGraphVariant::new(plan, graph)?;
        self.variants.insert(topology.clone(), variant);
        self.insertion_order.push_back(topology.clone());
        self.compile_count = self.compile_count.saturating_add(1);
        Ok(self
            .variants
            .get(&topology)
            .expect("inserted realtime IBL topology must be retrievable"))
    }

    pub(in crate::graphics) fn stats(&self) -> RealtimeIblCompiledGraphCacheStats {
        RealtimeIblCompiledGraphCacheStats {
            cache_hit_count: self.cache_hit_count,
            cache_miss_count: self.cache_miss_count,
            compile_count: self.compile_count,
            eviction_count: self.eviction_count,
            variant_count: self.variants.len(),
        }
    }

    fn reset_for_layout_if_needed(
        &mut self,
        layout: RealtimeIblResourceLayout,
        batch: &RealtimeIblFrameBatch,
    ) {
        if self.resource_layout == Some(layout) {
            return;
        }
        self.resource_layout = Some(layout);
        self.variants.clear();
        self.insertion_order.clear();
        self.capacity = batch.topology_cache_capacity().max(1);
    }
}
