use std::collections::{BTreeMap, HashMap, HashSet};

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract, RenderVirtualGeometryPageDependency,
};

use super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(crate) fn register_extract(&mut self, extract: Option<&RenderVirtualGeometryExtract>) {
        self.clear_evictable_pages();

        let Some(extract) = extract else {
            *self = Self::default();
            return;
        };

        let live_page_ids = live_page_id_index(extract);
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

// 存在 cooked 页依赖时以其为准；仅无依赖清单才从 cluster 父链恢复页关系。
fn page_parent_pages(extract: &RenderVirtualGeometryExtract) -> BTreeMap<u32, u32> {
    // Cooked page dependencies are authoritative even when they describe a flat root-only graph.
    if !extract.page_dependencies.is_empty() {
        return cooked_page_parent_pages(extract);
    }

    let mut clusters_by_id = HashMap::with_capacity(extract.clusters.len());
    for &cluster in &extract.clusters {
        clusters_by_id.insert(cluster.cluster_id, cluster);
    }
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

fn cooked_page_parent_pages(extract: &RenderVirtualGeometryExtract) -> BTreeMap<u32, u32> {
    let live_page_ids = live_page_id_index(extract);
    let mut page_parent_pages = BTreeMap::new();

    for dependency in &extract.page_dependencies {
        insert_cooked_page_parent_link(dependency, &live_page_ids, &mut page_parent_pages);
        for child_page_id in &dependency.child_page_ids {
            if live_page_ids.contains(child_page_id) && *child_page_id != dependency.page_id {
                page_parent_pages
                    .entry(*child_page_id)
                    .or_insert(dependency.page_id);
            }
        }
    }

    page_parent_pages
}

fn insert_cooked_page_parent_link(
    dependency: &RenderVirtualGeometryPageDependency,
    live_page_ids: &HashSet<u32>,
    page_parent_pages: &mut BTreeMap<u32, u32>,
) {
    let Some(parent_page_id) = dependency.parent_page_id else {
        return;
    };
    if dependency.page_id == parent_page_id {
        return;
    }
    if live_page_ids.contains(&dependency.page_id) && live_page_ids.contains(&parent_page_id) {
        page_parent_pages.insert(dependency.page_id, parent_page_id);
    }
}

fn nearest_distinct_parent_page(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &HashMap<u32, RenderVirtualGeometryCluster>,
) -> Option<u32> {
    let mut current_parent_cluster_id = cluster.parent_cluster_id;
    let mut visited_cluster_ids = HashSet::new();

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

fn live_page_id_index(extract: &RenderVirtualGeometryExtract) -> HashSet<u32> {
    let mut live_page_ids = HashSet::with_capacity(extract.pages.len());
    live_page_ids.extend(extract.pages.iter().map(|page| page.page_id));
    live_page_ids
}

#[cfg(test)]
#[path = "extract_registration/tests/performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "tests/extract_registration.rs"]
mod tests;
