#[cfg(feature = "cpal-backend")]
use zircon_runtime::core::framework::sound::SoundError;

#[cfg(not(feature = "cpal-backend"))]
pub(crate) fn cpal_backend_unavailable_detail() -> String {
    "sound output backend `cpal` requires the `cpal-backend` feature".to_string()
}

#[cfg(feature = "cpal-backend")]
pub(in crate::output::cpal) fn backend_unavailable(detail: String) -> SoundError {
    SoundError::BackendUnavailable { detail }
}
