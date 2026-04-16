use crate::backend::OffscreenTarget;

#[derive(Default)]
pub(in crate::service) struct SharedTextureOffscreenRenderer {
    pub(in crate::service) target: Option<OffscreenTarget>,
}
