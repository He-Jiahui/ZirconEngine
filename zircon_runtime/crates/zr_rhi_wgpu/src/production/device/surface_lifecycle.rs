use std::sync::MutexGuard;

use zr_rhi::{
    RenderDevice, RenderSurfaceDescriptor, RhiError, SubmissionStatus, SubmissionTicket,
    SurfaceAcquireOutcome, SurfaceFrameLease, SurfacePresentReceipt, SurfaceSession,
    SurfaceSessionCreateOutcome, SwapchainDesc,
};

use super::{WgpuRenderDevice, WgpuSurfaceService};

impl WgpuRenderDevice {
    pub(super) fn lock_surfaces(&self) -> MutexGuard<'_, WgpuSurfaceService> {
        self.surfaces
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn terminalize_surface_frames(&self) {
        let mut surfaces = self.lock_surfaces();
        let mut registry = self.lock_registry();
        let _ = surfaces.terminalize_all(&mut registry);
    }

    fn settle_surface_submissions(&self, tickets: &[SubmissionTicket]) -> Result<(), RhiError> {
        let statuses = self.submissions.settle_abandoned_submissions(tickets)?;
        let mut diagnostics = self.lock_diagnostics();
        for (&ticket, status) in tickets.iter().zip(statuses) {
            if status == SubmissionStatus::Cancelled {
                diagnostics
                    .terminalize_submission(ticket, zr_rhi::DiagnosticReadbackTerminal::Cancelled);
            }
        }
        Ok(())
    }

    /// Opens an additional session for an already initialized device generation.
    ///
    /// Primary native startup must use [`super::super::WgpuSurfaceBootstrap`] so its adapter is
    /// selected against the native surface before a device exists.
    pub(super) fn create_surface_session_impl(
        &self,
        descriptor: &RenderSurfaceDescriptor,
    ) -> Result<SurfaceSessionCreateOutcome, RhiError> {
        self.ensure_admission()?;
        if !self.caps.supports_surface {
            return Err(RhiError::SurfaceUnavailable(
                "native surface sessions are unavailable for this build target".to_string(),
            ));
        }
        self.lock_surfaces()
            .create_session(&self.instance, &self.adapter, &self.device, descriptor)
    }

    pub(crate) fn adopt_surface_session(
        &self,
        descriptor: RenderSurfaceDescriptor,
        surface: wgpu::Surface<'static>,
    ) -> Result<SurfaceSessionCreateOutcome, RhiError> {
        self.ensure_admission()?;
        if !self.caps.supports_surface {
            return Err(RhiError::SurfaceUnavailable(
                "native surface sessions are unavailable for this build target".to_string(),
            ));
        }
        self.lock_surfaces()
            .adopt_session(&self.adapter, &self.device, descriptor, surface)
    }

    pub(super) fn reconfigure_surface_session_impl(
        &self,
        session: SurfaceSession,
        swapchain: &SwapchainDesc,
    ) -> Result<SurfaceSessionCreateOutcome, RhiError> {
        self.ensure_admission()?;
        let result = {
            let mut surfaces = self.lock_surfaces();
            let mut registry = self.lock_registry();
            let tickets = surfaces.session_submission_tickets(&registry, session)?;
            self.settle_surface_submissions(&tickets)?;
            surfaces.reconfigure_session(
                &self.adapter,
                &self.device,
                &mut registry,
                session,
                swapchain,
            )
        };
        self.prune_terminal_resources();
        result
    }

    pub(super) fn acquire_surface_frame_impl(
        &self,
        session: SurfaceSession,
    ) -> Result<SurfaceAcquireOutcome, RhiError> {
        self.ensure_admission()?;
        let mut surfaces = self.lock_surfaces();
        let mut registry = self.lock_registry();
        surfaces.acquire_frame(&mut registry, session)
    }

    pub(super) fn present_surface_frame_impl(
        &self,
        frame: SurfaceFrameLease,
        submission: SubmissionTicket,
    ) -> Result<SurfacePresentReceipt, RhiError> {
        self.ensure_admission()?;
        if submission.device_id() != self.device_id()
            || submission.generation() != self.generation()
        {
            return Err(RhiError::SurfaceFrameSubmissionMismatch {
                frame: frame.frame(),
                submission,
            });
        }
        let status = self.submissions.status(submission)?;
        if !matches!(
            status,
            SubmissionStatus::Submitted | SubmissionStatus::Completed
        ) {
            return Err(RhiError::SurfaceFrameSubmissionNotReady {
                frame: frame.frame(),
                status,
            });
        }
        let mut surfaces = self.lock_surfaces();
        let mut registry = self.lock_registry();
        surfaces.present_frame(&mut registry, &frame, submission)
    }

    pub(super) fn discard_surface_frame_impl(
        &self,
        frame: SurfaceFrameLease,
    ) -> Result<(), RhiError> {
        let result = {
            let mut surfaces = self.lock_surfaces();
            let mut registry = self.lock_registry();
            surfaces.validate_frame_lease(&frame)?;
            let tickets = surfaces.frame_submission_tickets(&registry, frame.frame())?;
            self.settle_surface_submissions(&tickets)?;
            surfaces.discard_frame(&mut registry, &frame)
        };
        self.prune_terminal_resources();
        result
    }

    pub(super) fn destroy_surface_session_impl(
        &self,
        session: SurfaceSession,
    ) -> Result<(), RhiError> {
        let result = {
            let mut surfaces = self.lock_surfaces();
            let mut registry = self.lock_registry();
            let tickets = surfaces.session_submission_tickets(&registry, session)?;
            self.settle_surface_submissions(&tickets)?;
            surfaces.destroy_session(&mut registry, session)
        };
        self.prune_terminal_resources();
        result
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Barrier};

    use zr_rhi::{
        BufferDesc, BufferUsage, DeviceGeneration, DiagnosticReadbackAdmission,
        DiagnosticReadbackBudget, DiagnosticReadbackTerminal, GpuMemoryBudget, RenderDeviceProfile,
        RenderDeviceQueueTopology, RenderDeviceRequestPolicy, RenderQueueClass, SubmissionLimits,
    };

    use super::super::WgpuRenderDeviceContext;
    use super::*;
    use crate::{next_wgpu_device_id, wgpu_adapter_facts, wgpu_device_limits, wgpu_device_request};

    #[test]
    fn surface_teardown_settles_tickets_under_one_submission_lock() {
        let source = include_str!("surface_lifecycle.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production surface lifecycle source");

        assert!(production.contains("settle_abandoned_submissions(tickets)?"));
        assert!(!production.contains("self.submissions.status(ticket)?"));
        assert!(!production.contains("self.submissions.cancel(ticket)?"));
    }

    #[test]
    fn surface_teardown_rejects_a_mixed_unknown_batch_without_partial_cancellation() {
        let device = production_test_device();
        let valid = device
            .submissions
            .begin_packet(RenderQueueClass::Copy)
            .unwrap();
        let unknown = SubmissionTicket::new(
            valid.device_id(),
            valid.generation(),
            valid.queue_class(),
            valid.sequence().saturating_add(1_024),
        );

        assert!(matches!(
            device
                .submissions
                .settle_abandoned_submissions(&[valid, unknown]),
            Err(RhiError::UnknownSubmissionTicket(ticket)) if ticket == unknown
        ));
        assert_eq!(
            device.submissions.status(valid).unwrap(),
            SubmissionStatus::Accepted
        );
        assert_eq!(
            device.submissions.cancel(valid).unwrap(),
            SubmissionStatus::Cancelled
        );
    }

    #[test]
    fn surface_teardown_settles_reserved_pending_and_duplicate_tickets_once() {
        let device = production_test_device();
        let reserved = device
            .submissions
            .begin_packet(RenderQueueClass::Copy)
            .unwrap();
        let pending = device
            .submissions
            .begin_packet(RenderQueueClass::Copy)
            .unwrap();
        device
            .submissions
            .commit_packet(pending, Vec::new())
            .unwrap();

        assert_eq!(device.command_context_pool_counts_for_tests(), (2, 0));
        assert_eq!(
            device
                .submissions
                .settle_abandoned_submissions(&[pending, reserved, pending])
                .unwrap(),
            vec![
                SubmissionStatus::Cancelled,
                SubmissionStatus::Cancelled,
                SubmissionStatus::Cancelled,
            ]
        );
        assert_eq!(device.command_context_pool_counts_for_tests(), (2, 2));
        assert_eq!(device.submissions.flush().unwrap(), 0);
    }

    #[test]
    fn surface_teardown_terminalizes_diagnostic_delivery_exactly_once() {
        let device = production_test_device();
        let source = device
            .create_buffer(&BufferDesc::new(
                "surface-teardown-diagnostic-source",
                4,
                BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
            ))
            .unwrap();
        device.begin_diagnostic_readback_frame(91).unwrap();
        let request = match device
            .enqueue_diagnostic_buffer_readback(source, 0, 4)
            .unwrap()
        {
            DiagnosticReadbackAdmission::Admitted(request) => request,
            DiagnosticReadbackAdmission::Rejected(receipt) => {
                panic!("readback request unexpectedly rejected: {receipt:?}")
            }
        };
        let frame = device
            .submit_diagnostic_readback_frame("surface-teardown-diagnostic")
            .unwrap()
            .expect("one admitted request must produce a submission-qualified frame");
        let ticket = frame.submission();

        device.settle_surface_submissions(&[ticket]).unwrap();
        assert_eq!(
            device.submissions.status(ticket).unwrap(),
            SubmissionStatus::Cancelled
        );
        let delivery = device
            .take_diagnostic_readback_delivery()
            .expect("teardown must publish one cancelled diagnostic delivery");
        assert_eq!(delivery.receipt().request(), request);
        assert_eq!(delivery.receipt().frame_key(), Some(frame));
        assert_eq!(
            delivery.receipt().terminal(),
            DiagnosticReadbackTerminal::Cancelled
        );
        assert_eq!(delivery.bytes(), None);

        device.settle_surface_submissions(&[ticket]).unwrap();
        assert!(device.take_diagnostic_readback_delivery().is_none());
    }

    #[test]
    fn surface_teardown_racing_flush_has_only_atomic_terminal_outcomes() {
        const BATCH_SIZE: usize = 4;

        let device = Arc::new(production_test_device());

        let settle_first = committed_pending_tickets(&device, BATCH_SIZE);
        assert!(device
            .submissions
            .settle_abandoned_submissions(&settle_first)
            .unwrap()
            .iter()
            .all(|status| *status == SubmissionStatus::Cancelled));
        assert_eq!(device.submissions.flush().unwrap(), 0);
        assert!(settle_first.iter().all(|ticket| {
            device.submissions.status(*ticket).unwrap() == SubmissionStatus::Cancelled
        }));

        let flush_first = committed_pending_tickets(&device, BATCH_SIZE);
        assert_eq!(device.submissions.flush().unwrap(), BATCH_SIZE);
        assert!(device
            .submissions
            .settle_abandoned_submissions(&flush_first)
            .unwrap()
            .iter()
            .all(is_submitted_or_completed));
        assert!(flush_first
            .iter()
            .all(|ticket| is_submitted_or_completed(&device.submissions.status(*ticket).unwrap())));

        for _ in 0..16 {
            let tickets = committed_pending_tickets(&device, BATCH_SIZE);
            let barrier = Arc::new(Barrier::new(3));
            let flush_device = Arc::clone(&device);
            let flush_barrier = Arc::clone(&barrier);
            let flush = std::thread::spawn(move || {
                flush_barrier.wait();
                flush_device.submissions.flush()
            });
            let settle_device = Arc::clone(&device);
            let settle_barrier = Arc::clone(&barrier);
            let settle_tickets = tickets.clone();
            let settle = std::thread::spawn(move || {
                settle_barrier.wait();
                settle_device
                    .submissions
                    .settle_abandoned_submissions(&settle_tickets)
            });
            barrier.wait();

            let flushed = flush.join().expect("flush worker panicked").unwrap();
            let settled = settle.join().expect("settle worker panicked").unwrap();
            match flushed {
                0 => {
                    assert!(settled
                        .iter()
                        .all(|status| *status == SubmissionStatus::Cancelled));
                    assert!(tickets.iter().all(|ticket| {
                        device.submissions.status(*ticket).unwrap() == SubmissionStatus::Cancelled
                    }));
                }
                BATCH_SIZE => {
                    assert!(settled.iter().all(is_submitted_or_completed));
                    assert!(tickets.iter().all(|ticket| is_submitted_or_completed(
                        &device.submissions.status(*ticket).unwrap()
                    )));
                }
                partial => {
                    panic!("flush and surface settlement split one batch: {partial}/{BATCH_SIZE}")
                }
            }
        }
    }

    fn committed_pending_tickets(device: &WgpuRenderDevice, count: usize) -> Vec<SubmissionTicket> {
        (0..count)
            .map(|_| {
                let ticket = device
                    .submissions
                    .begin_packet(RenderQueueClass::Copy)
                    .unwrap();
                device
                    .submissions
                    .commit_packet(ticket, Vec::new())
                    .unwrap();
                ticket
            })
            .collect()
    }

    fn is_submitted_or_completed(status: &SubmissionStatus) -> bool {
        matches!(
            status,
            SubmissionStatus::Submitted | SubmissionStatus::Completed
        )
    }

    fn production_test_device() -> WgpuRenderDevice {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .expect("surface teardown behavior tests require a WGPU adapter");
        let policy = RenderDeviceRequestPolicy::mvp_baseline();
        let request = wgpu_device_request(adapter.features(), &policy)
            .expect("the test adapter must satisfy the MVP device policy");
        let adapter_facts = wgpu_adapter_facts(&adapter.get_info(), adapter.features());
        let (native_device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("zircon-surface-teardown-test-device"),
                required_features: request.requested_features(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
            }))
            .expect("the test adapter must create the requested WGPU device");
        let profile = RenderDeviceProfile::new(
            next_wgpu_device_id(),
            DeviceGeneration::initial(),
            adapter_facts,
            request.feature_negotiation().clone(),
            wgpu_device_limits(&native_device.limits()),
            RenderDeviceQueueTopology::single_serialized_queue(),
            GpuMemoryBudget::reference_1080p_mid(),
            SubmissionLimits::default(),
            DiagnosticReadbackBudget::default(),
        );
        WgpuRenderDevice::new(
            WgpuRenderDeviceContext::new(instance, adapter, native_device, queue),
            profile,
        )
        .expect("the test context and profile must describe the same adapter")
    }
}
