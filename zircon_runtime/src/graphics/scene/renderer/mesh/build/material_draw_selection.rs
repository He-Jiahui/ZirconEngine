use std::collections::HashMap;

use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::{
    MaterialDrawGenerationSelection, PublishedMaterialDrawProxy, ResourceStreamer,
};

/// Per-build material generation choice, resolved before phase ordering and GPU projection.
///
/// Missing entries deliberately mean the newest published generation, keeping the steady-state
/// representation sparse. Previous/error choices and admitted cold candidates allocate rows.
#[derive(Default)]
pub(super) struct MaterialDrawSelection {
    overrides: HashMap<ResourceId, MaterialDrawGenerationSelection>,
    admitted_cold_staged_generations: HashMap<ResourceId, u64>,
    error_proxy_count: usize,
}

impl MaterialDrawSelection {
    pub(super) fn proxy<'a>(
        &self,
        streamer: &'a ResourceStreamer,
        id: &ResourceId,
    ) -> PublishedMaterialDrawProxy<'a> {
        if let Some(&generation) = self.admitted_cold_staged_generations.get(id) {
            return streamer.admitted_cold_staged_material_draw_proxy(id, generation);
        }
        streamer.material_draw_proxy(id, self.overrides.get(id).copied().unwrap_or_default())
    }

    pub(super) fn select(&mut self, id: ResourceId, selection: MaterialDrawGenerationSelection) {
        self.admitted_cold_staged_generations.remove(&id);
        let previous = if selection == MaterialDrawGenerationSelection::Published {
            self.overrides.remove(&id)
        } else {
            self.overrides.insert(id, selection)
        };
        if previous == Some(MaterialDrawGenerationSelection::ErrorProxy) {
            self.error_proxy_count = self.error_proxy_count.saturating_sub(1);
        }
        if selection == MaterialDrawGenerationSelection::ErrorProxy {
            self.error_proxy_count = self.error_proxy_count.saturating_add(1);
        }
    }

    pub(super) fn select_admitted_cold_staged(&mut self, id: ResourceId, generation: u64) {
        self.select(id, MaterialDrawGenerationSelection::Published);
        self.admitted_cold_staged_generations.insert(id, generation);
    }

    pub(super) fn selection_for(&self, id: &ResourceId) -> MaterialDrawGenerationSelection {
        self.overrides.get(id).copied().unwrap_or_default()
    }

    pub(super) fn has_overrides(&self) -> bool {
        !self.overrides.is_empty() || !self.admitted_cold_staged_generations.is_empty()
    }

    pub(super) fn has_previous_proxies(&self) -> bool {
        self.overrides.len() > self.error_proxy_count
    }

    pub(super) const fn has_error_proxies(&self) -> bool {
        self.error_proxy_count != 0
    }
}

#[cfg(test)]
#[path = "../../../mesh/build/tests/material_draw_selection.rs"]
mod tests;
