use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::sound::{SoundError, SoundOutputDeviceState};

use crate::engine::SoundEngineState;
use crate::SoundConfig;

use super::super::cpal;
#[cfg(feature = "cpal-backend")]
use super::session::SoundOutputBackendSession;
use super::SoundOutputDeviceRuntimeState;

impl SoundOutputDeviceRuntimeState {
    pub(crate) fn start_with_engine(
        &mut self,
        engine_state: Arc<Mutex<SoundEngineState>>,
        config: Arc<Mutex<SoundConfig>>,
    ) -> Result<(), SoundError> {
        if let Some(error) = self.unavailable_backend_error() {
            return Err(error);
        }
        self.clear_backend_session();
        if self.descriptor.backend == cpal::CPAL_BACKEND {
            return self.start_cpal_backend(engine_state, config);
        }
        self.start_software_backend();
        Ok(())
    }

    fn start_software_backend(&mut self) {
        self.state = SoundOutputDeviceState::Started;
        self.last_error = None;
    }

    #[cfg(feature = "cpal-backend")]
    fn start_cpal_backend(
        &mut self,
        engine_state: Arc<Mutex<SoundEngineState>>,
        config: Arc<Mutex<SoundConfig>>,
    ) -> Result<(), SoundError> {
        match cpal::start_cpal_session(&self.descriptor, engine_state, config) {
            Ok(session) => {
                self.backend_session = SoundOutputBackendSession::Cpal(session);
                self.state = SoundOutputDeviceState::Started;
                self.last_error = None;
                Ok(())
            }
            Err(error) => {
                if let SoundError::BackendUnavailable { detail } = &error {
                    self.record_backend_unavailable(cpal::CPAL_BACKEND, detail.clone());
                } else {
                    self.record_error(&error);
                    self.state = SoundOutputDeviceState::Stopped;
                }
                Err(error)
            }
        }
    }

    #[cfg(not(feature = "cpal-backend"))]
    fn start_cpal_backend(
        &mut self,
        _engine_state: Arc<Mutex<SoundEngineState>>,
        _config: Arc<Mutex<SoundConfig>>,
    ) -> Result<(), SoundError> {
        let detail = cpal::cpal_backend_unavailable_detail();
        self.record_backend_unavailable(cpal::CPAL_BACKEND, detail.clone());
        Err(SoundError::BackendUnavailable { detail })
    }

    pub(crate) fn stop(&mut self) {
        self.clear_backend_session();
        self.state = SoundOutputDeviceState::Stopped;
    }

    pub(crate) fn block_size_frames(&self) -> Result<usize, SoundError> {
        if let Some(error) = self.unavailable_backend_error() {
            return Err(error);
        }
        if self.state != SoundOutputDeviceState::Started {
            return Err(SoundError::BackendUnavailable {
                detail: "sound output device is stopped".to_string(),
            });
        }
        Ok(self.descriptor.block_size_frames)
    }
}
