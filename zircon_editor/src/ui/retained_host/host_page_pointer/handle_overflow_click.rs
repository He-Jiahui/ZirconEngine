use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use super::host_page_pointer_bridge::HostPagePointerBridge;
use super::host_page_pointer_dispatch::HostPagePointerDispatch;
use super::HostPagePointerError;

impl HostPagePointerBridge {
    pub(crate) fn handle_overflow_click(
        &mut self,
        point: UiPoint,
    ) -> Result<HostPagePointerDispatch, HostPagePointerError> {
        let Some(overflow) = self.layout.overflow.as_ref() else {
            return Ok(HostPagePointerDispatch { route: None });
        };
        let point = UiPoint::new(overflow.frame.x + point.x, overflow.frame.y + point.y);
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        Ok(HostPagePointerDispatch { route })
    }
}
