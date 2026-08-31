use std::sync::{Mutex, MutexGuard, TryLockError};
use std::time::{Duration, Instant};

use crate::graphics::types::GraphicsError;
use thiserror::Error;
use zr_rhi::{
    AdapterSelectionPolicy, DeviceGeneration, DiagnosticFrameKey, DiagnosticReadbackAdmission,
    DiagnosticReadbackBudget, DiagnosticReadbackReceipt, DiagnosticReadbackRequestId,
    DiagnosticReadbackTerminal, GpuMemoryBudget, RenderDevice, RenderDeviceProfile,
    RenderDeviceQueueTopology, RenderDeviceRequestPolicy, SubmissionLimits, SubmissionStatus,
    SubmissionTicket, TextureCopyRegion, TextureHandle,
};
use zr_rhi_wgpu::{
    WgpuMvpOffscreenTriangle, WgpuRenderDevice, WgpuRenderDeviceContext, next_wgpu_device_id,
    wgpu_device_limits,
};

use super::config::RenderBackendConfig;
use super::request_device::request_device_with_policy;
use super::select_offscreen_adapter;

/// Product-facing minimum renderer whose complete native generation belongs to neutral RHI.
///
/// This is intentionally separate from the legacy scene `RenderBackend`: its constructor does
/// not create that raw owner, and every submitted frame traverses `WgpuRenderDevice`.
pub struct NeutralMvpRenderer {
    device: WgpuRenderDevice,
    frame: WgpuMvpOffscreenTriangle,
    width: u32,
    height: u32,
    diagnostic_capture_state: Mutex<DiagnosticCaptureState>,
}

/// Errors exposed by the bounded neutral-MVP diagnostic capture path.
#[derive(Debug, Error)]
pub enum NeutralMvpCaptureError {
    #[error(transparent)]
    Rhi(#[from] zr_rhi::RhiError),
    #[error("neutral MVP render submission stopped as {status:?}")]
    RenderSubmissionTerminal { status: SubmissionStatus },
    #[error("neutral MVP diagnostic readback request was rejected: {receipt:?}")]
    ReadbackRejected { receipt: DiagnosticReadbackReceipt },
    #[error("neutral MVP diagnostic readback frame had no active request")]
    ReadbackFrameNotSubmitted,
    #[error(
        "neutral MVP received diagnostic delivery {actual_request:?}/{actual_frame:?}; expected {expected_request:?}/{expected_frame:?}"
    )]
    UnexpectedReadbackDelivery {
        expected_request: DiagnosticReadbackRequestId,
        expected_frame: DiagnosticFrameKey,
        actual_request: DiagnosticReadbackRequestId,
        actual_frame: Option<DiagnosticFrameKey>,
    },
    #[error("neutral MVP diagnostic readback terminated as {terminal:?}")]
    ReadbackTerminal {
        terminal: DiagnosticReadbackTerminal,
    },
    #[error("neutral MVP diagnostic readback succeeded without pixel bytes")]
    MissingPixelBytes,
    #[error("neutral MVP dimensions {width}x{height} overflow an RGBA8 byte count")]
    PixelByteLengthOverflow { width: u32, height: u32 },
    #[error("neutral MVP diagnostic readback returned {actual} bytes; expected {expected}")]
    UnexpectedPixelByteLength { expected: usize, actual: usize },
    #[error("neutral MVP diagnostic capture timed out after {timeout:?}")]
    TimedOut { timeout: Duration },
    #[error("neutral MVP diagnostic capture state was poisoned")]
    CaptureGatePoisoned,
}

#[derive(Clone, Copy, Debug, Default)]
enum DiagnosticCaptureState {
    #[default]
    Idle,
    Awaiting {
        request: DiagnosticReadbackRequestId,
        frame: DiagnosticFrameKey,
    },
}

impl NeutralMvpRenderer {
    pub fn new_offscreen(width: u32, height: u32) -> Result<Self, GraphicsError> {
        Self::new_offscreen_with_policy(width, height, &RenderDeviceRequestPolicy::mvp_baseline())
    }

    pub(crate) fn new_offscreen_with_policy(
        width: u32,
        height: u32,
        device_request_policy: &RenderDeviceRequestPolicy,
    ) -> Result<Self, GraphicsError> {
        let config = RenderBackendConfig::from_environment();
        let instance = wgpu::Instance::new(config.instance_descriptor());
        let (adapter, adapter_facts) = select_offscreen_adapter(
            &instance,
            config.backends,
            &AdapterSelectionPolicy::default(),
        )?;
        let requested_device = request_device_with_policy(&adapter, device_request_policy)?;
        let profile = RenderDeviceProfile::new(
            next_wgpu_device_id(),
            DeviceGeneration::initial(),
            adapter_facts,
            requested_device
                .profile_request
                .feature_negotiation()
                .clone(),
            wgpu_device_limits(&requested_device.device.limits()),
            RenderDeviceQueueTopology::single_serialized_queue(),
            GpuMemoryBudget::reference_1080p_mid(),
            SubmissionLimits::default(),
            DiagnosticReadbackBudget::default(),
        );
        let context = WgpuRenderDeviceContext::new(
            instance,
            adapter,
            requested_device.device,
            requested_device.queue,
        );
        let device = WgpuRenderDevice::new(context, profile)?;
        let frame = WgpuMvpOffscreenTriangle::new(&device, width, height)?;

        Ok(Self {
            device,
            frame,
            width,
            height,
            diagnostic_capture_state: Mutex::new(DiagnosticCaptureState::Idle),
        })
    }

    /// Records one clear-and-triangle graphics frame through the neutral submission service.
    pub fn render_frame(&self) -> Result<SubmissionTicket, GraphicsError> {
        Ok(self.frame.submit(&self.device)?)
    }

    /// Returns the neutral offscreen output owned by this renderer generation.
    pub const fn target(&self) -> TextureHandle {
        self.frame.target()
    }

    /// Renders one frame and returns its tightly packed RGBA8 pixels through bounded diagnostics.
    pub fn capture_rgba8(
        &self,
        frame_index: u64,
        timeout: Duration,
    ) -> Result<Vec<u8>, NeutralMvpCaptureError> {
        let started = Instant::now();
        let mut capture_state = self.lock_capture_state(started, timeout)?;
        self.reap_previous_capture(&mut capture_state, started, timeout)?;
        remaining_timeout(started, timeout)?;
        let ticket = self.frame.submit(&self.device)?;
        let render_status = self
            .device
            .wait_for_submission(ticket, remaining_timeout(started, timeout)?)?;
        remaining_timeout(started, timeout)?;
        if render_status != SubmissionStatus::Completed {
            return Err(NeutralMvpCaptureError::RenderSubmissionTerminal {
                status: render_status,
            });
        }

        self.device.begin_diagnostic_readback_frame(frame_index)?;
        let admission = match self.device.enqueue_diagnostic_texture_readback(
            self.frame.target(),
            TextureCopyRegion::new(self.width, self.height),
        ) {
            Ok(admission) => admission,
            Err(error) => {
                self.device
                    .abort_diagnostic_readback_frame(DiagnosticReadbackTerminal::Cancelled);
                return Err(error.into());
            }
        };
        let request = match admission {
            DiagnosticReadbackAdmission::Admitted(request) => request,
            DiagnosticReadbackAdmission::Rejected(receipt) => {
                self.device
                    .abort_diagnostic_readback_frame(DiagnosticReadbackTerminal::OverBudget);
                return Err(NeutralMvpCaptureError::ReadbackRejected { receipt });
            }
        };
        if let Err(error) = remaining_timeout(started, timeout) {
            self.device
                .abort_diagnostic_readback_frame(DiagnosticReadbackTerminal::Cancelled);
            return Err(error);
        }
        let frame = match self
            .device
            .submit_diagnostic_readback_frame("zircon-neutral-mvp-readback")
        {
            Ok(Some(frame)) => frame,
            Ok(None) => {
                self.device
                    .abort_diagnostic_readback_frame(DiagnosticReadbackTerminal::Cancelled);
                return Err(NeutralMvpCaptureError::ReadbackFrameNotSubmitted);
            }
            Err(error) => {
                self.device
                    .abort_diagnostic_readback_frame(DiagnosticReadbackTerminal::Cancelled);
                return Err(error.into());
            }
        };
        *capture_state = DiagnosticCaptureState::Awaiting { request, frame };
        self.device.flush_submissions()?;
        self.await_capture_delivery(&mut capture_state, request, frame, started, timeout)
    }

    /// Explicitly releases the persistent MVP frame resources before device teardown.
    pub fn destroy(self) -> Result<(), GraphicsError> {
        self.frame.destroy(&self.device)?;
        Ok(())
    }
}

impl NeutralMvpRenderer {
    fn lock_capture_state(
        &self,
        started: Instant,
        timeout: Duration,
    ) -> Result<MutexGuard<'_, DiagnosticCaptureState>, NeutralMvpCaptureError> {
        loop {
            match self.diagnostic_capture_state.try_lock() {
                Ok(state) => return Ok(state),
                Err(TryLockError::Poisoned(_)) => {
                    return Err(NeutralMvpCaptureError::CaptureGatePoisoned);
                }
                Err(TryLockError::WouldBlock) => {
                    remaining_timeout(started, timeout)?;
                    std::thread::yield_now();
                }
            }
        }
    }

    fn reap_previous_capture(
        &self,
        capture_state: &mut DiagnosticCaptureState,
        started: Instant,
        timeout: Duration,
    ) -> Result<(), NeutralMvpCaptureError> {
        let DiagnosticCaptureState::Awaiting { request, frame } = *capture_state else {
            return Ok(());
        };
        loop {
            remaining_timeout(started, timeout)?;
            self.device.poll_submissions()?;
            remaining_timeout(started, timeout)?;
            if let Some(delivery) = self.device.take_diagnostic_readback_delivery_for(request) {
                *capture_state = DiagnosticCaptureState::Idle;
                ensure_delivery_identity(delivery.receipt(), request, frame)?;
                remaining_timeout(started, timeout)?;
                return Ok(());
            }
            std::thread::yield_now();
        }
    }

    fn await_capture_delivery(
        &self,
        capture_state: &mut DiagnosticCaptureState,
        request: DiagnosticReadbackRequestId,
        frame: DiagnosticFrameKey,
        started: Instant,
        timeout: Duration,
    ) -> Result<Vec<u8>, NeutralMvpCaptureError> {
        loop {
            remaining_timeout(started, timeout)?;
            self.device.poll_submissions()?;
            remaining_timeout(started, timeout)?;
            if let Some(delivery) = self.device.take_diagnostic_readback_delivery_for(request) {
                let receipt = delivery.receipt();
                *capture_state = DiagnosticCaptureState::Idle;
                ensure_delivery_identity(receipt, request, frame)?;
                remaining_timeout(started, timeout)?;
                if receipt.terminal() != DiagnosticReadbackTerminal::Succeeded {
                    return Err(NeutralMvpCaptureError::ReadbackTerminal {
                        terminal: receipt.terminal(),
                    });
                }
                let bytes = delivery
                    .into_bytes()
                    .ok_or(NeutralMvpCaptureError::MissingPixelBytes)?;
                let expected = rgba8_byte_len(self.width, self.height)?;
                if bytes.len() != expected {
                    return Err(NeutralMvpCaptureError::UnexpectedPixelByteLength {
                        expected,
                        actual: bytes.len(),
                    });
                }
                return Ok(bytes);
            }
            std::thread::yield_now();
        }
    }
}

fn remaining_timeout(
    started: Instant,
    timeout: Duration,
) -> Result<Duration, NeutralMvpCaptureError> {
    timeout
        .checked_sub(started.elapsed())
        .ok_or(NeutralMvpCaptureError::TimedOut { timeout })
}

fn rgba8_byte_len(width: u32, height: u32) -> Result<usize, NeutralMvpCaptureError> {
    usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(NeutralMvpCaptureError::PixelByteLengthOverflow { width, height })
}

fn ensure_delivery_identity(
    receipt: DiagnosticReadbackReceipt,
    expected_request: DiagnosticReadbackRequestId,
    expected_frame: DiagnosticFrameKey,
) -> Result<(), NeutralMvpCaptureError> {
    (receipt.request() == expected_request && receipt.frame_key() == Some(expected_frame))
        .then_some(())
        .ok_or(NeutralMvpCaptureError::UnexpectedReadbackDelivery {
            expected_request,
            expected_frame,
            actual_request: receipt.request(),
            actual_frame: receipt.frame_key(),
        })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::NeutralMvpRenderer;

    #[test]
    fn neutral_mvp_renderer_captures_the_completed_offscreen_triangle() {
        let Ok(renderer) = NeutralMvpRenderer::new_offscreen(64, 64) else {
            return;
        };
        let pixels = renderer.capture_rgba8(62, Duration::from_secs(5)).unwrap();
        let center = ((32 * 64 + 32) * 4) as usize;
        assert_eq!(&pixels[center..center + 4], &[26, 204, 77, 255]);
        renderer.destroy().unwrap();
    }
}
