use zircon_ui::{UiCompiledDocument, UiSize, UiSurface, UiTemplateSurfaceBuilder, UiTreeId};

use super::session::UiAssetEditorSessionError;

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
        let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
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

    pub fn surface(&self) -> &UiSurface {
        &self.surface
    }

    pub fn preview_size(&self) -> UiSize {
        self.preview_size
    }
}
