use std::collections::{BTreeMap, BTreeSet};

use zircon_scene::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract, ViewportCameraSnapshot,
};

use super::super::super::declarations::{
    VisibilityHistorySnapshot, VisibilityVirtualGeometryCluster,
    VisibilityVirtualGeometryDrawSegment, VisibilityVirtualGeometryFeedback,
    VisibilityVirtualGeometryPageUploadPlan,
};
use super::frontier::{refine_visible_cluster_frontier, unique_pages};
use super::ordering::{virtual_geometry_cluster_count, virtual_geometry_cluster_ordinal};
use super::visibility::cluster_visible;

pub(crate) fn build_virtual_geometry_plan(
    extract: Option<&RenderVirtualGeometryExtract>,
    visible_entities: &BTreeSet<u64>,
    camera: &ViewportCameraSnapshot,
    previous: Option<&VisibilityHistorySnapshot>,
) -> (
    Vec<VisibilityVirtualGeometryCluster>,
    Vec<VisibilityVirtualGeometryDrawSegment>,
    VisibilityVirtualGeometryPageUploadPlan,
    VisibilityVirtualGeometryFeedback,
    Vec<u32>,
) {
    let Some(extract) = extract else {
        return (
            Vec::new(),
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
    let clusters_by_id = candidate_visible_clusters
        .iter()
        .copied()
        .map(|cluster| (cluster.cluster_id, cluster))
        .collect::<BTreeMap<_, _>>();
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
    let virtual_geometry_draw_segments =
        build_visibility_owned_draw_segments(extract, &visible_clusters);
    let visible_page_set = virtual_geometry_visible_clusters
        .iter()
        .map(|cluster| cluster.page_id)
        .collect::<BTreeSet<_>>();
    let visible_cluster_id_set = virtual_geometry_visible_clusters
        .iter()
        .map(|cluster| cluster.cluster_id)
        .collect::<BTreeSet<_>>();

    let hierarchy_cascade_requested_pages = candidate_visible_clusters
        .iter()
        .filter(|cluster| previous_visible_cluster_ids.contains(&cluster.cluster_id))
        .filter(|cluster| !visible_cluster_id_set.contains(&cluster.cluster_id))
        .filter_map(|cluster| {
            highest_nonresident_ancestor_page_before_visible(
                *cluster,
                &clusters_by_id,
                &resident_page_set,
                &visible_cluster_id_set,
            )
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let continued_requested_pages = previous_requested_page_ids
        .iter()
        .copied()
        .filter(|page_id| !resident_page_set.contains(page_id))
        .filter(|page_id| {
            requested_page_reaches_visible_frontier(
                *page_id,
                &candidate_visible_clusters,
                &clusters_by_id,
                &visible_cluster_id_set,
            )
        })
        .collect::<Vec<_>>();
    let requested_pages = prioritized_requested_pages(
        hierarchy_cascade_requested_pages
            .into_iter()
            .chain(continued_requested_pages)
            .collect(),
        unique_pages(
            &streaming_target_clusters,
            &resident_page_set,
            extract.page_budget as usize,
        ),
        extract.page_budget as usize,
    );
    let dirty_requested_pages = requested_pages
        .iter()
        .copied()
        .filter(|page_id| !previous_requested_page_ids.contains(page_id))
        .collect::<Vec<_>>();
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
        .filter(|cluster| has_visible_ancestor(**cluster, &clusters_by_id, &visible_cluster_id_set))
        .map(|cluster| cluster.page_id)
        .collect::<BTreeSet<_>>();
    let requested_frontier_cluster_ids = requested_frontier_cluster_ids(
        &requested_pages,
        &candidate_visible_clusters,
        &clusters_by_id,
        &visible_cluster_id_set,
    );
    let cascade_requested_frontier_hold_protected_pages = candidate_visible_clusters
        .iter()
        .filter(|cluster| resident_page_set.contains(&cluster.page_id))
        .filter(|cluster| !visible_cluster_id_set.contains(&cluster.cluster_id))
        .filter(|cluster| {
            visible_frontier_cluster_id_for_cluster(
                **cluster,
                &clusters_by_id,
                &visible_cluster_id_set,
            )
            .is_some_and(|frontier_cluster_id| {
                requested_frontier_cluster_ids.contains(&frontier_cluster_id)
            })
        })
        .filter(|cluster| {
            !has_hidden_resident_descendant(
                **cluster,
                &clusters_by_id,
                &resident_page_set,
                &visible_cluster_id_set,
            )
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
        .filter(|page_id| !cascade_requested_frontier_hold_protected_pages.contains(page_id))
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
        virtual_geometry_draw_segments,
        page_upload_plan,
        feedback,
        requested_pages,
    )
}

fn build_visibility_owned_draw_segments(
    extract: &RenderVirtualGeometryExtract,
    visible_clusters: &[RenderVirtualGeometryCluster],
) -> Vec<VisibilityVirtualGeometryDrawSegment> {
    let mut draw_segments: Vec<VisibilityVirtualGeometryDrawSegment> = Vec::new();
    let mut last_parent_cluster_id = None::<Option<u32>>;

    for cluster in visible_clusters {
        let cluster_ordinal = virtual_geometry_cluster_ordinal(extract, cluster);
        let cluster_count = virtual_geometry_cluster_count(extract, cluster.entity);
        if let Some(previous) = draw_segments.last_mut() {
            let previous_end = previous
                .cluster_ordinal
                .saturating_add(previous.cluster_span_count);
            let same_visibility_segment = previous.entity == cluster.entity
                && previous.page_id == cluster.page_id
                && previous.cluster_count == cluster_count
                && previous.lod_level == cluster.lod_level
                && previous_end == cluster_ordinal
                && last_parent_cluster_id == Some(cluster.parent_cluster_id);
            if same_visibility_segment {
                previous.cluster_span_count = previous.cluster_span_count.saturating_add(1);
                continue;
            }
        }

        draw_segments.push(VisibilityVirtualGeometryDrawSegment {
            entity: cluster.entity,
            cluster_id: cluster.cluster_id,
            page_id: cluster.page_id,
            cluster_ordinal,
            cluster_span_count: 1,
            cluster_count,
            lod_level: cluster.lod_level,
        });
        last_parent_cluster_id = Some(cluster.parent_cluster_id);
    }

    draw_segments
}

fn prioritized_requested_pages(
    cascade_requests: Vec<u32>,
    ranked_requests: Vec<u32>,
    budget: usize,
) -> Vec<u32> {
    if budget == 0 {
        return Vec::new();
    }

    let mut requested_pages = Vec::with_capacity(budget);
    for page_id in cascade_requests.into_iter().chain(ranked_requests) {
        if requested_pages.contains(&page_id) {
            continue;
        }
        requested_pages.push(page_id);
        if requested_pages.len() >= budget {
            break;
        }
    }
    requested_pages
}

fn requested_page_reaches_visible_frontier(
    page_id: u32,
    candidate_visible_clusters: &[RenderVirtualGeometryCluster],
    clusters_by_id: &BTreeMap<u32, RenderVirtualGeometryCluster>,
    visible_cluster_id_set: &BTreeSet<u32>,
) -> bool {
    candidate_visible_clusters
        .iter()
        .copied()
        .filter(|cluster| cluster.page_id == page_id)
        .any(|cluster| {
            visible_frontier_cluster_id_for_cluster(cluster, clusters_by_id, visible_cluster_id_set)
                .is_some()
        })
}

fn requested_frontier_cluster_ids(
    requested_pages: &[u32],
    candidate_visible_clusters: &[RenderVirtualGeometryCluster],
    clusters_by_id: &BTreeMap<u32, RenderVirtualGeometryCluster>,
    visible_cluster_id_set: &BTreeSet<u32>,
) -> BTreeSet<u32> {
    requested_pages
        .iter()
        .flat_map(|page_id| {
            candidate_visible_clusters
                .iter()
                .copied()
                .filter(move |cluster| cluster.page_id == *page_id)
        })
        .filter_map(|cluster| {
            visible_frontier_cluster_id_for_cluster(cluster, clusters_by_id, visible_cluster_id_set)
        })
        .collect()
}

fn has_visible_ancestor(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &BTreeMap<u32, RenderVirtualGeometryCluster>,
    visible_cluster_id_set: &BTreeSet<u32>,
) -> bool {
    let mut current_parent_cluster_id = cluster.parent_cluster_id;
    let mut visited_cluster_ids = BTreeSet::new();

    while let Some(parent_cluster_id) = current_parent_cluster_id {
        if !visited_cluster_ids.insert(parent_cluster_id) {
            break;
        }
        if visible_cluster_id_set.contains(&parent_cluster_id) {
            return true;
        }
        current_parent_cluster_id = clusters_by_id
            .get(&parent_cluster_id)
            .and_then(|parent| parent.parent_cluster_id);
    }

    false
}

fn visible_frontier_cluster_id_for_cluster(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &BTreeMap<u32, RenderVirtualGeometryCluster>,
    visible_cluster_id_set: &BTreeSet<u32>,
) -> Option<u32> {
    let mut current_parent_cluster_id = cluster.parent_cluster_id;
    let mut visited_cluster_ids = BTreeSet::new();

    while let Some(parent_cluster_id) = current_parent_cluster_id {
        if !visited_cluster_ids.insert(parent_cluster_id) {
            break;
        }
        if visible_cluster_id_set.contains(&parent_cluster_id) {
            return Some(parent_cluster_id);
        }
        current_parent_cluster_id = clusters_by_id
            .get(&parent_cluster_id)
            .and_then(|parent| parent.parent_cluster_id);
    }

    None
}

fn has_hidden_resident_descendant(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &BTreeMap<u32, RenderVirtualGeometryCluster>,
    resident_page_set: &BTreeSet<u32>,
    visible_cluster_id_set: &BTreeSet<u32>,
) -> bool {
    let mut stack = clusters_by_id
        .values()
        .filter(|candidate| candidate.parent_cluster_id == Some(cluster.cluster_id))
        .copied()
        .collect::<Vec<_>>();
    let mut visited_cluster_ids = BTreeSet::new();

    while let Some(candidate_cluster) = stack.pop() {
        if !visited_cluster_ids.insert(candidate_cluster.cluster_id) {
            continue;
        }
        if resident_page_set.contains(&candidate_cluster.page_id)
            && !visible_cluster_id_set.contains(&candidate_cluster.cluster_id)
        {
            return true;
        }
        stack.extend(
            clusters_by_id
                .values()
                .filter(|child| child.parent_cluster_id == Some(candidate_cluster.cluster_id))
                .copied(),
        );
    }

    false
}

fn highest_nonresident_ancestor_page_before_visible(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &BTreeMap<u32, RenderVirtualGeometryCluster>,
    resident_page_set: &BTreeSet<u32>,
    visible_cluster_id_set: &BTreeSet<u32>,
) -> Option<u32> {
    let mut current_parent_cluster_id = cluster.parent_cluster_id;
    let mut visited_cluster_ids = BTreeSet::new();
    let mut highest_missing_ancestor_page_id = None;

    while let Some(parent_cluster_id) = current_parent_cluster_id {
        if !visited_cluster_ids.insert(parent_cluster_id) {
            break;
        }
        let Some(parent_cluster) = clusters_by_id.get(&parent_cluster_id) else {
            break;
        };
        if !resident_page_set.contains(&parent_cluster.page_id) {
            highest_missing_ancestor_page_id = Some(parent_cluster.page_id);
        }
        if visible_cluster_id_set.contains(&parent_cluster_id) {
            return highest_missing_ancestor_page_id;
        }
        current_parent_cluster_id = parent_cluster.parent_cluster_id;
    }

    None
}
