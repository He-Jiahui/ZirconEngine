use crate::template::UiCompiledDocument;
use crate::template::UiTemplateInstance;
use crate::event_ui::UiTreeId;
use crate::UiSurface;

use super::build_error::UiTemplateBuildError;
use super::tree_builder::UiTemplateTreeBuilder;

#[derive(Default)]
pub struct UiTemplateSurfaceBuilder;

impl UiTemplateSurfaceBuilder {
    pub fn build_surface(
        tree_id: UiTreeId,
        instance: &UiTemplateInstance,
    ) -> Result<UiSurface, UiTemplateBuildError> {
        let tree = UiTemplateTreeBuilder::build_tree(tree_id.clone(), instance)?;
        let mut surface = UiSurface::new(tree_id);
        surface.tree = tree;
        surface.rebuild();
        Ok(surface)
    }

    pub fn build_surface_from_compiled_document(
        tree_id: UiTreeId,
        document: &UiCompiledDocument,
    ) -> Result<UiSurface, UiTemplateBuildError> {
        Self::build_surface(tree_id, document.template_instance())
    }
}
