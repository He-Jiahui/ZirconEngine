use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

/// 数据树的顺序与可见性查询；实际帧绘制使用已发布的 arranged 快照，查询结果不能代替帧发布。
pub trait UiRuntimeTreeRenderOrderExt {
    /// 返回所有节点的确定顺序，不过滤可见性；相同层级与绘制序号由节点身份打破平局。
    fn draw_order(&self) -> Vec<UiNodeId>;
    /// 检查节点和祖先的渲染可见性；父引用必须完整且无环，缺失节点返回错误。
    fn is_visible_in_tree(&self, node_id: UiNodeId) -> Result<bool, UiTreeError>;
}

impl UiRuntimeTreeRenderOrderExt for UiTree {
    fn draw_order(&self) -> Vec<UiNodeId> {
        let mut order: Vec<_> = self
            .nodes
            .values()
            .map(|node| (node.z_index, node.paint_order, node.node_id))
            .collect();
        order.sort_unstable_by_key(|entry| (entry.0, entry.1, entry.2));
        order.into_iter().map(|(_, _, node_id)| node_id).collect()
    }

    fn is_visible_in_tree(&self, node_id: UiNodeId) -> Result<bool, UiTreeError> {
        let mut current = Some(node_id);
        while let Some(id) = current {
            let node = self.nodes.get(&id).ok_or(UiTreeError::MissingNode(id))?;
            if !node.is_render_visible() {
                return Ok(false);
            }
            current = node.parent;
        }
        Ok(true)
    }
}

#[cfg(test)]
#[path = "tests/render_order.rs"]
mod tests;
