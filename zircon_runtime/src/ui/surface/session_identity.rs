use std::sync::Arc;

#[derive(Debug, Default)]
pub(super) struct UiSurfaceSessionIdentity(Arc<()>);

impl UiSurfaceSessionIdentity {
    pub(super) fn handle(&self) -> UiSurfaceSessionIdentityHandle {
        UiSurfaceSessionIdentityHandle(Arc::clone(&self.0))
    }
}

impl Clone for UiSurfaceSessionIdentity {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for UiSurfaceSessionIdentity {
    fn eq(&self, _other: &Self) -> bool {
        // Runtime ownership identity is intentionally absent from semantic surface equality.
        true
    }
}

#[derive(Clone, Debug)]
pub(crate) struct UiSurfaceSessionIdentityHandle(Arc<()>);

impl PartialEq for UiSurfaceSessionIdentityHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for UiSurfaceSessionIdentityHandle {}
