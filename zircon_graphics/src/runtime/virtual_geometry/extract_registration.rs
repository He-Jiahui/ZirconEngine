use std::collections::BTreeSet;

use zircon_scene::RenderVirtualGeometryExtract;

use super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(crate) fn register_extract(&mut self, extract: Option<&RenderVirtualGeometryExtract>) {
        self.evictable_pages.clear();

        let Some(extract) = extract else {
            *self = Self::default();
            return;
        };

        let live_page_ids = extract
            .pages
            .iter()
            .map(|page| page.page_id)
            .collect::<BTreeSet<_>>();
        let stale_resident_page_ids = self
            .resident_slots
            .keys()
            .copied()
            .filter(|page_id| !live_page_ids.contains(page_id))
            .collect::<Vec<_>>();
        for page_id in stale_resident_page_ids {
            self.evict_page(page_id);
        }
        self.pending_pages
            .retain(|page_id| live_page_ids.contains(page_id));
        self.pending_requests
            .retain(|request| live_page_ids.contains(&request.page_id));
        self.page_sizes
            .retain(|page_id, _| live_page_ids.contains(page_id));

        self.page_budget = (extract.page_budget as usize)
            .max(extract.pages.iter().filter(|page| page.resident).count());

        for page in &extract.pages {
            self.page_sizes.insert(page.page_id, page.size_bytes);
            if page.resident {
                self.promote_to_resident(page.page_id);
            }
        }
    }
}
