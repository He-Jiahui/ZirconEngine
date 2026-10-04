//! 表面会话变更先结算仍被帧引用的提交，再释放逻辑句柄并按票据延迟回收原生资源。
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

    // 重建会话前结算相关接受票据，并保持表面、资源表和提交状态的锁顺序。
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
#[path = "tests/surface_lifecycle.rs"]
mod tests;
