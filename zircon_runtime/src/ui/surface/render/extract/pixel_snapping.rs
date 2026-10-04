use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiPixelSnappingPolicy, surface::UiRenderCommand, tree::UiTree,
};

/// 在本轮命令完成后补齐祖先继承的像素对齐策略，实际几何对齐留给渲染后端。
/// 只解析已输出命令的祖先路径并共享结果，避免少量可见命令触发全树扫描；返回访问量用于诊断。
pub(super) fn apply_resolved_pixel_snapping_policies(
    tree: &UiTree,
    commands: &mut [UiRenderCommand],
) -> usize {
    let mut resolved = BTreeMap::new();
    let mut unresolved_path = Vec::new();
    let mut visited_node_count = 0;

    for command in commands.iter_mut() {
        command.style.pixel_snapping = resolve_command_policy(
            tree,
            command.node_id,
            &mut resolved,
            &mut unresolved_path,
            &mut visited_node_count,
        );
    }

    crate::profile_counter!(
        "runtime",
        "ui.render_extract.pixel_snapping_node_visit_count",
        visited_node_count
    );
    visited_node_count
}

fn resolve_command_policy(
    tree: &UiTree,
    node_id: UiNodeId,
    resolved: &mut BTreeMap<UiNodeId, UiPixelSnappingPolicy>,
    unresolved_path: &mut Vec<(UiNodeId, UiPixelSnappingPolicy)>,
    visited_node_count: &mut usize,
) -> UiPixelSnappingPolicy {
    if let Some(policy) = resolved.get(&node_id).copied() {
        return policy;
    }

    unresolved_path.clear();
    let mut current = Some(node_id);
    let mut parent_policy = UiPixelSnappingPolicy::Inherit;
    let mut hop_count = 0;
    while let Some(current_id) = current {
        if let Some(policy) = resolved.get(&current_id).copied() {
            parent_policy = policy;
            break;
        }
        if hop_count > tree.nodes.len() {
            unresolved_path.clear();
            return UiPixelSnappingPolicy::Inherit;
        }
        hop_count += 1;

        let Some(node) = tree.node(current_id) else {
            break;
        };
        *visited_node_count += 1;
        let authored = node
            .template_metadata
            .as_ref()
            .map(|metadata| metadata.pixel_snapping)
            .unwrap_or_default();
        unresolved_path.push((current_id, authored));
        current = node.parent;
    }

    while let Some((current_id, authored)) = unresolved_path.pop() {
        parent_policy = authored.inherit_from(parent_policy);
        resolved.insert(current_id, parent_policy);
    }
    resolved.get(&node_id).copied().unwrap_or_default()
}

#[cfg(test)]
#[path = "tests/pixel_snapping.rs"]
mod tests;
