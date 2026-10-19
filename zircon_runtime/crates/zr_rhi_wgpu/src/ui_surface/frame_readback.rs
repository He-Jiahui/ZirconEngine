//! Optional readback owned by the actual normal presenter and existing device timeline.
use super::WgpuUiSurfaceRenderer;
use std::sync::{Arc, Mutex};
use web_time::{Duration, Instant};
use zr_rhi::{
    DiagnosticReadbackAdmission, DiagnosticReadbackReceipt, DiagnosticReadbackRequestId,
    DiagnosticReadbackTerminal, RenderDevice, RhiError, SubmissionTicket, UiSurfaceDrawList,
    UiSurfaceFramePixelFormat, UiSurfaceFrameReadback, UiSurfaceFrameReadbackRequest,
};

type Delivery = Arc<Mutex<Option<(DiagnosticReadbackReceipt, Option<Vec<u8>>)>>>;
pub(super) struct PendingFrameReadback {
    request: UiSurfaceFrameReadbackRequest,
    deadline: Instant,
    epoch: u64,
    present_index: u64,
    id: Option<DiagnosticReadbackRequestId>,
    submission: Option<SubmissionTicket>,
    format: Option<UiSurfaceFramePixelFormat>,
    delivery: Delivery,
    failure: Option<String>,
}
impl WgpuUiSurfaceRenderer {
    pub(super) fn has_submitted_frame_readback(&self) -> bool {
        self.frame_readback
            .as_ref()
            .is_some_and(|pending| pending.submission.is_some())
    }

    pub(super) fn request_frame_readback(
        &mut self,
        request: UiSurfaceFrameReadbackRequest,
    ) -> Result<(), RhiError> {
        if self.frame_readback.is_some()
            || self.consumed_readback_nonces.len() >= 256
            || self.consumed_readback_nonces.contains(&request.nonce)
        {
            return Err(RhiError::SurfaceUnavailable(
                "normal readback request already pending or nonce consumed".into(),
            ));
        }
        if request.nonce.is_empty()
            || request.nonce.len() > 96
            || !request
                .nonce
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || !(50..=10000).contains(&request.deadline_ms)
            || request.physical_size != (self.config.width, self.config.height)
        {
            return Err(RhiError::SurfaceUnavailable(
                "normal readback request identity, extent or deadline invalid".into(),
            ));
        }
        if !self.completion_owner.is_local() && self.diagnostic_completion_owner.is_none() {
            return Err(RhiError::SurfaceUnavailable(
                "external normal readback completion owner unavailable".into(),
            ));
        }
        self.consumed_readback_nonces.push(request.nonce.clone());
        self.frame_readback = Some(PendingFrameReadback {
            deadline: Instant::now() + Duration::from_millis(request.deadline_ms),
            request,
            epoch: self.surface_epoch,
            present_index: 0,
            id: None,
            submission: None,
            format: None,
            delivery: Arc::new(Mutex::new(None)),
            failure: None,
        });
        Ok(())
    }
    pub(super) fn cancel_frame_readback(&mut self, reason: &str) {
        if let Some(pending) = self.frame_readback.as_mut() {
            pending.failure = Some(reason.to_owned());
            if let (Some(owner), Some(id)) = (&self.diagnostic_completion_owner, pending.id) {
                owner.cancel_receipt_callback(id);
            }
        }
    }
    pub(super) fn prepare_frame_readback(
        &mut self,
        draw_list: &UiSurfaceDrawList,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<crate::WgpuNativeDiagnosticReadbackFrame> {
        let pending = self.frame_readback.as_ref()?;
        if pending.failure.is_some() || pending.id.is_some() {
            return None;
        }
        if Instant::now() >= pending.deadline
            || pending.epoch != self.surface_epoch
            || pending.request.physical_size != draw_list.surface_size
            || pending.request.expected_draw_generation != draw_list.generation()
            || draw_list.is_target_only_resize()
        {
            self.cancel_frame_readback("normal readback deadline, extent or generation changed");
            return None;
        }
        let Some(cache) = self
            .retained_cache
            .as_ref()
            .filter(|cache| cache.matches(self.config.format, draw_list.surface_size))
        else {
            self.cancel_frame_readback(
                "surface does not support exact retained-cache COPY_DST presentation",
            );
            return None;
        };
        let texture = cache.texture().clone();
        let format = match texture.format() {
            wgpu::TextureFormat::Rgba8UnormSrgb => UiSurfaceFramePixelFormat::Rgba8Srgb,
            wgpu::TextureFormat::Bgra8UnormSrgb => UiSurfaceFramePixelFormat::Bgra8Srgb,
            wgpu::TextureFormat::Rgba8Unorm => UiSurfaceFramePixelFormat::Rgba8Linear,
            wgpu::TextureFormat::Bgra8Unorm => UiSurfaceFramePixelFormat::Bgra8Linear,
            _ => {
                self.cancel_frame_readback("surface pixel format unsupported");
                return None;
            }
        };
        // Never consume or clear an already admitted scene batch. begin_frame rejects it.
        if let Err(error) = self
            .render_device
            .begin_diagnostic_readback_frame(self.present_index)
        {
            self.cancel_frame_readback(&format!("normal diagnostic batch unavailable: {error}"));
            return None;
        }
        let admission = self
            .render_device
            .enqueue_native_diagnostic_texture_color8_readback(
                &texture,
                draw_list.surface_size.0,
                draw_list.surface_size.1,
            );
        match admission {
            Ok(DiagnosticReadbackAdmission::Admitted(id)) => {
                let pending = self.frame_readback.as_mut().expect("request remains owned");
                pending.id = Some(id);
                pending.format = Some(format);
                pending.present_index = self.present_index;
                if let Some(owner) = &self.diagnostic_completion_owner {
                    let delivery = Arc::clone(&pending.delivery);
                    match owner.register_receipt_callback(
                        DiagnosticReadbackAdmission::Admitted(id),
                        Box::new(move |receipt, bytes| {
                            *delivery.lock().unwrap_or_else(|p| p.into_inner()) =
                                Some((receipt, bytes));
                        }),
                    ) {
                        Ok(true) => {}
                        Ok(false) => {
                            self.cancel_frame_readback("normal receipt route was not admitted")
                        }
                        Err(error) => self.cancel_frame_readback(&format!(
                            "normal receipt route failed: {error}"
                        )),
                    }
                }
            }
            Ok(DiagnosticReadbackAdmission::Rejected(receipt)) => {
                self.cancel_frame_readback(&format!(
                    "normal readback quota rejected: {:?}",
                    receipt.terminal()
                ));
                let pending = self.frame_readback.as_mut().unwrap();
                *pending.delivery.lock().unwrap_or_else(|p| p.into_inner()) = Some((receipt, None));
            }
            Err(error) => self
                .cancel_frame_readback(&format!("normal color readback admission failed: {error}")),
        }
        // Seal the batch once even on quota rejection; no global abort or extra submission.
        match self
            .render_device
            .prepare_native_diagnostic_readback_frame("zircon-normal-surface-readback", encoder)
        {
            Ok(frame) => frame,
            Err(error) => {
                self.cancel_frame_readback(&format!("normal readback encoding failed: {error}"));
                None
            }
        }
    }
    pub(super) fn bind_frame_readback_submission(&mut self, ticket: SubmissionTicket) {
        if let Some(pending) = self
            .frame_readback
            .as_mut()
            .filter(|p| p.id.is_some() && p.failure.is_none())
        {
            pending.submission = Some(ticket);
        }
    }
    pub(super) fn drain_frame_readback(
        &mut self,
    ) -> Result<Option<UiSurfaceFrameReadback>, RhiError> {
        let Some(pending) = self.frame_readback.as_ref() else {
            return Ok(None);
        };
        if Instant::now() >= pending.deadline {
            self.cancel_frame_readback("normal readback completion deadline expired");
        }
        if self.frame_readback.as_ref().is_some_and(|p| {
            p.epoch != self.surface_epoch
                || p.request.physical_size != (self.config.width, self.config.height)
                || (p.submission.is_some() && p.present_index != self.present_index)
        }) {
            self.cancel_frame_readback(
                "normal readback surface epoch, extent or actual present changed",
            );
        }
        if self
            .frame_readback
            .as_ref()
            .is_some_and(|p| p.failure.is_none() && p.submission.is_some())
        {
            let polled = if self.completion_owner.is_local() {
                self.render_device.poll_submissions()
            } else {
                self.diagnostic_completion_owner
                    .as_ref()
                    .expect("external owner checked at admission")
                    .drain_once()
            };
            if let Err(error) = polled {
                self.cancel_frame_readback(&format!("normal completion owner failed: {error}"));
            } else if self.completion_owner.is_local() {
                let pending = self.frame_readback.as_mut().unwrap();
                if let Some(delivery) = pending
                    .id
                    .and_then(|id| self.render_device.take_diagnostic_readback_delivery_for(id))
                {
                    let receipt = delivery.receipt();
                    *pending.delivery.lock().unwrap_or_else(|p| p.into_inner()) =
                        Some((receipt, delivery.into_bytes()));
                }
            }
        }
        let terminal = {
            let pending = self.frame_readback.as_ref().unwrap();
            pending.failure.is_some()
                || pending
                    .delivery
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .is_some()
        };
        if !terminal {
            return Ok(None);
        }
        let mut pending = self.frame_readback.take().unwrap();
        if let (Some(owner), Some(id)) = (&self.diagnostic_completion_owner, pending.id) {
            owner.cancel_receipt_callback(id);
        }
        let observed = pending
            .delivery
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take();
        let receipt = observed.as_ref().map(|(receipt, _)| *receipt);
        let mut bytes = observed.and_then(|(_, bytes)| bytes);
        if pending.failure.is_none()
            && !receipt.is_some_and(|r| {
                r.request() == pending.id.unwrap()
                    && r.terminal() == DiagnosticReadbackTerminal::Succeeded
                    && r.frame_key().map(|key| key.submission()) == pending.submission
            })
        {
            pending.failure =
                Some("normal readback receipt does not match successful actual submission".into());
        }
        let opaque_pixels = bytes
            .as_ref()
            .map(|bytes| bytes.chunks_exact(4).all(|p| p[3] == 255));
        if pending.failure.is_none() {
            let expected = u64::from(pending.request.physical_size.0)
                * u64::from(pending.request.physical_size.1)
                * 4;
            if bytes
                .as_ref()
                .is_none_or(|bytes| bytes.len() as u64 != expected)
            {
                pending.failure = Some("normal readback payload extent invalid".into());
            }
        }

        if pending.failure.is_some() {
            bytes = None;
        }
        Ok(Some(UiSurfaceFrameReadback {
            request: pending.request,
            surface_epoch: pending.epoch,
            present_index: pending.present_index,
            submission: pending.submission,
            receipt,
            pixel_format: pending.format,
            color8: bytes,
            opaque_pixels,
            surface_alpha_mode: format!("{:?}", self.config.alpha_mode),
            native_target: self.native_target,
            failure: pending.failure,
        }))
    }
}
