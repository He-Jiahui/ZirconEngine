/// Releases an IBL bake graph reservation when its producer does not publish a cache payload.
///
/// The reservation is created only for a runtime-cache miss and moves from frame compilation
/// into the GPU readback queue. A submitted writeback owns it until the readback either writes
/// the cache artifact or fails, so every early-return path makes the request eligible to retry.
pub(crate) struct EnvironmentIblBakeReservation {
    release: Option<Box<dyn FnOnce() + Send + 'static>>,
}

impl EnvironmentIblBakeReservation {
    pub(crate) fn new(release: impl FnOnce() + Send + 'static) -> Self {
        Self {
            release: Some(Box::new(release)),
        }
    }
}

impl Drop for EnvironmentIblBakeReservation {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            release();
        }
    }
}
