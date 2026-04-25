use zircon_runtime::ui::layout::UiPoint;

use super::host_drawer_header_pointer_bridge::HostDrawerHeaderPointerBridge;

impl HostDrawerHeaderPointerBridge {
    pub(super) fn global_point(
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
            .ok_or_else(|| format!("Unknown drawer header surface {surface_key}"))?;
        Ok(UiPoint::new(
            strip_frame.x + point.x,
            strip_frame.y + point.y,
        ))
    }
}
