use crate::core::framework::render::{
    ProjectionMode, RenderVirtualGeometryCluster, RenderVirtualGeometryHierarchyNode,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot, RenderVirtualGeometryPage,
    ViewportCameraSnapshot,
};
use crate::core::math::view_matrix;
use crate::graphics::types::{
    VirtualGeometryNodeAndClusterCullTraversalChildSource,
    VirtualGeometryNodeAndClusterCullTraversalOp, VirtualGeometryNodeAndClusterCullTraversalRecord,
};

pub(super) struct VirtualGeometryNodeAndClusterCullChildDecisionOutput {
    pub(super) traversal_records: Vec<VirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub(super) requested_page_ids: Vec<u32>,
}

#[cfg(test)]
pub(super) fn build_node_and_cluster_cull_child_decision_records(
    child_visit_records: &[VirtualGeometryNodeAndClusterCullTraversalRecord],
    global_state: &RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    camera: &ViewportCameraSnapshot,
    clusters: &[RenderVirtualGeometryCluster],
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
    pages: &[RenderVirtualGeometryPage],
    first_traversal_index: u32,
) -> Vec<VirtualGeometryNodeAndClusterCullTraversalRecord> {
    build_node_and_cluster_cull_child_decision_output(
        child_visit_records,
        global_state,
        camera,
        clusters,
        hierarchy_nodes,
        pages,
        first_traversal_index,
    )
    .traversal_records
}

pub(super) fn build_node_and_cluster_cull_child_decision_output(
    child_visit_records: &[VirtualGeometryNodeAndClusterCullTraversalRecord],
    global_state: &RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    camera: &ViewportCameraSnapshot,
    clusters: &[RenderVirtualGeometryCluster],
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
    pages: &[RenderVirtualGeometryPage],
    first_traversal_index: u32,
) -> VirtualGeometryNodeAndClusterCullChildDecisionOutput {
    let mut decision_records = Vec::new();
    let mut requested_page_ids = Vec::new();
    let mut traversal_index = first_traversal_index;

    for visit_record in child_visit_records.iter().filter(|record| {
        record.op == VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode
            && record.node_cluster_count > 0
    }) {
        if let Some(node) = splittable_hierarchy_node_for_visit_record(
            visit_record,
            camera,
            global_state,
            hierarchy_nodes,
            clusters,
        ) {
            decision_records.push(VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
                child_source:
                    VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
                instance_index: visit_record.instance_index,
                entity: visit_record.entity,
                cluster_array_index: visit_record.node_cluster_start,
                hierarchy_node_id: visit_record.hierarchy_node_id,
                node_cluster_start: visit_record.node_cluster_start,
                node_cluster_count: visit_record.node_cluster_count,
                child_base: node.child_base,
                child_count: node.child_count,
                traversal_index,
                cluster_budget: visit_record.cluster_budget,
                page_budget: visit_record.page_budget,
                forced_mip: visit_record.forced_mip,
            });
            traversal_index = traversal_index.saturating_add(1);
            continue;
        }

        let mut visit_store_cluster_count = 0u32;
        for cluster_offset in 0..visit_record.node_cluster_count {
            let cluster_array_index = visit_record
                .node_cluster_start
                .saturating_add(cluster_offset);
            match node_and_cluster_cull_child_cluster_decision(
                cluster_array_index,
                camera,
                global_state,
                clusters,
                pages,
                visit_record.forced_mip,
            ) {
                NodeAndClusterCullChildClusterDecision::Store => {
                    push_node_and_cluster_cull_store_cluster_record(
                        &mut decision_records,
                        visit_record,
                        cluster_array_index,
                        clusters,
                        hierarchy_nodes,
                        &mut visit_store_cluster_count,
                        &mut traversal_index,
                    );
                }
                NodeAndClusterCullChildClusterDecision::RequestPage {
                    page_id,
                    fallback_cluster_array_index,
                } => {
                    append_node_and_cluster_cull_requested_page_id(
                        &mut requested_page_ids,
                        page_id,
                        visit_record.page_budget,
                    );
                    if let Some(fallback_cluster_array_index) = fallback_cluster_array_index {
                        push_node_and_cluster_cull_store_cluster_record(
                            &mut decision_records,
                            visit_record,
                            fallback_cluster_array_index,
                            clusters,
                            hierarchy_nodes,
                            &mut visit_store_cluster_count,
                            &mut traversal_index,
                        );
                    }
                }
                NodeAndClusterCullChildClusterDecision::Skip => {}
            };
        }
    }

    VirtualGeometryNodeAndClusterCullChildDecisionOutput {
        traversal_records: decision_records,
        requested_page_ids,
    }
}

enum NodeAndClusterCullChildClusterDecision {
    Store,
    RequestPage {
        page_id: u32,
        fallback_cluster_array_index: Option<u32>,
    },
    Skip,
}

fn node_and_cluster_cull_child_cluster_decision(
    cluster_array_index: u32,
    camera: &ViewportCameraSnapshot,
    global_state: &RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    clusters: &[RenderVirtualGeometryCluster],
    pages: &[RenderVirtualGeometryPage],
    forced_mip: Option<u8>,
) -> NodeAndClusterCullChildClusterDecision {
    let cluster = clusters.get(cluster_array_index as usize);

    if let (Some(cluster), Some(forced_mip)) = (cluster, forced_mip) {
        if cluster.lod_level != forced_mip {
            return NodeAndClusterCullChildClusterDecision::Skip;
        }
    }

    if global_state.child_frustum_culling_enabled
        && cluster.is_some_and(|cluster| {
            !node_and_cluster_cull_cluster_intersects_frustum(cluster, camera)
        })
    {
        return NodeAndClusterCullChildClusterDecision::Skip;
    }

    if pages.is_empty() {
        return NodeAndClusterCullChildClusterDecision::Store;
    }

    let Some(cluster) = cluster else {
        return NodeAndClusterCullChildClusterDecision::Skip;
    };
    let effective_pages =
        node_and_cluster_cull_effective_pages(pages, global_state.cull_input.page_count);

    if effective_pages
        .iter()
        .any(|page| page.page_id == cluster.page_id && page.resident)
    {
        NodeAndClusterCullChildClusterDecision::Store
    } else {
        NodeAndClusterCullChildClusterDecision::RequestPage {
            page_id: cluster.page_id,
            fallback_cluster_array_index: node_and_cluster_cull_resident_parent_cluster_array_index(
                cluster_array_index,
                cluster,
                clusters,
                effective_pages,
                forced_mip,
            ),
        }
    }
}

fn node_and_cluster_cull_effective_pages(
    pages: &[RenderVirtualGeometryPage],
    page_count: u32,
) -> &[RenderVirtualGeometryPage] {
    &pages[..pages.len().min(page_count as usize)]
}

fn push_node_and_cluster_cull_store_cluster_record(
    decision_records: &mut Vec<VirtualGeometryNodeAndClusterCullTraversalRecord>,
    visit_record: &VirtualGeometryNodeAndClusterCullTraversalRecord,
    cluster_array_index: u32,
    clusters: &[RenderVirtualGeometryCluster],
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
    visit_store_cluster_count: &mut u32,
    traversal_index: &mut u32,
) {
    if decision_records.iter().any(|record| {
        record.op == VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster
            && record.instance_index == visit_record.instance_index
            && record.entity == visit_record.entity
            && record.cluster_array_index == cluster_array_index
    }) {
        return;
    }
    if *visit_store_cluster_count >= visit_record.cluster_budget {
        return;
    }

    let cluster_hierarchy_node_id = clusters
        .get(cluster_array_index as usize)
        .and_then(|cluster| cluster.hierarchy_node_id);
    let cluster_node_range = cluster_hierarchy_node_id.and_then(|hierarchy_node_id| {
        node_and_cluster_cull_hierarchy_node_range(
            visit_record.instance_index,
            hierarchy_node_id,
            hierarchy_nodes,
        )
    });

    decision_records.push(node_and_cluster_cull_store_cluster_record(
        visit_record,
        cluster_array_index,
        cluster_hierarchy_node_id,
        cluster_node_range,
        *traversal_index,
    ));
    *visit_store_cluster_count = (*visit_store_cluster_count).saturating_add(1);
    *traversal_index = (*traversal_index).saturating_add(1);
}

fn node_and_cluster_cull_store_cluster_record(
    visit_record: &VirtualGeometryNodeAndClusterCullTraversalRecord,
    cluster_array_index: u32,
    cluster_hierarchy_node_id: Option<u32>,
    cluster_node_range: Option<(u32, u32)>,
    traversal_index: u32,
) -> VirtualGeometryNodeAndClusterCullTraversalRecord {
    let (node_cluster_start, node_cluster_count) = cluster_node_range.unwrap_or((
        visit_record.node_cluster_start,
        visit_record.node_cluster_count,
    ));
    VirtualGeometryNodeAndClusterCullTraversalRecord {
        op: VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster,
        child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
        instance_index: visit_record.instance_index,
        entity: visit_record.entity,
        cluster_array_index,
        hierarchy_node_id: cluster_hierarchy_node_id.or(visit_record.hierarchy_node_id),
        node_cluster_start,
        node_cluster_count,
        child_base: 0,
        child_count: 0,
        traversal_index,
        cluster_budget: visit_record.cluster_budget,
        page_budget: visit_record.page_budget,
        forced_mip: visit_record.forced_mip,
    }
}

fn node_and_cluster_cull_hierarchy_node_range(
    instance_index: u32,
    hierarchy_node_id: u32,
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
) -> Option<(u32, u32)> {
    hierarchy_nodes
        .iter()
        .find(|node| node.instance_index == instance_index && node.node_id == hierarchy_node_id)
        .map(|node| (node.cluster_start, node.cluster_count))
}

fn node_and_cluster_cull_resident_parent_cluster_array_index(
    child_cluster_array_index: u32,
    child_cluster: &RenderVirtualGeometryCluster,
    clusters: &[RenderVirtualGeometryCluster],
    pages: &[RenderVirtualGeometryPage],
    forced_mip: Option<u8>,
) -> Option<u32> {
    if forced_mip.is_some() {
        return None;
    }

    let mut parent_cluster_id = child_cluster.parent_cluster_id;
    let mut visited_parent_count = 0usize;
    while let Some(current_parent_cluster_id) = parent_cluster_id {
        if visited_parent_count >= clusters.len() {
            return None;
        }
        visited_parent_count = visited_parent_count.saturating_add(1);

        let Some((parent_cluster_array_index, parent_cluster)) =
            clusters
                .iter()
                .enumerate()
                .find(|(cluster_array_index, cluster)| {
                    *cluster_array_index != child_cluster_array_index as usize
                        && cluster.entity == child_cluster.entity
                        && cluster.cluster_id == current_parent_cluster_id
                })
        else {
            return None;
        };

        if pages
            .iter()
            .any(|page| page.page_id == parent_cluster.page_id && page.resident)
        {
            return Some(parent_cluster_array_index as u32);
        }

        parent_cluster_id = parent_cluster.parent_cluster_id;
    }

    None
}

fn append_node_and_cluster_cull_requested_page_id(
    requested_page_ids: &mut Vec<u32>,
    page_id: u32,
    page_budget: u32,
) {
    if requested_page_ids.len() >= page_budget as usize || requested_page_ids.contains(&page_id) {
        return;
    }

    requested_page_ids.push(page_id);
}

fn splittable_hierarchy_node_for_visit_record(
    visit_record: &VirtualGeometryNodeAndClusterCullTraversalRecord,
    camera: &ViewportCameraSnapshot,
    global_state: &RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
    clusters: &[RenderVirtualGeometryCluster],
) -> Option<RenderVirtualGeometryHierarchyNode> {
    let hierarchy_node_id = visit_record.hierarchy_node_id?;
    let node = hierarchy_nodes.iter().copied().find(|node| {
        node.instance_index == visit_record.instance_index && node.node_id == hierarchy_node_id
    })?;

    let exceeds_budget = visit_record.node_cluster_count > visit_record.cluster_budget;
    let exceeds_screen_space_error = node_and_cluster_cull_child_node_exceeds_screen_space_error(
        visit_record,
        clusters,
        global_state,
    );
    let intersects_frustum = !global_state.child_frustum_culling_enabled
        || node_and_cluster_cull_child_node_intersects_frustum(visit_record, camera, clusters);

    (node.child_count > 0 && intersects_frustum && (exceeds_budget || exceeds_screen_space_error))
        .then_some(node)
}

fn node_and_cluster_cull_child_node_exceeds_screen_space_error(
    visit_record: &VirtualGeometryNodeAndClusterCullTraversalRecord,
    clusters: &[RenderVirtualGeometryCluster],
    global_state: &RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
) -> bool {
    if clusters.is_empty() {
        return false;
    }

    (0..visit_record.node_cluster_count).any(|cluster_offset| {
        let cluster_array_index = visit_record
            .node_cluster_start
            .saturating_add(cluster_offset);
        clusters
            .get(cluster_array_index as usize)
            .is_some_and(|cluster| {
                cluster.screen_space_error > global_state.child_split_screen_space_error_threshold
            })
    })
}

fn node_and_cluster_cull_child_node_intersects_frustum(
    visit_record: &VirtualGeometryNodeAndClusterCullTraversalRecord,
    camera: &ViewportCameraSnapshot,
    clusters: &[RenderVirtualGeometryCluster],
) -> bool {
    if clusters.is_empty() {
        return true;
    }

    let mut saw_referenced_cluster = false;
    for cluster_offset in 0..visit_record.node_cluster_count {
        let cluster_array_index = visit_record
            .node_cluster_start
            .saturating_add(cluster_offset);
        let Some(cluster) = clusters.get(cluster_array_index as usize) else {
            continue;
        };
        saw_referenced_cluster = true;
        if node_and_cluster_cull_cluster_intersects_frustum(cluster, camera) {
            return true;
        }
    }

    !saw_referenced_cluster
}

fn node_and_cluster_cull_cluster_intersects_frustum(
    cluster: &RenderVirtualGeometryCluster,
    camera: &ViewportCameraSnapshot,
) -> bool {
    if cluster.bounds_radius <= 0.0 {
        return true;
    }

    let view_position = view_matrix(camera.transform).transform_point3(cluster.bounds_center);
    let depth = -view_position.z;
    let near = camera.z_near.max(0.001);
    let far = camera.z_far.max(near);
    let radius = cluster.bounds_radius.max(0.0);

    if depth + radius < near || depth - radius > far {
        return false;
    }

    match camera.projection_mode {
        ProjectionMode::Perspective => {
            let clamped_depth = depth.max(near);
            let half_height = clamped_depth * (camera.fov_y_radians * 0.5).tan();
            let half_width = half_height * camera.aspect_ratio.max(0.001);
            view_position.x.abs() <= half_width + radius
                && view_position.y.abs() <= half_height + radius
        }
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.01);
            let half_width = half_height * camera.aspect_ratio.max(0.001);
            view_position.x.abs() <= half_width + radius
                && view_position.y.abs() <= half_height + radius
        }
    }
}
