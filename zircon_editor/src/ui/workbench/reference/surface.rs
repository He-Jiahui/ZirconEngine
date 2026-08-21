use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::tree::UiTreeError;

use super::{
    builder::ReferenceSurfaceBuilder, EditorWorkbenchReferenceIds, EditorWorkbenchReferenceMetrics,
    EditorWorkbenchReferencePalette,
};

#[derive(Clone, Debug)]
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

pub fn build_editor_workbench_reference_surface(
) -> Result<EditorWorkbenchReferenceSurface, UiTreeError> {
    ReferenceSurfaceBuilder::new(
        EditorWorkbenchReferenceMetrics::default(),
        EditorWorkbenchReferencePalette::default(),
        EditorWorkbenchReferenceIds::default(),
    )
    .build()
}
