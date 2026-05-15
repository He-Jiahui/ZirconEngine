use std::sync::Arc;
use std::thread::JoinHandle;

use crate::scene::viewport::{RenderFramework, RenderFrameworkError};
use crate::ui::retained_host::host_contract::WorldSpaceUiSurfaceSubmission;
use crate::ui::retained_host::primitives::Image;
use zircon_runtime::core::manager::resolve_render_framework;
use zircon_runtime::core::CoreHandle;

use super::active_viewport::ActiveViewport;

pub(super) struct ViewportState {
    pub(super) render_framework: Option<Arc<dyn RenderFramework>>,
    render_framework_core: Option<CoreHandle>,
    render_framework_task: Option<JoinHandle<Result<Arc<dyn RenderFramework>, String>>>,
    pub(super) viewport: Option<ActiveViewport>,
    pub(super) latest_generation: Option<u64>,
    pub(super) latest_image: Option<Image>,
    pub(super) last_error: Option<String>,
    #[allow(dead_code)]
    pub(super) last_world_space_ui_surfaces: Vec<WorldSpaceUiSurfaceSubmission>,
    pub(super) world_space_ui_pointer_capture: Option<WorldSpaceUiSurfaceSubmission>,
}

impl ViewportState {
    pub(super) fn lazy(core: CoreHandle) -> Self {
        Self::new(None, Some(core))
    }

    #[cfg(test)]
    pub(super) fn with_render_framework(render_framework: Arc<dyn RenderFramework>) -> Self {
        Self::new(Some(render_framework), None)
    }

    pub(super) fn render_framework(
        &mut self,
    ) -> Result<Arc<dyn RenderFramework>, RenderFrameworkError> {
        self.poll_or_start_render_framework()?.ok_or_else(|| {
            RenderFrameworkError::Backend("render framework is still starting".into())
        })
    }

    pub(super) fn poll_or_start_render_framework(
        &mut self,
    ) -> Result<Option<Arc<dyn RenderFramework>>, RenderFrameworkError> {
        if let Some(render_framework) = &self.render_framework {
            return Ok(Some(render_framework.clone()));
        }

        if let Some(task) = &self.render_framework_task {
            if !task.is_finished() {
                return Ok(None);
            }
        }

        if let Some(task) = self.render_framework_task.take() {
            let render_framework = match task.join() {
                Ok(Ok(render_framework)) => render_framework,
                Ok(Err(error)) => return Err(RenderFrameworkError::Backend(error)),
                Err(_) => {
                    return Err(RenderFrameworkError::Backend(
                        "editor viewport render framework resolver panicked".to_string(),
                    ))
                }
            };
            self.render_framework = Some(render_framework.clone());
            return Ok(Some(render_framework));
        }

        let Some(core) = self.render_framework_core.clone() else {
            return Err(RenderFrameworkError::Backend(
                "render framework was not configured for the editor viewport".to_string(),
            ));
        };

        zircon_runtime::profile_scope!(
            "editor",
            "viewport",
            "start_async_render_framework_resolve"
        );
        self.render_framework_task = Some(
            std::thread::Builder::new()
                .name("zircon-editor-render-framework-resolve".to_string())
                .spawn(move || {
                    zircon_runtime::profile_scope!(
                        "editor",
                        "viewport",
                        "async_resolve_render_framework"
                    );
                    resolve_render_framework(&core).map_err(|error| {
                        format!("failed to resolve editor viewport render framework: {error}")
                    })
                })
                .map_err(|error| {
                    RenderFrameworkError::Backend(format!(
                        "failed to start editor viewport render framework resolver: {error}"
                    ))
                })?,
        );
        Ok(None)
    }

    fn new(
        render_framework: Option<Arc<dyn RenderFramework>>,
        render_framework_core: Option<CoreHandle>,
    ) -> Self {
        Self {
            render_framework,
            render_framework_core,
            render_framework_task: None,
            viewport: None,
            latest_generation: None,
            latest_image: None,
            last_error: None,
            last_world_space_ui_surfaces: Vec::new(),
            world_space_ui_pointer_capture: None,
        }
    }
}
