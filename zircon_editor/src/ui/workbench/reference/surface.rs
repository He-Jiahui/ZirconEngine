use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::tree::UiTreeError;

use super::{
    builder::ReferenceSurfaceBuilder, EditorWorkbenchReferenceIds, EditorWorkbenchReferenceMetrics,
    EditorWorkbenchReferencePalette,
};

#[derive(Clone, Debug)]
/// 可重复的参考树、节点身份和布局度量，用于几何、交互及模板对照。
pub struct EditorWorkbenchReferenceSurface {
    pub surface: UiSurface,
    pub ids: EditorWorkbenchReferenceIds,
    pub metrics: EditorWorkbenchReferenceMetrics,
    pub palette: EditorWorkbenchReferencePalette,
}

impl EditorWorkbenchReferenceSurface {
    pub fn compute_reference_layout(&mut self) -> Result<(), UiTreeError> {
        self.surface.compute_layout(self.metrics.target_size())
    }
}

/// 构造固定视觉样本；需要几何时调用其布局计算入口。
pub fn build_editor_workbench_reference_surface(
) -> Result<EditorWorkbenchReferenceSurface, UiTreeError> {
    ReferenceSurfaceBuilder::new(
        EditorWorkbenchReferenceMetrics::default(),
        EditorWorkbenchReferencePalette::default(),
        EditorWorkbenchReferenceIds::default(),
    )
    .build()
}
