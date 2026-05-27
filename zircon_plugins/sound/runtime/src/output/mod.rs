use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::sound::{
    SoundBackendCallbackReport, SoundBackendCapability, SoundError, SoundOutputDeviceDescriptor,
    SoundOutputDeviceInfo, SoundOutputDeviceState, SoundOutputDeviceStatus,
    SoundOutputLatencyStatus,
};

use crate::engine::SoundEngineState;
use crate::SoundConfig;

mod cpal;
mod ring_buffer;
mod software;

#[cfg(feature = "cpal-backend")]
use self::cpal::CpalOutputSession;
#[cfg(feature = "cpal-backend")]
use self::ring_buffer::SoundOutputRingBuffer;

// Facade state delegates backend-specific behavior while preserving the neutral DTO status shape.
#[derive(Debug)]
pub(crate) struct SoundOutputDeviceRuntimeState {
    descriptor: SoundOutputDeviceDescriptor,
    state: SoundOutputDeviceState,
    rendered_blocks: u64,
    rendered_frames: u64,
    callback_count: u64,
    last_callback_sequence: Option<u64>,
    next_callback_sequence: u64,
    underrun_count: u64,
    last_error: Option<String>,
    unavailable_backend: Option<String>,
    unavailable_detail: Option<String>,
    backend_session: SoundOutputBackendSession,
}

#[derive(Debug, Default)]
enum SoundOutputBackendSession {
    #[default]
    None,
    #[cfg(feature = "cpal-backend")]
    Cpal(CpalOutputSession),
}

impl SoundOutputDeviceRuntimeState {
    pub(crate) fn new(config: &SoundConfig) -> Self {
        Self {
            descriptor: SoundOutputDeviceDescriptor::software(
                config.backend.clone(),
                config.sample_rate_hz,
                config.channel_count,
                config.block_size_frames,
            ),
            state: SoundOutputDeviceState::Stopped,
            rendered_blocks: 0,
            rendered_frames: 0,
            callback_count: 0,
            last_callback_sequence: None,
            next_callback_sequence: 0,
            underrun_count: 0,
            last_error: None,
            unavailable_backend: None,
            unavailable_detail: None,
            backend_session: SoundOutputBackendSession::None,
        }
    }

    pub(crate) fn configure(
        &mut self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError> {
        validate_output_device_descriptor(&descriptor)?;
        validate_backend_supported(&descriptor)?;
        self.clear_backend_session();
        self.descriptor = descriptor;
        self.state = SoundOutputDeviceState::Stopped;
        self.rendered_blocks = 0;
        self.rendered_frames = 0;
        self.callback_count = 0;
        self.last_callback_sequence = None;
        self.next_callback_sequence = 0;
        self.underrun_count = 0;
        self.last_error = None;
        self.unavailable_backend = None;
        self.unavailable_detail = None;
        Ok(())
    }

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

    pub(crate) fn record_rendered_block(&mut self, frames: usize, sample_count: usize) {
        self.rendered_blocks = self.rendered_blocks.saturating_add(1);
        self.rendered_frames = self.rendered_frames.saturating_add(frames as u64);
        let expected_samples = frames.saturating_mul(self.descriptor.channel_count as usize);
        if sample_count != expected_samples {
            self.underrun_count = self.underrun_count.saturating_add(1);
        }
        self.last_error = None;
    }

    pub(crate) fn record_error(&mut self, error: &SoundError) {
        self.underrun_count = self.underrun_count.saturating_add(1);
        self.last_error = Some(error.to_string());
    }

    pub(crate) fn record_backend_unavailable(
        &mut self,
        backend: impl Into<String>,
        detail: impl Into<String>,
    ) {
        self.clear_backend_session();
        let detail = detail.into();
        self.unavailable_backend = Some(backend.into());
        self.unavailable_detail = Some(detail.clone());
        self.state = SoundOutputDeviceState::Stopped;
        self.last_error = Some(format!("sound backend unavailable: {detail}"));
    }

    pub(crate) fn unavailable_backend_status(&self) -> Option<(&str, &str)> {
        Some((
            self.unavailable_backend.as_deref()?,
            self.unavailable_detail.as_deref()?,
        ))
    }

    pub(crate) fn unavailable_backend_error(&self) -> Option<SoundError> {
        let (_, detail) = self.unavailable_backend_status()?;
        Some(SoundError::BackendUnavailable {
            detail: detail.to_string(),
        })
    }

    pub(crate) fn record_callback_block(
        &mut self,
        requested_frames: usize,
        rendered_frames: usize,
        sample_count: usize,
    ) -> SoundBackendCallbackReport {
        let sequence_index = self.next_callback_sequence;
        self.next_callback_sequence = self.next_callback_sequence.saturating_add(1);
        self.callback_count = self.callback_count.saturating_add(1);
        self.last_callback_sequence = Some(sequence_index);
        self.record_rendered_block(rendered_frames, sample_count);
        let expected_samples =
            requested_frames.saturating_mul(self.descriptor.channel_count as usize);
        SoundBackendCallbackReport {
            device: self.descriptor.id.clone(),
            backend: self.descriptor.backend.clone(),
            sequence_index,
            requested_frames,
            rendered_frames,
            sample_count,
            underrun: rendered_frames != requested_frames || sample_count != expected_samples,
            error: None,
        }
    }

    pub(crate) fn record_callback_error(
        &mut self,
        requested_frames: usize,
        error: &SoundError,
    ) -> SoundBackendCallbackReport {
        let sequence_index = self.next_callback_sequence;
        self.next_callback_sequence = self.next_callback_sequence.saturating_add(1);
        self.callback_count = self.callback_count.saturating_add(1);
        self.last_callback_sequence = Some(sequence_index);
        self.record_error(error);
        SoundBackendCallbackReport {
            device: self.descriptor.id.clone(),
            backend: self.descriptor.backend.clone(),
            sequence_index,
            requested_frames,
            rendered_frames: 0,
            sample_count: 0,
            underrun: true,
            error: Some(error.to_string()),
        }
    }

    pub(crate) fn status(&self) -> SoundOutputDeviceStatus {
        let status = SoundOutputDeviceStatus {
            descriptor: self.descriptor.clone(),
            state: self.state,
            latency: latency_status_for_descriptor(&self.descriptor, None, None),
            rendered_blocks: self.rendered_blocks,
            rendered_frames: self.rendered_frames,
            callback_count: self.callback_count,
            last_callback_sequence: self.last_callback_sequence,
            underrun_count: self.underrun_count,
            last_error: self.last_error.clone(),
            diagnostics: Vec::new(),
        };
        self.finalize_status(self.status_with_backend_session(status))
    }

    #[cfg(feature = "cpal-backend")]
    fn status_with_backend_session(
        &self,
        mut status: SoundOutputDeviceStatus,
    ) -> SoundOutputDeviceStatus {
        #[cfg(feature = "cpal-backend")]
        if let SoundOutputBackendSession::Cpal(session) = &self.backend_session {
            let cpal_status = session.status();
            status.rendered_blocks = status
                .rendered_blocks
                .saturating_add(cpal_status.rendered_blocks);
            status.rendered_frames = status
                .rendered_frames
                .saturating_add(cpal_status.rendered_frames);
            status.callback_count = status
                .callback_count
                .saturating_add(cpal_status.callback_count);
            status.last_callback_sequence = cpal_status
                .last_callback_sequence
                .or(status.last_callback_sequence);
            status.underrun_count = status
                .underrun_count
                .saturating_add(cpal_status.underrun_count);
            status.latency.queued_samples = cpal_status.queued_samples;
            status.latency.capacity_samples = cpal_status.capacity_samples;
            status.last_error = cpal_status.last_error.or(status.last_error);
        }
        status
    }

    #[cfg(not(feature = "cpal-backend"))]
    fn status_with_backend_session(
        &self,
        status: SoundOutputDeviceStatus,
    ) -> SoundOutputDeviceStatus {
        status
    }

    fn clear_backend_session(&mut self) {
        match &mut self.backend_session {
            SoundOutputBackendSession::None => {}
            #[cfg(feature = "cpal-backend")]
            SoundOutputBackendSession::Cpal(session) => session.stop(),
        }
        self.backend_session = SoundOutputBackendSession::None;
    }

    fn finalize_status(&self, mut status: SoundOutputDeviceStatus) -> SoundOutputDeviceStatus {
        if let Some((_, detail)) = self.unavailable_backend_status() {
            push_status_diagnostic(&mut status, format!("sound backend unavailable: {detail}"));
        }
        if let Some(last_error) = status.last_error.clone() {
            push_status_diagnostic(&mut status, last_error);
        }
        status
    }
}

pub(crate) fn available_output_backends() -> Vec<SoundBackendCapability> {
    let mut backends = software::software_backend_capabilities();
    backends.extend(cpal::cpal_backend_capabilities());
    backends
}

pub(crate) fn available_output_devices(config: &SoundConfig) -> Vec<SoundOutputDeviceInfo> {
    let mut devices = software::software_output_devices(config);
    devices.extend(cpal::cpal_output_devices(config));
    devices
}

fn latency_status_for_descriptor(
    descriptor: &SoundOutputDeviceDescriptor,
    queued_samples: Option<usize>,
    capacity_samples: Option<usize>,
) -> SoundOutputLatencyStatus {
    let estimated_latency_frames = descriptor
        .block_size_frames
        .saturating_mul(descriptor.latency_blocks);
    let estimated_latency_seconds = if descriptor.sample_rate_hz == 0 {
        0.0
    } else {
        estimated_latency_frames as f64 / descriptor.sample_rate_hz as f64
    };
    SoundOutputLatencyStatus {
        requested_latency_blocks: descriptor.latency_blocks,
        estimated_latency_frames,
        estimated_latency_seconds,
        queued_samples,
        capacity_samples,
    }
}

fn push_status_diagnostic(status: &mut SoundOutputDeviceStatus, diagnostic: String) {
    if !status.diagnostics.iter().any(|entry| entry == &diagnostic) {
        status.diagnostics.push(diagnostic);
    }
}

fn validate_backend_supported(descriptor: &SoundOutputDeviceDescriptor) -> Result<(), SoundError> {
    if software::supports_software_backend(&descriptor.backend) {
        return Ok(());
    }
    if descriptor.backend == cpal::CPAL_BACKEND {
        return cpal::validate_cpal_backend_supported();
    }
    Err(SoundError::BackendUnavailable {
        detail: format!(
            "sound output backend `{}` is not available",
            descriptor.backend
        ),
    })
}

pub(crate) fn validate_output_device_descriptor(
    descriptor: &SoundOutputDeviceDescriptor,
) -> Result<(), SoundError> {
    if descriptor.id.as_str().trim().is_empty() {
        return Err(SoundError::InvalidParameter(
            "output device id must be non-empty".to_string(),
        ));
    }
    if descriptor.backend.trim().is_empty() {
        return Err(SoundError::InvalidParameter(
            "output backend must be non-empty".to_string(),
        ));
    }
    if descriptor.display_name.trim().is_empty() {
        return Err(SoundError::InvalidParameter(
            "output display name must be non-empty".to_string(),
        ));
    }
    if descriptor.sample_rate_hz == 0 {
        return Err(SoundError::InvalidParameter(
            "output sample rate must be non-zero".to_string(),
        ));
    }
    if descriptor.channel_count == 0 {
        return Err(SoundError::InvalidParameter(
            "output channel count must be non-zero".to_string(),
        ));
    }
    if descriptor.block_size_frames == 0 {
        return Err(SoundError::InvalidParameter(
            "output block size must be non-zero".to_string(),
        ));
    }
    if descriptor.latency_blocks == 0 {
        return Err(SoundError::InvalidParameter(
            "output latency blocks must be non-zero".to_string(),
        ));
    }
    Ok(())
}
