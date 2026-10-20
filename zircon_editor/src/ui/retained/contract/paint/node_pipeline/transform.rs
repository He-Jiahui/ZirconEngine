use super::super::super::data::{FrameRect, TemplatePaneNodeData};

/// Borrow filters and clone only the rows whose semantics actually change.
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

    fn filter_row(
        &self,
        _row: usize,
        _node: &TemplatePaneNodeData,
        clip: &FrameRect,
    ) -> Option<FrameRect> {
        Some(clip.clone())
    }

    fn patch_row(
        &self,
        _row: usize,
        _node: &TemplatePaneNodeData,
        _clip: &FrameRect,
    ) -> Option<TemplatePaneNodeData> {
        None
    }

    #[cfg(test)]
    fn transform_row(
        &self,
        row: usize,
        node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        let clip = self.filter_row(row, &node, &clip)?;
        let node = self.patch_row(row, &node, &clip).unwrap_or(node);
        Some((node, clip))
    }
}
