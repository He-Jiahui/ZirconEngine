use crate::scene::viewport::{CapturedFrame, RenderViewportHandle};

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
        self.state.borrow_mut().viewport_image = Some(image);
        true
    }
}
