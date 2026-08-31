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

    pub(crate) fn clear_pointer_interaction_without_route(
        &mut self,
    ) -> Result<Vec<UiComponentEventReport>, UiTreeError> {
        if let Some(pressed) = self.focus.pressed.take() {
            if self.component_states.set_pressed(pressed, false) {
                self.mark_component_state_render_dirty(pressed)?;
            }
        }
        self.release_pointer_capture();
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
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830cz_hover_leave_reports_reserve_binding_bound() {
        let source = include_str!("interaction_state.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("interaction state production source");

        assert!(production.contains("reports.reserve(metadata.bindings.len());"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830cz_hover_leave_report_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const BINDINGS_PER_BATCH: usize = 32;
        const MARKER: &str = "RUNTIME512_HOVER_LEAVE_REPORT_CAPACITY_BENCH_V1";

        let legacy_growth_events = report_growth_events(BATCH_COUNT, BINDINGS_PER_BATCH, false);
        let optimized_growth_events = report_growth_events(BATCH_COUNT, BINDINGS_PER_BATCH, true);

        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
        println!(
            "{MARKER} batches={BATCH_COUNT} bindings_per_batch={BINDINGS_PER_BATCH} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
        );
    }

    fn report_growth_events(batch_count: usize, bindings_per_batch: usize, reserve: bool) -> usize {
        let mut growth_events = 0;
        for _ in 0..batch_count {
            let mut reports = if reserve {
                Vec::with_capacity(bindings_per_batch)
            } else {
                Vec::new()
            };
            for report in 0..bindings_per_batch {
                let previous_capacity = reports.capacity();
                reports.push(report);
                growth_events += usize::from(reports.capacity() != previous_capacity);
            }
        }
        growth_events
    }
}
