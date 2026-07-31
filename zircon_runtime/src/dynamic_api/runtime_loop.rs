use std::sync::Arc;

use crate::core::framework::render::{
    CapturedFrame, RenderFrameExtract, RenderFramework, RenderFrameworkError, RenderProfileBundle,
    RenderViewportDescriptor, RenderViewportHandle, RenderViewportSurfaceDescriptor,
    RENDER_PROFILE_CONFIG_KEY,
};
use crate::core::manager::{
    render_framework_handle, resolve_manager_service, ManagerServiceHandle,
};
use crate::core::math::UVec2;
use crate::core::{CoreError, CoreHandle};
use zircon_runtime_interface::ui::surface::UiRenderExtract;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveViewport {
    handle: RenderViewportHandle,
    size: UVec2,
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
        if submission_config.pipelined_render {
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
        ui: Option<UiRenderExtract>,
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
            render_framework.capture_frame_if_newer(viewport, self.last_generation)?
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
        let viewport = self.ensure_viewport(descriptor.size, render_framework.as_ref())?;
        render_framework.bind_viewport_surface(viewport, descriptor)
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
        ui: Option<UiRenderExtract>,
    ) -> Result<(), RenderFrameworkError> {
        crate::profile_scope!("runtime", "frame", "runtime_frame_submit");
        crate::profile_scope!("runtime", "render_bridge", "present_extract");
        let render_framework = self.resolve_render_framework()?;
        let viewport = self.ensure_viewport(size, render_framework.as_ref())?;
        extract.apply_viewport_size(size);
        render_framework.present_frame_extract_with_ui(viewport, extract, ui)
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

        let descriptor = RenderViewportDescriptor::new(size).with_label("runtime.viewport");
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
