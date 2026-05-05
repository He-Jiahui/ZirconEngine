use crate::ui::tree::{UiHitTestIndex, UiHitTestResult};
use zircon_runtime_interface::ui::{
    layout::UiPoint,
    surface::{
        UiHitTestDebugDump, UiHitTestQuery, UiHitTestReject, UiHitTestRejectReason,
        UiSurfaceFrame,
    },
    tree::UiInputPolicy,
};

use super::{arranged_effective_input_policy, is_arranged_child_hit_path_visible};

pub fn hit_test_surface_frame(surface_frame: &UiSurfaceFrame, point: UiPoint) -> UiHitTestResult {
    hit_test_surface_frame_with_query(surface_frame, UiHitTestQuery::new(point))
}

pub fn hit_test_surface_frame_with_query(
    surface_frame: &UiSurfaceFrame,
    query: UiHitTestQuery,
) -> UiHitTestResult {
    UiHitTestIndex::hit_test_grid_arranged_with_query(
        &surface_frame.hit_grid,
        &surface_frame.arranged_tree,
        query,
    )
}

pub fn debug_hit_test_surface_frame(
    surface_frame: &UiSurfaceFrame,
    point: UiPoint,
) -> UiHitTestDebugDump {
    debug_hit_test_surface_frame_with_query(surface_frame, UiHitTestQuery::new(point))
}

pub fn debug_hit_test_surface_frame_with_query(
    surface_frame: &UiSurfaceFrame,
    query: UiHitTestQuery,
) -> UiHitTestDebugDump {
    let point = query.hit_point();
    let hit = hit_test_surface_frame_with_query(surface_frame, query);
    let mut rejected = Vec::new();
    for node in &surface_frame.arranged_tree.nodes {
        if !node.frame.contains_point(point) {
            rejected.push(UiHitTestReject {
                node_id: node.node_id,
                control_id: node.control_id.clone(),
                reason: UiHitTestRejectReason::OutsideFrame,
                message: "point is outside the arranged frame".to_string(),
            });
        } else if !node.clip_frame.contains_point(point) {
            rejected.push(UiHitTestReject {
                node_id: node.node_id,
                control_id: node.control_id.clone(),
                reason: UiHitTestRejectReason::OutsideClip,
                message: "point is outside the effective clip frame".to_string(),
            });
        } else if !is_arranged_child_hit_path_visible(&surface_frame.arranged_tree, node.node_id)
            .unwrap_or(false)
        {
            rejected.push(UiHitTestReject {
                node_id: node.node_id,
                control_id: node.control_id.clone(),
                reason: UiHitTestRejectReason::VisibilityFiltered,
                message: "node or ancestor visibility excludes hit testing".to_string(),
            });
        } else if !node.enabled {
            rejected.push(UiHitTestReject {
                node_id: node.node_id,
                control_id: node.control_id.clone(),
                reason: UiHitTestRejectReason::Disabled,
                message: "node is disabled".to_string(),
            });
        } else if arranged_effective_input_policy(&surface_frame.arranged_tree, node.node_id)
            .is_ok_and(|policy| policy == UiInputPolicy::Ignore)
        {
            rejected.push(UiHitTestReject {
                node_id: node.node_id,
                control_id: node.control_id.clone(),
                reason: UiHitTestRejectReason::InputPolicyIgnore,
                message: "effective input policy ignores pointer input".to_string(),
            });
        } else if !node.supports_pointer() {
            rejected.push(UiHitTestReject {
                node_id: node.node_id,
                control_id: node.control_id.clone(),
                reason: UiHitTestRejectReason::NotPointerTarget,
                message: "node does not declare pointer interaction support".to_string(),
            });
        }
    }
    UiHitTestDebugDump {
        tree_id: surface_frame.tree_id.clone(),
        point,
        hit_stack: hit.stacked,
        hit_path: hit.path,
        inspected: surface_frame.arranged_tree.nodes.len(),
        rejected,
    }
}
