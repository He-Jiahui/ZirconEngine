use std::collections::HashMap;

use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::{
    MaterialDrawGenerationSelection, PublishedMaterialDrawProxy, ResourceStreamer,
};

/// Per-build material generation choice, resolved before phase ordering and GPU projection.
///
/// Missing entries deliberately mean the newest published generation, keeping the steady-state
/// representation sparse. Only contexts that must retain a previous/error proxy allocate rows.
#[derive(Default)]
pub(super) struct MaterialDrawSelection {
    overrides: HashMap<ResourceId, MaterialDrawGenerationSelection>,
    error_proxy_count: usize,
}

impl MaterialDrawSelection {
    pub(super) fn proxy<'a>(
        &self,
        streamer: &'a ResourceStreamer,
        id: &ResourceId,
    ) -> PublishedMaterialDrawProxy<'a> {
        streamer.material_draw_proxy(id, self.overrides.get(id).copied().unwrap_or_default())
    }

    pub(super) fn select(&mut self, id: ResourceId, selection: MaterialDrawGenerationSelection) {
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

    pub(super) fn selection_for(&self, id: &ResourceId) -> MaterialDrawGenerationSelection {
        self.overrides.get(id).copied().unwrap_or_default()
    }

    pub(super) fn has_overrides(&self) -> bool {
        !self.overrides.is_empty()
    }

    pub(super) fn has_previous_proxies(&self) -> bool {
        self.overrides.len() > self.error_proxy_count
    }

    pub(super) const fn has_error_proxies(&self) -> bool {
        self.error_proxy_count != 0
    }
}

#[cfg(test)]
mod tests {
    use crate::core::resource::ResourceId;
    use crate::graphics::scene::resources::MaterialDrawGenerationSelection;

    use super::MaterialDrawSelection;

    #[test]
    fn current_generation_is_implicit_and_fallback_rows_are_sparse() {
        let id = ResourceId::from_stable_label("res://tests/material-selection");
        let mut selection = MaterialDrawSelection::default();

        assert!(selection.overrides.is_empty());
        selection.select(id, MaterialDrawGenerationSelection::PreviousPublished);
        assert_eq!(selection.overrides.len(), 1);
        assert!(selection.has_previous_proxies());
        assert!(!selection.has_error_proxies());
        selection.select(id, MaterialDrawGenerationSelection::ErrorProxy);
        assert!(!selection.has_previous_proxies());
        assert!(selection.has_error_proxies());
        selection.select(id, MaterialDrawGenerationSelection::Published);
        assert!(selection.overrides.is_empty());
        assert!(!selection.has_error_proxies());
    }
}
