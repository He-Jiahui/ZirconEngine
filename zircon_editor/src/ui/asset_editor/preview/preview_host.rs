use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::template::UiCompiledDocument;
use zircon_runtime_interface::ui::{event_ui::UiTreeId, layout::UiSize};

use crate::ui::asset_editor::session::UiAssetEditorSessionError;
use crate::ui::template::EditorTemplateRuntimeService;

#[derive(Clone, Debug, PartialEq)]
pub struct UiAssetPreviewHost {
    preview_size: UiSize,
    surface: UiSurface,
}

impl UiAssetPreviewHost {
    pub fn new(
        preview_size: UiSize,
        asset_id: &str,
        compiled: &UiCompiledDocument,
    ) -> Result<Self, UiAssetEditorSessionError> {
        let template_service = EditorTemplateRuntimeService;
        let mut surface = template_service.build_surface_from_compiled_document(
            UiTreeId::new(format!("ui_asset.preview.{asset_id}")),
            compiled,
        )?;
        surface.compute_layout(preview_size)?;
        Ok(Self {
            preview_size,
            surface,
        })
    }

    pub fn rebuild(
        &mut self,
        asset_id: &str,
        compiled: &UiCompiledDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        *self = Self::new(self.preview_size, asset_id, compiled)?;
        Ok(())
    }

    pub fn rebuild_with_size(
        &mut self,
        preview_size: UiSize,
        _asset_id: &str,
        _compiled: &UiCompiledDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        if self.preview_size == preview_size {
            return Ok(());
        }

        self.preview_size = preview_size;
        for root_id in self.surface.tree.roots.clone() {
            if let Some(root) = self.surface.tree.nodes.get_mut(&root_id) {
                root.dirty.layout = true;
                root.dirty.hit_test = true;
                root.dirty.render = true;
            }
        }
        self.surface.rebuild_dirty(preview_size)?;
        Ok(())
    }

    pub fn surface(&self) -> &UiSurface {
        &self.surface
    }

    #[cfg(test)]
    pub(crate) fn surface_mut(&mut self) -> &mut UiSurface {
        &mut self.surface
    }

    pub fn preview_size(&self) -> UiSize {
        self.preview_size
    }
}
