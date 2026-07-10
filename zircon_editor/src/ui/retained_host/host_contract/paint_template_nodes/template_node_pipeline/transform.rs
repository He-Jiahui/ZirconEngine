use super::super::super::data::{FrameRect, TemplatePaneNodeData};

/// Pane-scoped paint projection over an owned node clone and its effective clip.
pub(in crate::ui::retained_host::host_contract) trait TemplateNodePaintTransform {
    fn transform(
        &self,
        node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)>;
}
