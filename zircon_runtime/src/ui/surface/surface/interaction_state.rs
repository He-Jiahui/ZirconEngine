use std::collections::BTreeSet;

use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::UiComponentEvent,
    dispatch::UiComponentEventReport,
    event_ui::UiNodeId,
    tree::{UiTemplateNodeMetadata, UiTreeError, UiTreeNode},
};

use crate::ui::surface::ui_surface_effective_disabled;

use super::UiSurface;

impl UiSurface {
    /// 汇集节点、祖先、组件和模板禁用状态供行为层门禁使用；可见性、只读性、命中与路由权限仍由各入口检查。
    pub(super) fn node_interaction_enabled(&self, node_id: UiNodeId) -> Result<bool, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        Ok(
            self.node_interaction_enabled_from_parts(
                node_id,
                node,
                node.template_metadata.as_ref(),
            ),
        )
    }

    pub(super) fn widget_interaction_enabled(
        &self,
        node_id: UiNodeId,
        node: &UiTreeNode,
        metadata: &UiTemplateNodeMetadata,
    ) -> bool {
        self.node_interaction_enabled_from_parts(node_id, node, Some(metadata))
    }

    fn node_interaction_enabled_from_parts(
        &self,
        node_id: UiNodeId,
        node: &UiTreeNode,
        metadata: Option<&UiTemplateNodeMetadata>,
    ) -> bool {
        !ui_surface_effective_disabled(self, node_id, node, metadata)
    }

    /// 窗口离开/路由清空时消费旧悬停路径，先清标志并标脏，再返回已有 Hover 绑定的离开报告。
    /// 调用者负责发布报告；这一步不重新命中，也不会自动重建表面。
    pub(crate) fn clear_hovered_input_path(
        &mut self,
    ) -> Result<Vec<UiComponentEventReport>, UiTreeError> {
        let hovered = std::mem::take(&mut self.focus.hovered);
        let mut reports = Vec::new();
        let mut changed_node_ids = BTreeSet::new();
        for node_id in &hovered {
            if self.component_states.set_hovered(*node_id, false) {
                changed_node_ids.insert(*node_id);
            }
        }
        self.mark_component_states_render_dirty(&changed_node_ids)?;
        for node_id in hovered {
            self.push_hover_leave_reports(node_id, &mut reports)?;
        }
        Ok(reports)
    }

    /// 无有效路由时统一收束按压、捕获、最后光标和悬停，供窗口退出或丢失输入所有权的清理路径调用。
    pub(crate) fn clear_pointer_interaction_without_route(
        &mut self,
    ) -> Result<Vec<UiComponentEventReport>, UiTreeError> {
        let presses = std::mem::take(&mut self.input.pointer_presses);
        for press in presses.values() {
            if self.tree.node(press.owner).is_some() {
                self.set_node_pressed_dirty(press.owner, false)?;
            }
        }
        if let Some(pressed) = self.focus.pressed.take() {
            if self.tree.node(pressed).is_some() {
                self.set_node_pressed_dirty(pressed, false)?;
            }
        }
        self.release_pointer_capture();
        self.input.clear_pointer_capture();
        self.input.pointer_drags.clear();
        self.input.high_precision_owner = None;
        self.input.clear_last_cursor_point();
        self.clear_hovered_input_path()
    }

    fn push_hover_leave_reports(
        &self,
        node_id: UiNodeId,
        reports: &mut Vec<UiComponentEventReport>,
    ) -> Result<(), UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(());
        };
        reports.reserve(metadata.bindings.len());
        for _ in metadata
            .bindings
            .iter()
            .filter(|binding| binding.event == UiEventKind::Hover)
        {
            reports.push(UiComponentEventReport {
                target: node_id,
                event: UiComponentEvent::Hover { hovered: false },
                delivered: true,
                drag: None,
                template_action: None,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/interaction_state_optimization_tests.rs"]
mod optimization_tests;
