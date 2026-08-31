use super::super::super::data::{FrameRect, TemplatePaneNodeData};

/// Pane-scoped paint projection over an owned node clone and its effective clip.
pub(in crate::ui::retained_host::host_contract) trait TemplateNodePaintTransform {
    /// Returns true after emitting the complete row selection. A false result must emit no rows.
    fn stream_row_visit_indices(
        &self,
        _row_count: usize,
        _clip: &FrameRect,
        _visit: &mut dyn FnMut(usize),
    ) -> bool {
        false
    }

    fn row_visit_indices(&self, _row_count: usize, _clip: &FrameRect) -> Option<Vec<usize>> {
        None
    }

    fn transform_row(
        &self,
        row: usize,
        node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)>;
}
