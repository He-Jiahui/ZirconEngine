use std::fmt;

use super::{PlatformHostBackendRequestError, PlatformHostDescriptor, PlatformHostQuiesceRequest};

/// Arc-safe control endpoint for a platform host owned by the process host.
///
/// The implementation must enqueue work for its declared host thread. It must
/// not retain a native event loop or window object in the runtime service.
pub trait PlatformHostBackend: fmt::Debug + Send + Sync + 'static {
    fn descriptor(&self) -> PlatformHostDescriptor;

    fn request_quiesce(
        &self,
        request: PlatformHostQuiesceRequest,
    ) -> Result<(), PlatformHostBackendRequestError>;
}
