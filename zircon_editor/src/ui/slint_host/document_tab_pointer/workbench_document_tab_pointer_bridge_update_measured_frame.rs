use zircon_runtime::ui::layout::UiFrame;

use super::constants::{STRIP_Y, TAB_HEIGHT};
use super::helper::tab_min_width;
use super::workbench_document_tab_pointer_bridge::WorkbenchDocumentTabPointerBridge;

impl WorkbenchDocumentTabPointerBridge {
    pub(in crate::ui::slint_host::document_tab_pointer) fn update_measured_frame(
        &mut self,
        surface_key: &str,
        item_index: usize,
        tab_x: f32,
        tab_width: f32,
    ) -> Result<(), String> {
        let surface = self
            .layout
            .surfaces
            .iter()
            .find(|surface| surface.key == surface_key)
            .ok_or_else(|| format!("Unknown document tab surface {surface_key}"))?;
        let Some(frames) = self.measured_frames.get_mut(surface_key) else {
            return Err(format!("Missing measured frame store for {surface_key}"));
        };
        if item_index >= frames.len() {
            return Err(format!(
                "Document tab index {item_index} is outside surface {surface_key}"
            ));
        }
        frames[item_index] = Some(UiFrame::new(
            surface.strip_frame.x + tab_x,
            surface.strip_frame.y + STRIP_Y,
            tab_width.max(tab_min_width(surface, item_index)),
            TAB_HEIGHT,
        ));
        self.rebuild_surface();
        Ok(())
    }
}
