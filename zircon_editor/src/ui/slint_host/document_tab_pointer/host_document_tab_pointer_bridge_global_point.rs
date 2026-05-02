use zircon_runtime_interface::ui::layout::UiPoint;

use super::host_document_tab_pointer_bridge::HostDocumentTabPointerBridge;

impl HostDocumentTabPointerBridge {
    pub(in crate::ui::slint_host::document_tab_pointer) fn global_point(
        &self,
        surface_key: &str,
        point: UiPoint,
    ) -> Result<UiPoint, String> {
        let strip_frame = self
            .layout
            .surfaces
            .iter()
            .find(|surface| surface.key == surface_key)
            .map(|surface| surface.strip_frame)
            .ok_or_else(|| format!("Unknown document tab surface {surface_key}"))?;
        Ok(UiPoint::new(
            strip_frame.x + point.x,
            strip_frame.y + point.y,
        ))
    }
}
