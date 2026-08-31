use std::collections::HashSet;

use zr_rhi::{
    CommandListCommand, GpuMemoryClass, RenderResourceHandleAllocator, RenderSurfaceDescriptor,
    RenderSurfaceHandleAllocator, RhiError, SurfaceAcquireOutcome, SurfaceFrameId,
    SurfaceFrameLease, SurfaceFrameTerminal, SurfaceSession, SurfaceSessionCreateOutcome,
    SurfaceSessionReceipt, SwapchainDesc, TextureDesc, TextureHandle, TextureUsage,
    TextureViewDesc, TextureViewDimension,
};

use crate::resource_validation::{texture_storage_size, validate_texture_desc};

use super::{resources, DeterministicRhiContractDeviceState, WgpuTextureResource};

#[derive(Clone, Debug)]
pub(super) struct DeterministicSurfaceSession {
    pub(super) descriptor: RenderSurfaceDescriptor,
}

#[derive(Clone, Debug)]
pub(super) struct DeterministicSurfaceFrame {
    pub(super) session: SurfaceSession,
    pub(super) target: TextureHandle,
    pub(super) default_view: zr_rhi::TextureViewHandle,
    desc: TextureDesc,
    submissions: HashSet<zr_rhi::SubmissionTicket>,
}

impl DeterministicRhiContractDeviceState {
    pub(super) fn create_surface_session(
        &mut self,
        surface_handles: &RenderSurfaceHandleAllocator,
        descriptor: &RenderSurfaceDescriptor,
    ) -> Result<SurfaceSessionCreateOutcome, RhiError> {
        let session = surface_handles.allocate_session()?;
        self.surface_sessions.insert(
            session,
            DeterministicSurfaceSession {
                descriptor: descriptor.clone(),
            },
        );
        Ok(surface_session_outcome(
            session,
            descriptor.swapchain.clone(),
        ))
    }

    pub(super) fn reconfigure_surface_session(
        &mut self,
        resource_handles: &RenderResourceHandleAllocator,
        surface_handles: &RenderSurfaceHandleAllocator,
        session: SurfaceSession,
        swapchain: &SwapchainDesc,
    ) -> Result<SurfaceSessionCreateOutcome, RhiError> {
        surface_handles.validate_session(session)?;
        let descriptor = self
            .surface_sessions
            .get(&session)
            .map(|surface| surface.descriptor.clone())
            .ok_or_else(|| {
                RhiError::SurfaceUnavailable("surface session is not live".to_string())
            })?;
        self.destroy_surface_session(resource_handles, surface_handles, session)?;
        self.create_surface_session(
            surface_handles,
            &RenderSurfaceDescriptor {
                label: descriptor.label,
                target: descriptor.target,
                swapchain: swapchain.clone(),
            },
        )
    }

    pub(super) fn acquire_surface_frame(
        &mut self,
        resource_handles: &RenderResourceHandleAllocator,
        surface_handles: &RenderSurfaceHandleAllocator,
        memory_budget: zr_rhi::GpuMemoryBudget,
        session: SurfaceSession,
    ) -> Result<SurfaceAcquireOutcome, RhiError> {
        surface_handles.validate_session(session)?;
        let descriptor = self
            .surface_sessions
            .get(&session)
            .map(|surface| surface.descriptor.clone())
            .ok_or_else(|| {
                RhiError::SurfaceUnavailable("surface session is not live".to_string())
            })?;
        if !descriptor.is_renderable() {
            return Ok(SurfaceAcquireOutcome::NonRenderable { session });
        }
        if self
            .surface_frames
            .values()
            .any(|frame| frame.session == session)
        {
            return Err(RhiError::SurfaceUnavailable(
                "surface session already has an acquired frame lease".to_string(),
            ));
        }

        let frame = surface_handles.allocate_frame()?;
        let allocation = (|| {
            let desc = TextureDesc::new(
                "deterministic-surface-frame-target",
                descriptor.swapchain.width,
                descriptor.swapchain.height,
                descriptor.swapchain.format,
                TextureUsage::PRESENT | TextureUsage::RENDER_ATTACHMENT,
            );
            validate_texture_desc(&desc, false)?;
            let requested_bytes = texture_storage_size(&desc);
            let snapshot = self.memory_snapshot();
            resources::ensure_memory_capacity(
                GpuMemoryClass::Texture,
                snapshot.active_texture_bytes,
                requested_bytes,
                memory_budget.transient_texture_bytes(),
            )?;
            let contents =
                resources::allocate_zeroed_contents(GpuMemoryClass::Texture, requested_bytes)?;
            let target = resource_handles.allocate_texture()?;
            let default_view = match resource_handles.allocate_texture_view() {
                Ok(view) => view,
                Err(error) => {
                    let _ = resource_handles.release_texture(target);
                    return Err(error.into());
                }
            };
            Ok::<_, RhiError>((desc, contents, target, default_view))
        })();
        let (desc, contents, target, default_view) = match allocation {
            Ok(allocation) => allocation,
            Err(error) => {
                let _ = surface_handles.release_frame(frame);
                return Err(error);
            }
        };

        self.textures.insert(
            target,
            WgpuTextureResource {
                desc: desc.clone(),
                contents,
            },
        );
        let view_desc = TextureViewDesc::new(
            "deterministic-surface-frame-default-view",
            target,
            TextureViewDimension::D2,
        );
        self.texture_views.insert(default_view, view_desc);
        self.texture_view_counts.insert(target, 1);
        self.surface_owned_textures.insert(target);
        self.surface_owned_texture_views.insert(default_view);
        self.surface_frames.insert(
            frame,
            DeterministicSurfaceFrame {
                session,
                target,
                default_view,
                desc: desc.clone(),
                submissions: HashSet::new(),
            },
        );
        Ok(SurfaceAcquireOutcome::Acquired(SurfaceFrameLease::new(
            frame,
            session,
            target,
            default_view,
            desc,
        )))
    }

    pub(super) fn record_surface_frame_submission(
        &mut self,
        ticket: zr_rhi::SubmissionTicket,
        commands: &[CommandListCommand],
    ) {
        for frame in self.surface_frames.values_mut() {
            if commands
                .iter()
                .any(|command| command_references_texture(command, frame.target))
            {
                frame.submissions.insert(ticket);
            }
        }
    }

    pub(super) fn surface_frame_has_submission(
        &self,
        surface_handles: &RenderSurfaceHandleAllocator,
        frame: &SurfaceFrameLease,
        ticket: zr_rhi::SubmissionTicket,
    ) -> Result<bool, RhiError> {
        self.validate_surface_frame_lease(surface_handles, frame)?;
        self.surface_frames
            .get(&frame.frame())
            .map(|resources| resources.submissions.contains(&ticket))
            .ok_or_else(|| RhiError::SurfaceUnavailable("surface frame is not live".to_string()))
    }

    pub(super) fn validate_surface_frame_lease(
        &self,
        surface_handles: &RenderSurfaceHandleAllocator,
        frame: &SurfaceFrameLease,
    ) -> Result<(), RhiError> {
        if let Some(terminal) = self.terminal_surface_frames.terminal(frame.frame()) {
            return Err(RhiError::SurfaceFrameAlreadyTerminal {
                frame: frame.frame(),
                terminal,
            });
        }
        surface_handles.validate_frame(frame.frame())?;
        let active = self
            .surface_frames
            .get(&frame.frame())
            .ok_or_else(|| RhiError::SurfaceUnavailable("surface frame is not live".to_string()))?;
        if active.session != frame.session()
            || active.target != frame.target()
            || active.default_view != frame.default_view()
            || active.desc != *frame.desc()
        {
            return Err(RhiError::SurfaceFrameLeaseMismatch {
                frame: frame.frame(),
            });
        }
        Ok(())
    }

    pub(super) fn terminalize_surface_frame(
        &mut self,
        resource_handles: &RenderResourceHandleAllocator,
        surface_handles: &RenderSurfaceHandleAllocator,
        frame: SurfaceFrameId,
        terminal: SurfaceFrameTerminal,
    ) -> Result<(), RhiError> {
        if let Some(previous_terminal) = self.terminal_surface_frames.terminal(frame) {
            return Err(RhiError::SurfaceFrameAlreadyTerminal {
                frame,
                terminal: previous_terminal,
            });
        }
        surface_handles.validate_frame(frame)?;
        self.cancel_accepted_surface_frame_submissions(frame)?;
        let frame_resources = self
            .surface_frames
            .remove(&frame)
            .ok_or_else(|| RhiError::SurfaceUnavailable("surface frame is not live".to_string()))?;
        self.surface_owned_texture_views
            .remove(&frame_resources.default_view);
        self.texture_views
            .remove(&frame_resources.default_view)
            .ok_or(RhiError::UnknownTextureView(
                frame_resources.default_view.diagnostic_id(),
            ))?;
        decrement_texture_view_count(&mut self.texture_view_counts, frame_resources.target);
        resource_handles.release_texture_view(frame_resources.default_view)?;
        self.surface_owned_textures.remove(&frame_resources.target);
        self.textures
            .remove(&frame_resources.target)
            .ok_or(RhiError::UnknownTexture(
                frame_resources.target.diagnostic_id(),
            ))?;
        resource_handles.release_texture(frame_resources.target)?;
        surface_handles.release_frame(frame)?;
        self.terminal_surface_frames.record(frame, terminal);
        Ok(())
    }

    fn cancel_accepted_surface_frame_submissions(
        &mut self,
        frame: SurfaceFrameId,
    ) -> Result<(), RhiError> {
        let mut tickets: Vec<_> = self
            .surface_frames
            .get(&frame)
            .ok_or_else(|| RhiError::SurfaceUnavailable("surface frame is not live".to_string()))?
            .submissions
            .iter()
            .copied()
            .collect();
        tickets.sort_by_key(|ticket| ticket.sequence());
        for ticket in tickets {
            if self.submission_history.status(ticket) != Some(zr_rhi::SubmissionStatus::Accepted) {
                continue;
            }
            let pending_index = self
                .pending_submissions
                .iter()
                .position(|submission| submission.ticket() == ticket)
                .ok_or(RhiError::UnknownSubmissionTicket(ticket))?;
            self.pending_submissions.remove(pending_index);
            debug_assert_eq!(
                self.submission_history
                    .transition(ticket, zr_rhi::SubmissionStatus::Cancelled),
                Some(zr_rhi::SubmissionStatus::Accepted)
            );
        }
        Ok(())
    }

    pub(super) fn destroy_surface_session(
        &mut self,
        resource_handles: &RenderResourceHandleAllocator,
        surface_handles: &RenderSurfaceHandleAllocator,
        session: SurfaceSession,
    ) -> Result<(), RhiError> {
        surface_handles.validate_session(session)?;
        let frames: Vec<_> = self
            .surface_frames
            .iter()
            .filter_map(|(frame, resources)| (resources.session == session).then_some(*frame))
            .collect();
        for frame in frames {
            self.terminalize_surface_frame(
                resource_handles,
                surface_handles,
                frame,
                SurfaceFrameTerminal::Discarded,
            )?;
        }
        self.surface_sessions.remove(&session).ok_or_else(|| {
            RhiError::SurfaceUnavailable("surface session is not live".to_string())
        })?;
        surface_handles.release_session(session)?;
        Ok(())
    }
}

fn command_references_texture(command: &CommandListCommand, texture: TextureHandle) -> bool {
    match command {
        CommandListCommand::CopyBufferToTexture { destination, .. } => *destination == texture,
        CommandListCommand::CopyTextureToBuffer { source, .. } => *source == texture,
        CommandListCommand::CopyTextureToTexture {
            source,
            destination,
            ..
        } => *source == texture || *destination == texture,
        CommandListCommand::BeginRenderPass {
            color_attachments,
            depth_stencil_attachment,
            ..
        }
        | CommandListCommand::BeginRenderPassWithDiagnostics {
            color_attachments,
            depth_stencil_attachment,
            ..
        } => {
            color_attachments.iter().any(|attachment| {
                attachment.view.texture == texture
                    || attachment
                        .resolve_target
                        .is_some_and(|view| view.texture == texture)
            }) || depth_stencil_attachment
                .as_ref()
                .is_some_and(|attachment| attachment.view.texture == texture)
        }
        _ => false,
    }
}

fn surface_session_outcome(
    session: SurfaceSession,
    swapchain: SwapchainDesc,
) -> SurfaceSessionCreateOutcome {
    let receipt = SurfaceSessionReceipt::new(session, swapchain);
    if receipt.swapchain.width == 0 || receipt.swapchain.height == 0 {
        SurfaceSessionCreateOutcome::NonRenderable(receipt)
    } else {
        SurfaceSessionCreateOutcome::Renderable(receipt)
    }
}

fn decrement_texture_view_count(
    texture_view_counts: &mut std::collections::HashMap<TextureHandle, u32>,
    texture: TextureHandle,
) {
    let Some(count) = texture_view_counts.get_mut(&texture) else {
        return;
    };
    if *count <= 1 {
        texture_view_counts.remove(&texture);
    } else {
        *count -= 1;
    }
}

#[cfg(test)]
mod lease_validation_tests {
    use zr_rhi::{
        PresentMode, RenderDevice, RenderNativeSurfaceTarget, RenderSurfaceDescriptor, RhiError,
        SurfaceAcquireOutcome, SurfaceFrameLease, SurfaceSessionCreateOutcome, SwapchainDesc,
        TextureFormat,
    };

    use crate::DeterministicRhiContractDevice;

    fn surface_descriptor(label: &str) -> RenderSurfaceDescriptor {
        RenderSurfaceDescriptor::new(
            label,
            RenderNativeSurfaceTarget::Win32 {
                hwnd: 1,
                hinstance: None,
            },
            SwapchainDesc {
                width: 64,
                height: 64,
                present_mode: PresentMode::Fifo,
                format: TextureFormat::Bgra8UnormSrgb,
            },
        )
    }

    fn acquire_frame(device: &DeterministicRhiContractDevice, label: &str) -> SurfaceFrameLease {
        let SurfaceSessionCreateOutcome::Renderable(session) = device
            .create_surface_session(&surface_descriptor(label))
            .expect("deterministic surface session")
        else {
            panic!("nonzero deterministic surface must be renderable");
        };
        let SurfaceAcquireOutcome::Acquired(frame) = device
            .acquire_surface_frame(session.session())
            .expect("deterministic surface frame")
        else {
            panic!("renderable deterministic surface must acquire a frame");
        };
        frame
    }

    #[test]
    fn forged_surface_frame_lease_cannot_terminalize_an_active_frame() {
        let device = DeterministicRhiContractDevice::new_headless();
        let first = acquire_frame(&device, "first-surface");
        let second = acquire_frame(&device, "second-surface");
        let mut forged_desc = first.desc().clone();
        forged_desc.label = Some("forged-surface-frame".to_string());
        let forged = [
            SurfaceFrameLease::new(
                first.frame(),
                second.session(),
                first.target(),
                first.default_view(),
                first.desc().clone(),
            ),
            SurfaceFrameLease::new(
                first.frame(),
                first.session(),
                second.target(),
                first.default_view(),
                first.desc().clone(),
            ),
            SurfaceFrameLease::new(
                first.frame(),
                first.session(),
                first.target(),
                second.default_view(),
                first.desc().clone(),
            ),
            SurfaceFrameLease::new(
                first.frame(),
                first.session(),
                first.target(),
                first.default_view(),
                forged_desc,
            ),
        ];

        for lease in forged {
            assert!(matches!(
                device.discard_surface_frame(lease),
                Err(RhiError::SurfaceFrameLeaseMismatch { frame }) if frame == first.frame()
            ));
        }

        device
            .discard_surface_frame(first)
            .expect("rejected forged leases must leave the first frame active");
        device
            .discard_surface_frame(second)
            .expect("the second frame must remain independently active");
    }
}
