use super::super::super::data::{FrameRect, TemplatePaneNodeData};

/// Pane-scoped paint projection over an owned node clone and its effective clip.
pub(in crate::ui::retained_host::host_contract) trait TemplateNodePaintTransform {
    fn row_visit_indices(&self, _row_count: usize, _clip: &FrameRect) -> Option<Vec<usize>> {
        None
    }

    fn transform(
        &self,
        node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)>;
}
