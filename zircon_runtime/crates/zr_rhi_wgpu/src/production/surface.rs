use std::collections::{HashMap, HashSet};
#[cfg(target_os = "windows")]
use std::num::NonZeroIsize;

use zr_rhi::{
    DeviceGeneration, DeviceId, PresentMode, RenderNativeSurfaceTarget, RenderSurfaceDescriptor,
    RenderSurfaceHandleAllocator, RhiError, SubmissionTicket, SurfaceAcquireOutcome,
    SurfaceFrameId, SurfaceFrameLease, SurfaceFrameTerminal, SurfaceFrameTerminalHistory,
    SurfacePresentReceipt, SurfaceReconfigureReason, SurfaceRetryReason, SurfaceSession,
    SurfaceSessionCreateOutcome, SurfaceSessionReceipt, SwapchainDesc, TextureDesc, TextureFormat,
    TextureHandle, TextureUsage, TextureViewHandle,
};

use super::WgpuResourceRegistry;

pub(crate) struct WgpuSurfaceService {
    handles: RenderSurfaceHandleAllocator,
    frames: HashMap<SurfaceFrameId, WgpuSurfaceFrame>,
    sessions: HashMap<SurfaceSession, WgpuSurfaceSession>,
    terminal_frames: SurfaceFrameTerminalHistory,
}

struct WgpuSurfaceSession {
    descriptor: RenderSurfaceDescriptor,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    swapchain: SwapchainDesc,
}

struct WgpuSurfaceFrame {
    session: SurfaceSession,
    target: TextureHandle,
    default_view: TextureViewHandle,
    desc: TextureDesc,
    native: wgpu::SurfaceTexture,
}

impl WgpuSurfaceService {
    pub(crate) fn new(
        device_id: DeviceId,
        generation: DeviceGeneration,
        max_terminal_frames: usize,
    ) -> Self {
        Self {
            handles: RenderSurfaceHandleAllocator::new(device_id, generation),
            frames: HashMap::new(),
            sessions: HashMap::new(),
            terminal_frames: SurfaceFrameTerminalHistory::new(max_terminal_frames),
        }
    }

    pub(crate) fn create_session(
        &mut self,
        instance: &wgpu::Instance,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        descriptor: &RenderSurfaceDescriptor,
    ) -> Result<SurfaceSessionCreateOutcome, RhiError> {
        let surface = create_native_surface(instance, descriptor.target)?;
        self.adopt_session(adapter, device, descriptor.clone(), surface)
    }

    pub(crate) fn adopt_session(
        &mut self,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        descriptor: RenderSurfaceDescriptor,
        surface: wgpu::Surface<'static>,
    ) -> Result<SurfaceSessionCreateOutcome, RhiError> {
        let (config, swapchain) = negotiate_surface_config(&surface, adapter, &descriptor)?;
        if descriptor.is_renderable() {
            surface.configure(device, &config);
        }
        let session = self.handles.allocate_session()?;
        self.sessions.insert(
            session,
            WgpuSurfaceSession {
                descriptor,
                surface,
                config,
                swapchain: swapchain.clone(),
            },
        );
        Ok(surface_session_outcome(session, swapchain))
    }

    pub(crate) fn reconfigure_session(
        &mut self,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        registry: &mut WgpuResourceRegistry,
        session: SurfaceSession,
        swapchain: &SwapchainDesc,
    ) -> Result<SurfaceSessionCreateOutcome, RhiError> {
        self.handles.validate_session(session)?;
        let mut old = self.sessions.remove(&session).ok_or_else(|| {
            RhiError::SurfaceUnavailable("surface session is not live".to_string())
        })?;
        self.discard_session_frames(registry, session)?;
        self.handles.release_session(session)?;
        old.descriptor.swapchain = swapchain.clone();
        let (config, negotiated) =
            negotiate_surface_config(&old.surface, adapter, &old.descriptor)?;
        if old.descriptor.is_renderable() {
            old.surface.configure(device, &config);
        }
        old.config = config;
        old.swapchain = negotiated.clone();
        let replacement = self.handles.allocate_session()?;
        self.sessions.insert(replacement, old);
        Ok(surface_session_outcome(replacement, negotiated))
    }

    pub(crate) fn acquire_frame(
        &mut self,
        registry: &mut WgpuResourceRegistry,
        session: SurfaceSession,
    ) -> Result<SurfaceAcquireOutcome, RhiError> {
        self.handles.validate_session(session)?;
        let surface = self.sessions.get(&session).ok_or_else(|| {
            RhiError::SurfaceUnavailable("surface session is not live".to_string())
        })?;
        if !surface.descriptor.is_renderable() {
            return Ok(SurfaceAcquireOutcome::NonRenderable { session });
        }
        if self.frames.values().any(|frame| frame.session == session) {
            return Err(RhiError::SurfaceUnavailable(
                "surface session already has an acquired frame lease".to_string(),
            ));
        }

        let acquired = match surface.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(_) => {
                return Ok(SurfaceAcquireOutcome::ReconfigureRequired {
                    session,
                    reason: SurfaceReconfigureReason::Suboptimal,
                });
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                return Ok(SurfaceAcquireOutcome::ReconfigureRequired {
                    session,
                    reason: SurfaceReconfigureReason::Outdated,
                });
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                return Ok(SurfaceAcquireOutcome::ReconfigureRequired {
                    session,
                    reason: SurfaceReconfigureReason::Lost,
                });
            }
            wgpu::CurrentSurfaceTexture::Timeout => {
                return Ok(SurfaceAcquireOutcome::Retryable {
                    session,
                    reason: SurfaceRetryReason::Timeout,
                });
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(SurfaceAcquireOutcome::Retryable {
                    session,
                    reason: SurfaceRetryReason::Occluded,
                });
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RhiError::SurfaceUnavailable(
                    "native surface acquisition validation failed".to_string(),
                ));
            }
        };
        let frame = self.handles.allocate_frame()?;
        let texture_desc = TextureDesc::new(
            "zircon-surface-frame-target",
            surface.swapchain.width,
            surface.swapchain.height,
            surface.swapchain.format,
            TextureUsage::PRESENT | TextureUsage::RENDER_ATTACHMENT,
        );
        let default_native_view = acquired
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let (target, default_view) = match registry.register_surface_frame(
            texture_desc.clone(),
            acquired.texture.clone(),
            default_native_view,
        ) {
            Ok(handles) => handles,
            Err(error) => {
                let _ = self.handles.release_frame(frame);
                return Err(error);
            }
        };
        self.frames.insert(
            frame,
            WgpuSurfaceFrame {
                session,
                target,
                default_view,
                desc: texture_desc.clone(),
                native: acquired,
            },
        );
        Ok(SurfaceAcquireOutcome::Acquired(SurfaceFrameLease::new(
            frame,
            session,
            target,
            default_view,
            texture_desc,
        )))
    }

    pub(crate) fn present_frame(
        &mut self,
        registry: &mut WgpuResourceRegistry,
        frame: &SurfaceFrameLease,
        submission: zr_rhi::SubmissionTicket,
    ) -> Result<SurfacePresentReceipt, RhiError> {
        self.validate_frame_lease(frame)?;
        let frame_id = frame.frame();
        let frame_resources = self
            .frames
            .get(&frame_id)
            .map(|resources| (resources.target, resources.default_view))
            .ok_or_else(|| RhiError::SurfaceUnavailable("surface frame is not live".to_string()))?;
        if !registry.surface_frame_has_submission(frame_resources.0, submission)? {
            return Err(RhiError::SurfaceFrameSubmissionMissingTarget {
                frame: frame_id,
                submission,
            });
        }
        registry.release_surface_frame(frame_resources.0, frame_resources.1)?;
        let frame_resources = self
            .frames
            .remove(&frame_id)
            .ok_or_else(|| RhiError::SurfaceUnavailable("surface frame is not live".to_string()))?;
        frame_resources.native.present();
        self.handles.release_frame(frame_id)?;
        self.terminal_frames
            .record(frame_id, SurfaceFrameTerminal::Presented);
        Ok(SurfacePresentReceipt {
            frame: frame_id,
            submission,
            terminal: SurfaceFrameTerminal::Presented,
        })
    }

    pub(crate) fn discard_frame(
        &mut self,
        registry: &mut WgpuResourceRegistry,
        frame: &SurfaceFrameLease,
    ) -> Result<(), RhiError> {
        self.validate_frame_lease(frame)?;
        self.discard_active_frame(registry, frame.frame())
    }

    fn discard_active_frame(
        &mut self,
        registry: &mut WgpuResourceRegistry,
        frame: SurfaceFrameId,
    ) -> Result<(), RhiError> {
        self.ensure_active_frame(frame)?;
        let frame_resources = self
            .frames
            .get(&frame)
            .map(|resources| (resources.target, resources.default_view))
            .ok_or_else(|| RhiError::SurfaceUnavailable("surface frame is not live".to_string()))?;
        registry.release_surface_frame(frame_resources.0, frame_resources.1)?;
        self.frames
            .remove(&frame)
            .ok_or_else(|| RhiError::SurfaceUnavailable("surface frame is not live".to_string()))?;
        self.handles.release_frame(frame)?;
        self.terminal_frames
            .record(frame, SurfaceFrameTerminal::Discarded);
        Ok(())
    }

    pub(crate) fn frame_submission_tickets(
        &self,
        registry: &WgpuResourceRegistry,
        frame: SurfaceFrameId,
    ) -> Result<Vec<zr_rhi::SubmissionTicket>, RhiError> {
        self.ensure_active_frame(frame)?;
        let target = self
            .frames
            .get(&frame)
            .map(|resources| resources.target)
            .ok_or_else(|| RhiError::SurfaceUnavailable("surface frame is not live".to_string()))?;
        registry.surface_frame_submission_tickets(target)
    }

    pub(crate) fn validate_frame_lease(
        &self,
        frame: &SurfaceFrameLease,
    ) -> Result<(TextureHandle, TextureViewHandle), RhiError> {
        self.ensure_active_frame(frame.frame())?;
        let active = self
            .frames
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
        Ok((active.target, active.default_view))
    }

    pub(crate) fn session_submission_tickets(
        &self,
        registry: &WgpuResourceRegistry,
        session: SurfaceSession,
    ) -> Result<Vec<zr_rhi::SubmissionTicket>, RhiError> {
        self.handles.validate_session(session)?;
        let mut tickets = HashSet::<SubmissionTicket>::new();
        for target in self
            .frames
            .values()
            .filter_map(|frame| (frame.session == session).then_some(frame.target))
        {
            tickets.extend(registry.surface_frame_submission_tickets(target)?);
        }
        let mut tickets: Vec<_> = tickets.into_iter().collect();
        tickets.sort_by_key(|ticket| ticket.sequence());
        Ok(tickets)
    }

    pub(crate) fn destroy_session(
        &mut self,
        registry: &mut WgpuResourceRegistry,
        session: SurfaceSession,
    ) -> Result<(), RhiError> {
        self.handles.validate_session(session)?;
        self.discard_session_frames(registry, session)?;
        self.sessions.remove(&session).ok_or_else(|| {
            RhiError::SurfaceUnavailable("surface session is not live".to_string())
        })?;
        self.handles.release_session(session)?;
        Ok(())
    }

    pub(crate) fn terminalize_all(
        &mut self,
        registry: &mut WgpuResourceRegistry,
    ) -> Result<(), RhiError> {
        let frames: Vec<_> = self.frames.keys().copied().collect();
        for frame in frames {
            self.discard_active_frame(registry, frame)?;
        }
        Ok(())
    }

    fn discard_session_frames(
        &mut self,
        registry: &mut WgpuResourceRegistry,
        session: SurfaceSession,
    ) -> Result<(), RhiError> {
        let frames: Vec<_> = self
            .frames
            .iter()
            .filter_map(|(frame, resources)| (resources.session == session).then_some(*frame))
            .collect();
        for frame in frames {
            self.discard_active_frame(registry, frame)?;
        }
        Ok(())
    }

    fn ensure_active_frame(&self, frame: SurfaceFrameId) -> Result<(), RhiError> {
        if let Some(terminal) = self.terminal_frames.terminal(frame) {
            return Err(RhiError::SurfaceFrameAlreadyTerminal { frame, terminal });
        }
        Ok(self.handles.validate_frame(frame)?)
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

fn negotiate_surface_config(
    surface: &wgpu::Surface<'static>,
    adapter: &wgpu::Adapter,
    descriptor: &RenderSurfaceDescriptor,
) -> Result<(wgpu::SurfaceConfiguration, SwapchainDesc), RhiError> {
    let caps = surface.get_capabilities(adapter);
    let format =
        choose_surface_format(&caps.formats, descriptor.swapchain.format).ok_or_else(|| {
            RhiError::SurfaceUnavailable("surface has no supported SDR sRGB format".to_string())
        })?;
    let present_mode = choose_present_mode(&caps.present_modes, descriptor.swapchain.present_mode)
        .ok_or_else(|| {
            RhiError::SurfaceUnavailable("surface has no compatible present mode".to_string())
        })?;
    let receipt = SwapchainDesc {
        width: descriptor.swapchain.width,
        height: descriptor.swapchain.height,
        present_mode: surface_present_mode(present_mode),
        format: surface_texture_format(format).ok_or_else(|| {
            RhiError::SurfaceUnavailable(
                "surface format is not represented by the neutral SDR contract".to_string(),
            )
        })?,
    };
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: receipt.width,
        height: receipt.height,
        present_mode,
        desired_maximum_frame_latency: 2,
        alpha_mode: caps
            .alpha_modes
            .first()
            .copied()
            .unwrap_or(wgpu::CompositeAlphaMode::Auto),
        view_formats: Vec::new(),
    };
    Ok((config, receipt))
}

pub(crate) fn surface_descriptor_is_supported(
    capabilities: &wgpu::SurfaceCapabilities,
    descriptor: &RenderSurfaceDescriptor,
) -> bool {
    choose_surface_format(&capabilities.formats, descriptor.swapchain.format).is_some()
        && choose_present_mode(
            &capabilities.present_modes,
            descriptor.swapchain.present_mode,
        )
        .is_some()
}

fn choose_surface_format(
    formats: &[wgpu::TextureFormat],
    requested: TextureFormat,
) -> Option<wgpu::TextureFormat> {
    let requested = match requested {
        TextureFormat::Bgra8UnormSrgb => Some(wgpu::TextureFormat::Bgra8UnormSrgb),
        TextureFormat::Rgba8UnormSrgb => Some(wgpu::TextureFormat::Rgba8UnormSrgb),
        _ => None,
    };
    requested
        .filter(|format| formats.contains(format))
        .or_else(|| {
            [
                wgpu::TextureFormat::Bgra8UnormSrgb,
                wgpu::TextureFormat::Rgba8UnormSrgb,
            ]
            .into_iter()
            .find(|format| formats.contains(format))
        })
}

fn surface_texture_format(format: wgpu::TextureFormat) -> Option<TextureFormat> {
    match format {
        wgpu::TextureFormat::Bgra8UnormSrgb => Some(TextureFormat::Bgra8UnormSrgb),
        wgpu::TextureFormat::Rgba8UnormSrgb => Some(TextureFormat::Rgba8UnormSrgb),
        _ => None,
    }
}

fn choose_present_mode(
    present_modes: &[wgpu::PresentMode],
    requested: PresentMode,
) -> Option<wgpu::PresentMode> {
    let requested = wgpu_present_mode(requested);
    present_modes
        .contains(&requested)
        .then_some(requested)
        .or_else(|| {
            present_modes
                .contains(&wgpu::PresentMode::Fifo)
                .then_some(wgpu::PresentMode::Fifo)
        })
        .or_else(|| {
            present_modes
                .contains(&wgpu::PresentMode::Immediate)
                .then_some(wgpu::PresentMode::Immediate)
        })
        .or_else(|| {
            present_modes
                .contains(&wgpu::PresentMode::Mailbox)
                .then_some(wgpu::PresentMode::Mailbox)
        })
}

const fn wgpu_present_mode(mode: PresentMode) -> wgpu::PresentMode {
    match mode {
        PresentMode::Immediate => wgpu::PresentMode::Immediate,
        PresentMode::Fifo => wgpu::PresentMode::Fifo,
        PresentMode::Mailbox => wgpu::PresentMode::Mailbox,
    }
}

const fn surface_present_mode(mode: wgpu::PresentMode) -> PresentMode {
    match mode {
        wgpu::PresentMode::Immediate => PresentMode::Immediate,
        wgpu::PresentMode::Mailbox => PresentMode::Mailbox,
        wgpu::PresentMode::Fifo
        | wgpu::PresentMode::FifoRelaxed
        | wgpu::PresentMode::AutoVsync
        | wgpu::PresentMode::AutoNoVsync => PresentMode::Fifo,
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn create_native_surface(
    instance: &wgpu::Instance,
    target: RenderNativeSurfaceTarget,
) -> Result<wgpu::Surface<'static>, RhiError> {
    match target {
        RenderNativeSurfaceTarget::Win32 { hwnd, hinstance } => {
            let hwnd = required_nonzero_isize(hwnd, "invalid Win32 hwnd")?;
            let mut window = wgpu::rwh::Win32WindowHandle::new(hwnd);
            window.hinstance = optional_nonzero_isize(hinstance)?;
            let raw_window_handle = wgpu::rwh::RawWindowHandle::Win32(window);
            let raw_display_handle =
                wgpu::rwh::RawDisplayHandle::Windows(wgpu::rwh::WindowsDisplayHandle::new());
            let target = wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(raw_display_handle),
                raw_window_handle,
            };
            // The application owns the native window and must destroy the
            // session before the window handle becomes invalid.
            unsafe { instance.create_surface_unsafe(target) }
                .map_err(|error| RhiError::SurfaceUnavailable(error.to_string()))
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn create_native_surface(
    _instance: &wgpu::Instance,
    target: RenderNativeSurfaceTarget,
) -> Result<wgpu::Surface<'static>, RhiError> {
    match target {
        RenderNativeSurfaceTarget::Win32 { .. } => Err(RhiError::SurfaceUnavailable(
            "Win32 surface sessions are only supported on Windows".to_string(),
        )),
    }
}

#[cfg(target_os = "windows")]
fn required_nonzero_isize(value: u64, reason: &'static str) -> Result<NonZeroIsize, RhiError> {
    if value == 0 || value > isize::MAX as u64 {
        return Err(RhiError::SurfaceUnavailable(reason.to_string()));
    }
    Ok(NonZeroIsize::new(value as isize).expect("range check excludes zero"))
}

#[cfg(target_os = "windows")]
fn optional_nonzero_isize(value: Option<u64>) -> Result<Option<NonZeroIsize>, RhiError> {
    value
        .map(|value| required_nonzero_isize(value, "invalid Win32 hinstance"))
        .transpose()
}

#[cfg(test)]
mod tests {
    use zr_rhi::PresentMode;

    use super::surface_present_mode;

    #[test]
    fn native_fifo_variants_project_to_the_neutral_fifo_contract() {
        for mode in [
            wgpu::PresentMode::Fifo,
            wgpu::PresentMode::FifoRelaxed,
            wgpu::PresentMode::AutoVsync,
            wgpu::PresentMode::AutoNoVsync,
        ] {
            assert_eq!(surface_present_mode(mode), PresentMode::Fifo);
        }
    }
}
