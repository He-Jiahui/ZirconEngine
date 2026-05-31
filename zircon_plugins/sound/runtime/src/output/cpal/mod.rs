mod capability;
mod device;
mod error;

#[cfg(feature = "cpal-backend")]
mod callback;
#[cfg(feature = "cpal-backend")]
mod device_thread;
#[cfg(feature = "cpal-backend")]
mod producer_thread;
#[cfg(feature = "cpal-backend")]
mod selection;
#[cfg(feature = "cpal-backend")]
mod session;
#[cfg(feature = "cpal-backend")]
mod shared_state;

pub(crate) use capability::{cpal_backend_capabilities, validate_cpal_backend_supported};
pub(crate) use device::cpal_output_devices;
#[cfg(not(feature = "cpal-backend"))]
pub(crate) use error::cpal_backend_unavailable_detail;
#[cfg(feature = "cpal-backend")]
pub(crate) use session::{start_cpal_session, CpalOutputSession};

pub(crate) const CPAL_BACKEND: &str = "cpal";
#[cfg(feature = "cpal-backend")]
pub(super) const CPAL_DEFAULT_OUTPUT_DEVICE_ID: &str = "sound.output.cpal.default";
#[cfg(feature = "cpal-backend")]
pub(super) const CPAL_OUTPUT_DEVICE_ID_PREFIX: &str = "sound.output.cpal.device.";
