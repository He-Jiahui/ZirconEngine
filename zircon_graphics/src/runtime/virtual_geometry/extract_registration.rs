use zircon_scene::RenderVirtualGeometryExtract;

use super::virtual_geometry_runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(crate) fn register_extract(&mut self, extract: Option<&RenderVirtualGeometryExtract>) {
        self.evictable_pages.clear();

        let Some(extract) = extract else {
            self.page_budget = 0;
            return;
        };

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
