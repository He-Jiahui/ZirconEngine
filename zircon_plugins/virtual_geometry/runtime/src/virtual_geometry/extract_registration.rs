use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
};

use super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(crate) fn register_extract(&mut self, extract: Option<&RenderVirtualGeometryExtract>) {
        self.clear_evictable_pages();

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
            .resident_page_ids()
            .filter(|page_id| !live_page_ids.contains(page_id))
            .collect::<Vec<_>>();
        for page_id in stale_resident_page_ids {
            self.evict_page(page_id);
        }
        self.retain_pending_pages(|page_id| live_page_ids.contains(page_id));
        self.retain_pending_page_requests(|request| live_page_ids.contains(&request.page_id()));
        self.retain_current_requested_page_order(|page_id| live_page_ids.contains(page_id));
        self.retain_current_hot_resident_pages(|page_id| live_page_ids.contains(page_id));
        self.retain_recent_hot_resident_pages(|page_id, _| live_page_ids.contains(page_id));
        self.retain_page_sizes(|page_id| live_page_ids.contains(page_id));
        self.retain_page_parent_pages(|page_id, parent_page_id| {
            live_page_ids.contains(page_id) && live_page_ids.contains(parent_page_id)
        });

        self.set_page_budget(
            (extract.page_budget as usize)
                .max(extract.pages.iter().filter(|page| page.resident).count()),
        );
        self.replace_page_parent_pages(page_parent_pages(extract));

        for page in &extract.pages {
            self.insert_page_size(page.page_id, page.size_bytes);
            if page.resident {
                self.promote_to_resident(page.page_id);
            }
        }
    }
}

fn page_parent_pages(extract: &RenderVirtualGeometryExtract) -> BTreeMap<u32, u32> {
    let clusters_by_id = extract
        .clusters
        .iter()
        .copied()
        .map(|cluster| (cluster.cluster_id, cluster))
        .collect::<BTreeMap<_, _>>();
    let mut page_parent_pages = BTreeMap::new();

    for cluster in &extract.clusters {
        if page_parent_pages.contains_key(&cluster.page_id) {
            continue;
        }

        if let Some(parent_page_id) = nearest_distinct_parent_page(*cluster, &clusters_by_id) {
            page_parent_pages.insert(cluster.page_id, parent_page_id);
        }
    }

    page_parent_pages
}

fn nearest_distinct_parent_page(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &BTreeMap<u32, RenderVirtualGeometryCluster>,
) -> Option<u32> {
    let mut current_parent_cluster_id = cluster.parent_cluster_id;
    let mut visited_cluster_ids = BTreeSet::new();

    while let Some(parent_cluster_id) = current_parent_cluster_id {
        if !visited_cluster_ids.insert(parent_cluster_id) {
            break;
        }
        let parent_cluster = clusters_by_id.get(&parent_cluster_id)?;
        if parent_cluster.page_id != cluster.page_id {
            return Some(parent_cluster.page_id);
        }
        current_parent_cluster_id = parent_cluster.parent_cluster_id;
    }

    None
}
