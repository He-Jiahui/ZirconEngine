use crate::core::framework::render::ViewportIconId;

pub(crate) trait ViewportIconSource: Send + Sync + 'static {
    fn bytes(&self, id: ViewportIconId) -> Option<&'static [u8]>;
}
