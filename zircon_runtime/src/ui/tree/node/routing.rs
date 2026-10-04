use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

const COMMON_BUBBLE_ROUTE_DEPTH: usize = 16;

/// 为键盘、捕获和分派回退构造从目标到根的冒泡顺序；命中帧已有路由时应复用帧路由。
/// 此查询读取可变数据树，调用方须保持父引用完整、无环并与所处理事件的树身份一致。
pub trait UiRuntimeTreeRoutingExt {
    fn bubble_route(&self, node_id: UiNodeId) -> Result<Vec<UiNodeId>, UiTreeError>;
}

impl UiRuntimeTreeRoutingExt for UiTree {
    fn bubble_route(&self, node_id: UiNodeId) -> Result<Vec<UiNodeId>, UiTreeError> {
        let mut route = Vec::with_capacity(route_initial_capacity(self.nodes.len()));
        let mut current = Some(node_id);
        while let Some(id) = current {
            let node = self.nodes.get(&id).ok_or(UiTreeError::MissingNode(id))?;
            route.push(id);
            current = node.parent;
        }
        Ok(route)
    }
}

fn route_initial_capacity(node_count: usize) -> usize {
    node_count.min(COMMON_BUBBLE_ROUTE_DEPTH)
}

#[cfg(test)]
#[path = "tests/routing.rs"]
mod tests;
