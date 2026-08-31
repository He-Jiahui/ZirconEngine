use std::sync::Arc;

use crate::core::framework::render::{
    CapturedFrame, RenderFrameExtract, RenderFramework, RenderFrameworkError, RenderProfileBundle,
    RenderSubmissionConfig, RenderViewportDescriptor, RenderViewportHandle,
    RenderViewportPickDisposition, RenderViewportPickPolicy, RenderViewportPickPurpose,
    RenderViewportPickRequest, RenderViewportPickResult, RenderViewportPickTicket,
    RenderViewportSurfaceDescriptor, UiRenderSubmission, RENDER_PROFILE_CONFIG_KEY,
};
use crate::core::manager::{
    render_framework_handle, resolve_manager_service, ManagerServiceHandle,
};
use crate::core::math::UVec2;
use crate::core::{CoreError, CoreHandle};
use zircon_runtime_interface::{ZrRuntimeViewportPickPurposeV1, ZrRuntimeViewportPickRequestV1};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveViewport {
    handle: RenderViewportHandle,
    size: UVec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum RuntimeViewportPickAdmission {
    Backend {
        request: RenderViewportPickRequest,
        ticket: RenderViewportPickTicket,
    },
    Terminal(RenderViewportPickDisposition),
}

pub(super) struct RuntimeRenderBridge {
    core: CoreHandle,
    render_framework: ManagerServiceHandle<dyn RenderFramework>,
    viewport: Option<ActiveViewport>,
    last_generation: Option<u64>,
}

impl RuntimeRenderBridge {
    pub(super) fn new(core: &CoreHandle) -> Result<Self, CoreError> {
        let render_framework = {
            crate::profile_scope!("runtime", "render_bridge", "resolve_render_framework");
            render_framework_handle(core)?
        };
        let submission_config = match core.load_config_value(RENDER_PROFILE_CONFIG_KEY) {
            Some(_) => core
                .load_config::<RenderProfileBundle>(RENDER_PROFILE_CONFIG_KEY)?
                .submission_config(),
            None => Default::default(),
        };
        if submission_config != RenderSubmissionConfig::default() {
            resolve_manager_service(core, render_framework.clone())?
                .set_submission_config(submission_config)
                .map_err(|error| {
                    CoreError::Initialization(
                        "render submission configuration".to_string(),
                        error.to_string(),
                    )
                })?;
        }
        Ok(Self {
            core: core.clone(),
            render_framework,
            viewport: None,
            last_generation: None,
        })
    }

    pub(super) fn submit_extract(
        &mut self,
        mut extract: RenderFrameExtract,
        size: UVec2,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        self.submit_extract_with_ui(extract, size, None)
    }

    pub(super) fn submit_extract_with_ui(
        &mut self,
        mut extract: RenderFrameExtract,
        size: UVec2,
        ui: Option<Arc<UiRenderSubmission>>,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        crate::profile_scope!("runtime", "frame", "runtime_frame_submit");
        crate::profile_scope!("runtime", "render_bridge", "submit_extract");
        let render_framework = self.resolve_render_framework()?;
        let viewport = self.ensure_viewport(size, render_framework.as_ref())?;
        extract.apply_viewport_size(size);

        let pipelined_before_submit = render_framework.submission_config().pipelined_render;
        let completed_frame = if pipelined_before_submit {
            // Read frame N before queueing N+1 so capture does not collapse the
            // intentional one-frame render/simulation overlap.
            self.capture_frame_if_newer(render_framework.as_ref(), viewport)?
        } else {
            None
        };

        render_framework.submit_frame_extract_with_ui(viewport, extract, ui)?;
        if pipelined_before_submit && render_framework.submission_config().pipelined_render {
            return Ok(completed_frame);
        }
        self.capture_frame_if_newer(render_framework.as_ref(), viewport)
    }

    fn capture_frame_if_newer(
        &mut self,
        render_framework: &dyn RenderFramework,
        viewport: RenderViewportHandle,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        let Some(frame) =
            render_framework.poll_captured_frame_if_newer(viewport, self.last_generation)?
        else {
            return Ok(None);
        };
        self.last_generation = Some(frame.generation);
        Ok(Some(frame))
    }

    pub(super) fn bind_surface(
        &mut self,
        descriptor: RenderViewportSurfaceDescriptor,
    ) -> Result<(), RenderFrameworkError> {
        crate::profile_scope!("runtime", "render_bridge", "bind_surface");
        let render_framework = self.resolve_render_framework()?;
        let size = UVec2::new(descriptor.size.x.max(1), descriptor.size.y.max(1));
        let descriptor = RenderViewportSurfaceDescriptor::new(size, descriptor.target);
        let viewport = self.ensure_viewport_for_surface_rebind(size, render_framework.as_ref())?;
        render_framework.bind_viewport_surface(viewport, descriptor)?;
        self.viewport = Some(ActiveViewport {
            handle: viewport,
            size,
        });
        self.last_generation = None;
        Ok(())
    }

    pub(super) fn unbind_surface(&mut self) -> Result<(), RenderFrameworkError> {
        crate::profile_scope!("runtime", "render_bridge", "unbind_surface");
        let Some(viewport) = self.viewport else {
            return Ok(());
        };
        self.resolve_render_framework()?
            .unbind_viewport_surface(viewport.handle)
    }

    pub(super) fn present_extract(
        &mut self,
        mut extract: RenderFrameExtract,
        size: UVec2,
    ) -> Result<(), RenderFrameworkError> {
        self.present_extract_with_ui(extract, size, None)
    }

    pub(super) fn present_extract_with_ui(
        &mut self,
        mut extract: RenderFrameExtract,
        size: UVec2,
        ui: Option<Arc<UiRenderSubmission>>,
    ) -> Result<(), RenderFrameworkError> {
        crate::profile_scope!("runtime", "frame", "runtime_frame_submit");
        crate::profile_scope!("runtime", "render_bridge", "present_extract");
        let render_framework = self.resolve_render_framework()?;
        let viewport = self.ensure_viewport(size, render_framework.as_ref())?;
        extract.apply_viewport_size(size);
        render_framework.present_frame_extract_with_ui(viewport, extract, ui)
    }

    pub(super) fn request_viewport_pick(
        &self,
        request: ZrRuntimeViewportPickRequestV1,
    ) -> Result<RuntimeViewportPickAdmission, RenderFrameworkError> {
        let Some(viewport) = self.viewport else {
            return Ok(RuntimeViewportPickAdmission::Terminal(
                RenderViewportPickDisposition::Unavailable,
            ));
        };
        if viewport.size.x != request.viewport_size.width
            || viewport.size.y != request.viewport_size.height
        {
            return Ok(RuntimeViewportPickAdmission::Terminal(
                RenderViewportPickDisposition::StaleFrame,
            ));
        }

        let purpose = match request.purpose() {
            Some(ZrRuntimeViewportPickPurposeV1::Hover) => RenderViewportPickPurpose::Hover,
            Some(ZrRuntimeViewportPickPurposeV1::Press) => RenderViewportPickPurpose::Press,
            Some(ZrRuntimeViewportPickPurposeV1::Selection) => RenderViewportPickPurpose::Selection,
            None => {
                return Ok(RuntimeViewportPickAdmission::Terminal(
                    RenderViewportPickDisposition::Rejected,
                ));
            }
        };
        let Some(policy) = RenderViewportPickPolicy::from_bits(request.policy_flags) else {
            return Ok(RuntimeViewportPickAdmission::Terminal(
                RenderViewportPickDisposition::Rejected,
            ));
        };
        let backend_request = RenderViewportPickRequest::new(
            viewport.handle,
            viewport.size,
            UVec2::new(request.pixel.x, request.pixel.y),
            request.frame_generation,
            request.input_sequence,
            purpose,
            policy,
        );
        let render_framework = self.resolve_render_framework()?;
        match render_framework.request_viewport_pick(backend_request) {
            Ok(ticket) if ticket.is_valid() => Ok(RuntimeViewportPickAdmission::Backend {
                request: backend_request,
                ticket,
            }),
            Ok(_) => Ok(RuntimeViewportPickAdmission::Terminal(
                RenderViewportPickDisposition::Rejected,
            )),
            Err(RenderFrameworkError::UnsupportedCapability { .. }) => Ok(
                RuntimeViewportPickAdmission::Terminal(RenderViewportPickDisposition::Unavailable),
            ),
            Err(error) => Err(error),
        }
    }

    pub(super) fn poll_viewport_pick(
        &self,
        ticket: RenderViewportPickTicket,
    ) -> Result<Option<RenderViewportPickResult>, RenderFrameworkError> {
        self.resolve_render_framework()?.poll_viewport_pick(ticket)
    }

    pub(super) fn cancel_viewport_pick(
        &self,
        ticket: RenderViewportPickTicket,
    ) -> Result<(), RenderFrameworkError> {
        self.resolve_render_framework()?
            .cancel_viewport_pick(ticket)
    }

    fn ensure_viewport(
        &mut self,
        size: UVec2,
        render_framework: &dyn RenderFramework,
    ) -> Result<RenderViewportHandle, RenderFrameworkError> {
        let size = UVec2::new(size.x.max(1), size.y.max(1));
        if let Some(viewport) = self.viewport {
            if viewport.size == size {
                return Ok(viewport.handle);
            }
            render_framework.destroy_viewport(viewport.handle)?;
            self.viewport = None;
            self.last_generation = None;
        }

        let descriptor = RenderViewportDescriptor::new(size)
            .with_label("runtime.viewport")
            .with_hit_proxies();
        let handle = render_framework.create_viewport(descriptor)?;
        self.viewport = Some(ActiveViewport { handle, size });
        Ok(handle)
    }

    fn ensure_viewport_for_surface_rebind(
        &mut self,
        size: UVec2,
        render_framework: &dyn RenderFramework,
    ) -> Result<RenderViewportHandle, RenderFrameworkError> {
        if let Some(viewport) = self.viewport {
            return Ok(viewport.handle);
        }

        let descriptor = RenderViewportDescriptor::new(size)
            .with_label("runtime.viewport")
            .with_hit_proxies();
        let handle = render_framework.create_viewport(descriptor)?;
        self.viewport = Some(ActiveViewport { handle, size });
        Ok(handle)
    }

    fn resolve_render_framework(&self) -> Result<Arc<dyn RenderFramework>, RenderFrameworkError> {
        resolve_manager_service(&self.core, self.render_framework.clone())
            .map_err(|error| RenderFrameworkError::Backend(error.to_string()))
    }
}

impl Drop for RuntimeRenderBridge {
    fn drop(&mut self) {
        if let Some(viewport) = self.viewport {
            if let Ok(render_framework) = self.resolve_render_framework() {
                let _ = render_framework.destroy_viewport(viewport.handle);
            }
        }
    }
}
