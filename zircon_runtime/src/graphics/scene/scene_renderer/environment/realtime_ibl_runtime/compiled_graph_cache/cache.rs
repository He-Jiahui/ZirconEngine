use crate::core::framework::render::IblBakeArtifactRequest;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_graph_plan::append_realtime_ibl_graph_plan;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_time_slice::RealtimeIblFrameBatch;
use crate::render_graph::{RenderGraphBuilder, RenderGraphError};

use super::variant::RealtimeIblCompiledGraphVariant;

// The fixed eight-mip, two-face scheduler has 17 operation topologies and two
// work-slot choices. Updating that scheduler requires revisiting this bound.
const REALTIME_IBL_TOPOLOGY_VARIANT_CAPACITY: usize = 34;

pub(in crate::graphics) struct RealtimeIblCompiledGraphCache {
    variants: Vec<RealtimeIblCompiledGraphVariant>,
    #[cfg(test)]
    compile_count: usize,
}

impl RealtimeIblCompiledGraphCache {
    pub(in crate::graphics) fn new() -> Self {
        Self {
            variants: Vec::with_capacity(REALTIME_IBL_TOPOLOGY_VARIANT_CAPACITY),
            #[cfg(test)]
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
            return Ok(&self.variants[index]);
        }

        assert!(
            self.variants.len() < REALTIME_IBL_TOPOLOGY_VARIANT_CAPACITY,
            "realtime IBL scheduler exceeded its fixed topology cache capacity"
        );
        let mut builder = RenderGraphBuilder::new("realtime-ibl-frame");
        let plan = append_realtime_ibl_graph_plan(&mut builder, request, batch)?;
        let graph = builder.compile()?;
        self.variants.push(RealtimeIblCompiledGraphVariant::new(
            request, batch, plan, graph,
        ));
        self.record_compile();
        Ok(self
            .variants
            .last()
            .expect("compiled realtime IBL graph variant was just inserted"))
    }

    #[cfg(test)]
    pub(in crate::graphics) fn variant_count(&self) -> usize {
        self.variants.len()
    }

    #[cfg(test)]
    pub(in crate::graphics) fn compile_count(&self) -> usize {
        self.compile_count
    }

    #[cfg(test)]
    fn record_compile(&mut self) {
        self.compile_count += 1;
    }

    #[cfg(not(test))]
    fn record_compile(&mut self) {}
}
