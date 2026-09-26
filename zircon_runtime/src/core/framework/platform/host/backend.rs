use std::fmt;

use super::{PlatformHostBackendRequestError, PlatformHostDescriptor, PlatformHostQuiesceRequest};

/// Arc-safe control endpoint for a platform host owned by the process host.
///
/// The implementation must enqueue work for its declared host thread. It must
/// not retain a native event loop or window object in the runtime service.
pub trait PlatformHostBackend: fmt::Debug + Send + Sync + 'static {
    fn descriptor(&self) -> PlatformHostDescriptor;

    /// 只确认请求已进入宿主队列；真正完成后须由宿主用原请求发布终态回执。
    fn request_quiesce(
        &self,
        request: PlatformHostQuiesceRequest,
    ) -> Result<(), PlatformHostBackendRequestError>;
}
