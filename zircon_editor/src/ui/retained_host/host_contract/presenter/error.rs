use std::error::Error;
use std::fmt;

use zircon_runtime::rhi::RhiError;

pub(in crate::ui::retained_host::host_contract) type HostPresenterResult<T> =
    Result<T, HostPresenterError>;

#[derive(Debug)]
pub(in crate::ui::retained_host::host_contract) enum HostPresenterError {
    Softbuffer(String),
    GpuUnavailable(String),
    Rhi(RhiError),
}

impl HostPresenterError {
    pub(super) fn softbuffer(error: softbuffer::SoftBufferError) -> Self {
        Self::Softbuffer(error.to_string())
    }

    pub(super) fn gpu_unavailable(reason: impl Into<String>) -> Self {
        Self::GpuUnavailable(reason.into())
    }
}

impl fmt::Display for HostPresenterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Softbuffer(error) => write!(f, "softbuffer presenter failed: {error}"),
            Self::GpuUnavailable(reason) => write!(f, "gpu chrome presenter unavailable: {reason}"),
            Self::Rhi(error) => write!(f, "gpu chrome RHI error: {error}"),
        }
    }
}

impl Error for HostPresenterError {}

impl From<RhiError> for HostPresenterError {
    fn from(value: RhiError) -> Self {
        Self::Rhi(value)
    }
}
