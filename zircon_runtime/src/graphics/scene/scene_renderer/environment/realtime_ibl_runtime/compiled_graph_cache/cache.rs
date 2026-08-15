use crate::core::framework::render::IblBakeArtifactRequest;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_graph_plan::append_realtime_ibl_graph_plan;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_time_slice::RealtimeIblFrameBatch;
use crate::render_graph::{RenderGraphBuilder, RenderGraphError};

use super::variant::RealtimeIblCompiledGraphVariant;

// The fixed eight-mip, two-face scheduler has 17 operation topologies and two
// work-slot choices. Updating that scheduler requires revisiting this bound.
const REALTIME_IBL_TOPOLOGY_VARIANT_CAPACITY: usize = 34;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblCompiledGraphCacheStats {
    pub cache_hit_count: u64,
    pub cache_miss_count: u64,
    pub compile_count: u64,
    pub variant_count: usize,
}

pub(in crate::graphics) struct RealtimeIblCompiledGraphCache {
    variants: Vec<RealtimeIblCompiledGraphVariant>,
    cache_hit_count: u64,
    cache_miss_count: u64,
    compile_count: u64,
}

impl RealtimeIblCompiledGraphCache {
    pub(in crate::graphics) fn new() -> Self {
        Self {
            variants: Vec::with_capacity(REALTIME_IBL_TOPOLOGY_VARIANT_CAPACITY),
            cache_hit_count: 0,
            cache_miss_count: 0,
            compile_count: 0,
        }
    }

    pub(in crate::graphics) fn resolve(
        &mut self,
        request: &IblBakeArtifactRequest,
        batch: &RealtimeIblFrameBatch,
    ) -> Result<&RealtimeIblCompiledGraphVariant, RenderGraphError> {
        if let Some(index) = self
            .variants
            .iter()
            .position(|variant| variant.matches(request, batch))
        {
            self.cache_hit_count = self.cache_hit_count.saturating_add(1);
            return Ok(&self.variants[index]);
        }

        self.cache_miss_count = self.cache_miss_count.saturating_add(1);
        assert!(
            self.variants.len() < REALTIME_IBL_TOPOLOGY_VARIANT_CAPACITY,
            "realtime IBL scheduler exceeded its fixed topology cache capacity"
        );
        let mut builder = RenderGraphBuilder::new("realtime-ibl-frame");
        let plan = append_realtime_ibl_graph_plan(&mut builder, request, batch)?;
        let graph = builder.compile()?;
        self.variants.push(RealtimeIblCompiledGraphVariant::new(
            request, batch, plan, graph,
        )?);
        self.compile_count = self.compile_count.saturating_add(1);
        let last_index = self.variants.len() - 1;
        Ok(&self.variants[last_index])
    }

    pub(in crate::graphics) fn stats(&self) -> RealtimeIblCompiledGraphCacheStats {
        RealtimeIblCompiledGraphCacheStats {
            cache_hit_count: self.cache_hit_count,
            cache_miss_count: self.cache_miss_count,
            compile_count: self.compile_count,
            variant_count: self.variants.len(),
        }
    }
}
