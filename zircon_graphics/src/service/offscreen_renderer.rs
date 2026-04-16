use crate::backend::OffscreenTarget;

#[derive(Default)]
pub(in crate::service) struct OffscreenRenderer {
    pub(in crate::service) target: Option<OffscreenTarget>,
}
