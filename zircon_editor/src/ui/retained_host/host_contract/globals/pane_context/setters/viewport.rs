use crate::scene::viewport::{CapturedFrame, RenderViewportHandle, RenderViewportProduct};

use super::super::super::super::data::HostViewportImageData;
use super::super::PaneSurfaceHostContext;

impl PaneSurfaceHostContext<'_> {
    pub(crate) fn set_viewport_capture(
        &self,
        viewport: RenderViewportHandle,
        frame: CapturedFrame,
    ) -> bool {
        let Some(image) = HostViewportImageData::from_captured_frame(viewport, frame) else {
            return false;
        };
        self.state.borrow_mut().replace_viewport_image(image);
        true
    }

    pub(crate) fn set_viewport_product(&self, product: RenderViewportProduct) -> bool {
        let Some(image) = HostViewportImageData::from_viewport_product(product) else {
            return false;
        };
        self.state.borrow_mut().replace_viewport_image(image)
    }
}
