use crate::ui::retained_host::primitives::Image;

use super::super::super::super::data::HostViewportImageData;
use super::super::PaneSurfaceHostContext;

impl PaneSurfaceHostContext<'_> {
    pub(crate) fn set_viewport_image(&self, value: Image) -> bool {
        let Some(image) = HostViewportImageData::from_image(&value) else {
            return false;
        };
        self.state.borrow_mut().viewport_image = Some(image);
        true
    }
}
