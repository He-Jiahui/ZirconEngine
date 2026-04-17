use std::collections::{BTreeMap, BTreeSet};

use zircon_scene::{RenderVirtualGeometryExtract, ViewportCameraSnapshot};

use super::super::super::declarations::{
    VisibilityHistorySnapshot, VisibilityVirtualGeometryCluster, VisibilityVirtualGeometryFeedback,
    VisibilityVirtualGeometryPageUploadPlan,
};
use super::cluster_visible::cluster_visible;
use super::refine_visible_cluster_frontier::refine_visible_cluster_frontier;
use super::unique_pages::unique_pages;
use super::virtual_geometry_cluster_count::virtual_geometry_cluster_count;
use super::virtual_geometry_cluster_ordinal::virtual_geometry_cluster_ordinal;

pub(crate) fn build_virtual_geometry_plan(
    extract: Option<&RenderVirtualGeometryExtract>,
    visible_entities: &BTreeSet<u64>,
    camera: &ViewportCameraSnapshot,
    previous: Option<&VisibilityHistorySnapshot>,
) -> (
    Vec<VisibilityVirtualGeometryCluster>,
    VisibilityVirtualGeometryPageUploadPlan,
    VisibilityVirtualGeometryFeedback,
    Vec<u32>,
) {
    let Some(extract) = extract else {
        return (
            Vec::new(),
            VisibilityVirtualGeometryPageUploadPlan::default(),
            VisibilityVirtualGeometryFeedback::default(),
            Vec::new(),
        );
    };

    let resident_pages = extract
        .pages
        .iter()
        .filter(|page| page.resident)
        .map(|page| page.page_id)
        .collect::<Vec<_>>();
    let resident_page_set = resident_pages.iter().copied().collect::<BTreeSet<_>>();
    let previous_visible_cluster_ids = previous
        .map(|history| {
            history
                .virtual_geometry_visible_cluster_ids
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let previous_requested_page_ids = previous
        .map(|history| {
            history
                .virtual_geometry_requested_pages
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let candidate_visible_clusters = extract
        .clusters
        .iter()
        .filter(|cluster| visible_entities.contains(&cluster.entity))
        .filter(|cluster| cluster_visible(cluster, camera))
        .copied()
        .collect::<Vec<_>>();
    let streaming_target_clusters = refine_visible_cluster_frontier(
        &candidate_visible_clusters,
        extract.cluster_budget as usize,
        None,
        None,
        None,
    );
    let visible_clusters = refine_visible_cluster_frontier(
        &candidate_visible_clusters,
        extract.cluster_budget as usize,
        Some(&resident_page_set),
        Some(&previous_visible_cluster_ids),
        Some(&previous_requested_page_ids),
    );

    let virtual_geometry_visible_clusters = visible_clusters
        .iter()
        .map(|cluster| VisibilityVirtualGeometryCluster {
            entity: cluster.entity,
            cluster_id: cluster.cluster_id,
            page_id: cluster.page_id,
            lod_level: cluster.lod_level,
            cluster_ordinal: virtual_geometry_cluster_ordinal(extract, cluster),
            cluster_count: virtual_geometry_cluster_count(extract, cluster.entity),
            resident: resident_page_set.contains(&cluster.page_id),
        })
        .collect::<Vec<_>>();

    let requested_pages = unique_pages(
        &streaming_target_clusters,
        &resident_page_set,
        extract.page_budget as usize,
    );
    let dirty_requested_pages = requested_pages
        .iter()
        .copied()
        .filter(|page_id| !previous_requested_page_ids.contains(page_id))
        .collect::<Vec<_>>();
    let visible_page_set = virtual_geometry_visible_clusters
        .iter()
        .map(|cluster| cluster.page_id)
        .collect::<BTreeSet<_>>();
    let visible_cluster_id_set = virtual_geometry_visible_clusters
        .iter()
        .map(|cluster| cluster.cluster_id)
        .collect::<BTreeSet<_>>();
    let mut children_by_parent = BTreeMap::<u32, Vec<_>>::new();
    for cluster in candidate_visible_clusters.iter().copied() {
        if let Some(parent_cluster_id) = cluster.parent_cluster_id {
            children_by_parent
                .entry(parent_cluster_id)
                .or_default()
                .push(cluster);
        }
    }
    let split_hold_protected_pages = candidate_visible_clusters
        .iter()
        .filter(|cluster| resident_page_set.contains(&cluster.page_id))
        .filter(|cluster| previous_requested_page_ids.contains(&cluster.page_id))
        .filter(|cluster| {
            cluster.parent_cluster_id.is_some_and(|parent_cluster_id| {
                visible_cluster_id_set.contains(&parent_cluster_id)
            })
        })
        .map(|cluster| cluster.page_id)
        .collect::<BTreeSet<_>>();
    let merge_hold_protected_pages = candidate_visible_clusters
        .iter()
        .filter(|cluster| resident_page_set.contains(&cluster.page_id))
        .filter(|cluster| previous_visible_cluster_ids.contains(&cluster.cluster_id))
        .filter(|cluster| !visible_cluster_id_set.contains(&cluster.cluster_id))
        .filter(|cluster| {
            children_by_parent
                .get(&cluster.cluster_id)
                .is_some_and(|children| {
                    !children.is_empty()
                        && children
                            .iter()
                            .all(|child| visible_cluster_id_set.contains(&child.cluster_id))
                })
        })
        .map(|cluster| cluster.page_id)
        .collect::<BTreeSet<_>>();
    let merge_back_child_hold_protected_pages = candidate_visible_clusters
        .iter()
        .filter(|cluster| resident_page_set.contains(&cluster.page_id))
        .filter(|cluster| previous_visible_cluster_ids.contains(&cluster.cluster_id))
        .filter(|cluster| !visible_cluster_id_set.contains(&cluster.cluster_id))
        .filter(|cluster| {
            cluster.parent_cluster_id.is_some_and(|parent_cluster_id| {
                visible_cluster_id_set.contains(&parent_cluster_id)
            })
        })
        .map(|cluster| cluster.page_id)
        .collect::<BTreeSet<_>>();
    let evictable_pages = resident_pages
        .iter()
        .copied()
        .filter(|page_id| !visible_page_set.contains(page_id))
        .filter(|page_id| !split_hold_protected_pages.contains(page_id))
        .filter(|page_id| !merge_hold_protected_pages.contains(page_id))
        .filter(|page_id| !merge_back_child_hold_protected_pages.contains(page_id))
        .collect::<Vec<_>>();

    let page_upload_plan = VisibilityVirtualGeometryPageUploadPlan {
        resident_pages,
        requested_pages: requested_pages.clone(),
        dirty_requested_pages: dirty_requested_pages.clone(),
        evictable_pages: evictable_pages.clone(),
    };
    let feedback = VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: virtual_geometry_visible_clusters
            .iter()
            .map(|cluster| cluster.cluster_id)
            .collect(),
        requested_pages: requested_pages.clone(),
        evictable_pages: evictable_pages.clone(),
    };

    (
        virtual_geometry_visible_clusters,
        page_upload_plan,
        feedback,
        requested_pages,
    )
}
