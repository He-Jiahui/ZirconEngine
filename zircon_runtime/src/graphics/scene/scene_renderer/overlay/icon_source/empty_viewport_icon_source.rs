use crate::core::framework::render::ViewportIconId;

use super::ViewportIconSource;

#[derive(Debug, Default)]
pub(crate) struct EmptyViewportIconSource;

impl ViewportIconSource for EmptyViewportIconSource {
    fn bytes(&self, _id: ViewportIconId) -> Option<&'static [u8]> {
        None
    }
}
