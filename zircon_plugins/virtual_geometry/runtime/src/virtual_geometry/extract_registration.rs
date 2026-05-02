use std::collections::{BTreeMap, BTreeSet};

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
    // Cooked page dependencies are authoritative even when they describe a flat root-only graph.
    if !extract.page_dependencies.is_empty() {
        return cooked_page_parent_pages(extract);
    }

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

fn cooked_page_parent_pages(extract: &RenderVirtualGeometryExtract) -> BTreeMap<u32, u32> {
    let live_page_ids = extract
        .pages
        .iter()
        .map(|page| page.page_id)
        .collect::<BTreeSet<_>>();
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
    live_page_ids: &BTreeSet<u32>,
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use zircon_runtime::core::framework::render::{
        RenderVirtualGeometryPage, RenderVirtualGeometryPageDependency,
    };

    use super::*;

    #[test]
    fn cooked_page_dependencies_drive_runtime_page_parent_map() {
        let extract = RenderVirtualGeometryExtract {
            pages: vec![page(10), page(20), page(30)],
            page_dependencies: vec![
                RenderVirtualGeometryPageDependency {
                    page_id: 10,
                    parent_page_id: None,
                    child_page_ids: vec![20],
                },
                RenderVirtualGeometryPageDependency {
                    page_id: 20,
                    parent_page_id: Some(10),
                    child_page_ids: Vec::new(),
                },
                RenderVirtualGeometryPageDependency {
                    page_id: 30,
                    parent_page_id: None,
                    child_page_ids: Vec::new(),
                },
            ],
            ..RenderVirtualGeometryExtract::default()
        };

        assert_eq!(page_parent_pages(&extract), BTreeMap::from([(20, 10)]));
    }

    #[test]
    fn cooked_page_dependencies_can_recover_parent_links_from_child_rows() {
        let extract = RenderVirtualGeometryExtract {
            pages: vec![page(10), page(20)],
            page_dependencies: vec![RenderVirtualGeometryPageDependency {
                page_id: 10,
                parent_page_id: None,
                child_page_ids: vec![20],
            }],
            ..RenderVirtualGeometryExtract::default()
        };

        assert_eq!(page_parent_pages(&extract), BTreeMap::from([(20, 10)]));
    }

    #[test]
    fn empty_cooked_page_dependency_graph_suppresses_cluster_lineage_fallback() {
        let extract = RenderVirtualGeometryExtract {
            clusters: vec![cluster(1, 10, None), cluster(2, 20, Some(1))],
            pages: vec![page(10), page(20)],
            page_dependencies: vec![RenderVirtualGeometryPageDependency {
                page_id: 10,
                parent_page_id: None,
                child_page_ids: Vec::new(),
            }],
            ..RenderVirtualGeometryExtract::default()
        };

        assert_eq!(page_parent_pages(&extract), BTreeMap::new());
    }

    #[test]
    fn cluster_lineage_remains_fallback_when_cooked_page_dependencies_are_absent() {
        let extract = RenderVirtualGeometryExtract {
            clusters: vec![
                cluster(1, 10, None),
                cluster(2, 20, Some(1)),
                cluster(3, 30, Some(2)),
            ],
            pages: vec![page(10), page(20), page(30)],
            ..RenderVirtualGeometryExtract::default()
        };

        assert_eq!(
            page_parent_pages(&extract),
            BTreeMap::from([(20, 10), (30, 20)])
        );
    }

    fn page(page_id: u32) -> RenderVirtualGeometryPage {
        RenderVirtualGeometryPage {
            page_id,
            resident: false,
            size_bytes: 1,
        }
    }

    fn cluster(
        cluster_id: u32,
        page_id: u32,
        parent_cluster_id: Option<u32>,
    ) -> RenderVirtualGeometryCluster {
        RenderVirtualGeometryCluster {
            cluster_id,
            page_id,
            parent_cluster_id,
            ..RenderVirtualGeometryCluster::default()
        }
    }
}
