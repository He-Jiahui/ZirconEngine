use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use zircon_math::view_matrix;
use zircon_scene::{
    ProjectionMode, RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
    ViewportCameraSnapshot,
};

use super::super::culling::{
    orthographic_visible::orthographic_visible, perspective_visible::perspective_visible,
};
use super::super::declarations::{
    VisibilityHistorySnapshot, VisibilityVirtualGeometryCluster, VisibilityVirtualGeometryFeedback,
    VisibilityVirtualGeometryPageUploadPlan,
};

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

    let visible_clusters = extract
        .clusters
        .iter()
        .filter(|cluster| visible_entities.contains(&cluster.entity))
        .filter(|cluster| cluster_visible(cluster, camera))
        .copied()
        .collect::<Vec<_>>();
    let visible_clusters =
        refine_visible_cluster_frontier(&visible_clusters, extract.cluster_budget as usize);

    let virtual_geometry_visible_clusters = visible_clusters
        .iter()
        .map(|cluster| VisibilityVirtualGeometryCluster {
            entity: cluster.entity,
            cluster_id: cluster.cluster_id,
            page_id: cluster.page_id,
            lod_level: cluster.lod_level,
            resident: resident_page_set.contains(&cluster.page_id),
        })
        .collect::<Vec<_>>();

    let requested_pages = unique_pages(
        virtual_geometry_visible_clusters
            .iter()
            .filter(|cluster| !cluster.resident)
            .map(|cluster| cluster.page_id),
        extract.page_budget as usize,
    );
    let previous_requested_pages = previous
        .map(|history| {
            history
                .virtual_geometry_requested_pages
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let dirty_requested_pages = requested_pages
        .iter()
        .copied()
        .filter(|page_id| !previous_requested_pages.contains(page_id))
        .collect::<Vec<_>>();
    let visible_page_set = virtual_geometry_visible_clusters
        .iter()
        .map(|cluster| cluster.page_id)
        .collect::<BTreeSet<_>>();
    let evictable_pages = resident_pages
        .iter()
        .copied()
        .filter(|page_id| !visible_page_set.contains(page_id))
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

fn cluster_visible(
    cluster: &RenderVirtualGeometryCluster,
    camera: &ViewportCameraSnapshot,
) -> bool {
    let view_position = view_matrix(camera.transform).transform_point3(cluster.bounds_center);
    let depth = -view_position.z;
    let near = camera.z_near.max(0.001);
    let far = camera.z_far.max(near);
    let radius = cluster.bounds_radius.max(0.0);

    if depth + radius < near || depth - radius > far {
        return false;
    }

    match camera.projection_mode {
        ProjectionMode::Perspective => perspective_visible(view_position, depth, radius, camera),
        ProjectionMode::Orthographic => orthographic_visible(view_position, radius, camera),
    }
}

fn virtual_geometry_cluster_sort_key(
    left: &RenderVirtualGeometryCluster,
    right: &RenderVirtualGeometryCluster,
) -> Ordering {
    right
        .screen_space_error
        .partial_cmp(&left.screen_space_error)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.lod_level.cmp(&right.lod_level))
        .then_with(|| left.cluster_id.cmp(&right.cluster_id))
}

fn unique_pages(pages: impl IntoIterator<Item = u32>, budget: usize) -> Vec<u32> {
    if budget == 0 {
        return Vec::new();
    }

    let mut seen = BTreeSet::new();
    let mut unique = Vec::new();
    for page_id in pages {
        if seen.insert(page_id) {
            unique.push(page_id);
            if unique.len() == budget {
                break;
            }
        }
    }
    unique
}

fn refine_visible_cluster_frontier(
    visible_clusters: &[RenderVirtualGeometryCluster],
    cluster_budget: usize,
) -> Vec<RenderVirtualGeometryCluster> {
    if cluster_budget == 0 || visible_clusters.is_empty() {
        return Vec::new();
    }

    let visible_by_id = visible_clusters
        .iter()
        .map(|cluster| (cluster.cluster_id, *cluster))
        .collect::<BTreeMap<_, _>>();
    let mut children_by_parent = BTreeMap::<u32, Vec<RenderVirtualGeometryCluster>>::new();
    let mut frontier = visible_clusters
        .iter()
        .copied()
        .filter(|cluster| {
            cluster
                .parent_cluster_id
                .and_then(|parent| visible_by_id.get(&parent))
                .is_none()
        })
        .collect::<Vec<_>>();

    for cluster in visible_clusters.iter().copied() {
        if let Some(parent_cluster_id) = cluster.parent_cluster_id {
            if visible_by_id.contains_key(&parent_cluster_id) {
                children_by_parent
                    .entry(parent_cluster_id)
                    .or_default()
                    .push(cluster);
            }
        }
    }

    frontier.sort_by(virtual_geometry_cluster_sort_key);
    frontier.truncate(cluster_budget);

    loop {
        frontier.sort_by(virtual_geometry_cluster_sort_key);
        let mut refined = false;

        for index in 0..frontier.len() {
            let cluster = frontier[index];
            let mut children = children_by_parent
                .get(&cluster.cluster_id)
                .cloned()
                .unwrap_or_default();
            if children.is_empty() {
                continue;
            }

            children.sort_by(virtual_geometry_cluster_sort_key);
            let proposed_len = frontier.len() - 1 + children.len();
            if proposed_len > cluster_budget {
                continue;
            }

            frontier.remove(index);
            frontier.extend(children);
            refined = true;
            break;
        }

        if !refined {
            break;
        }
    }

    frontier.sort_by(virtual_geometry_cluster_sort_key);
    frontier.truncate(cluster_budget);
    frontier
}
