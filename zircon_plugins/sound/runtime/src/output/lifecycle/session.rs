#[cfg(feature = "cpal-backend")]
use super::super::cpal::CpalOutputSession;

#[derive(Debug, Default)]
pub(super) enum SoundOutputBackendSession {
    #[default]
    None,
    #[cfg(feature = "cpal-backend")]
    Cpal(CpalOutputSession),
}
