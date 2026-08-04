use std::sync::Arc;

use crate::core::jobs::{
    CancellationToken, EditorJobSpec, EditorJobSystem, JobCategory, JobTicket,
};
use crate::scene::viewport::{RenderFramework, RenderFrameworkError};
use crate::ui::retained_host::host_contract::WorldSpaceUiSurfaceSubmission;
use zircon_runtime::core::CoreHandle;
use zircon_runtime::core::manager::{ManagerServiceHandle, resolve_manager_service};

use super::active_viewport::ActiveViewport;
use super::render_framework_resolve_job::RenderFrameworkResolveJob;

pub(super) struct ViewportState {
    pub(super) render_framework: Option<ManagerServiceHandle<dyn RenderFramework>>,
    render_framework_core: Option<CoreHandle>,
    #[cfg(test)]
    test_render_framework: Option<Arc<dyn RenderFramework>>,
    pub(super) jobs: Option<EditorJobSystem>,
    pub(super) render_framework_cancel: Option<CancellationToken>,
    pub(super) render_framework_task: Option<JobTicket<ManagerServiceHandle<dyn RenderFramework>>>,
    pub(super) viewport: Option<ActiveViewport>,
    pub(super) latest_generation: Option<u64>,
    pub(super) last_error: Option<String>,
    pub(super) last_world_space_ui_surfaces: Vec<WorldSpaceUiSurfaceSubmission>,
    pub(super) world_space_ui_pointer_capture: Option<WorldSpaceUiSurfaceSubmission>,
}

impl ViewportState {
    pub(super) fn lazy(core: CoreHandle) -> Self {
        Self::new(Some(core))
    }

    #[cfg(test)]
    pub(super) fn with_render_framework(render_framework: Arc<dyn RenderFramework>) -> Self {
        let mut state = Self::new(None);
        state.test_render_framework = Some(render_framework);
        state
    }

    pub(super) fn bind_jobs(&mut self, jobs: EditorJobSystem) {
        self.jobs = Some(jobs);
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
        if self.render_framework.is_some() {
            return self.resolve_stored_render_framework();
        }
        #[cfg(test)]
        if self.test_render_framework.is_some() {
            return self.resolve_stored_render_framework();
        }

        if self.render_framework_task.is_some() {
            let result = self
                .render_framework_task
                .as_ref()
                .and_then(JobTicket::try_take);
            match result {
                None => return Ok(None),
                Some(Ok(render_framework)) => {
                    self.render_framework = Some(render_framework);
                    self.render_framework_cancel = None;
                    self.render_framework_task = None;
                    return self.resolve_stored_render_framework();
                }
                Some(Err(error)) => {
                    self.render_framework_cancel = None;
                    self.render_framework_task = None;
                    return Err(RenderFrameworkError::Backend(error.to_string()));
                }
            }
        }

        let Some(core) = self.render_framework_core.clone() else {
            return Err(RenderFrameworkError::Backend(
                "render framework was not configured for the editor viewport".to_string(),
            ));
        };

        let Some(jobs) = self.jobs.as_ref() else {
            return Err(RenderFrameworkError::Backend(
                "editor jobs were not configured for viewport lazy resolve".to_string(),
            ));
        };
        zircon_runtime::profile_scope!("editor", "viewport", "submit_render_framework_resolve");
        let cancel = CancellationToken::default();
        self.render_framework_task = Some(
            jobs.submit(
                EditorJobSpec::new("Resolve editor render framework", JobCategory::Misc)
                    .with_cancel(cancel.clone()),
                RenderFrameworkResolveJob::new(core),
            )
            .map_err(|error| {
                RenderFrameworkError::Backend(format!(
                    "failed to submit editor viewport render framework resolver: {error}"
                ))
            })?,
        );
        self.render_framework_cancel = Some(cancel);
        Ok(None)
    }

    pub(super) fn resolve_stored_render_framework(
        &self,
    ) -> Result<Option<Arc<dyn RenderFramework>>, RenderFrameworkError> {
        #[cfg(test)]
        if let Some(render_framework) = &self.test_render_framework {
            return Ok(Some(Arc::clone(render_framework)));
        }

        let Some(handle) = &self.render_framework else {
            return Ok(None);
        };
        let Some(core) = &self.render_framework_core else {
            return Err(RenderFrameworkError::Backend(
                "render framework handle has no owning runtime core".to_string(),
            ));
        };
        resolve_manager_service(core, handle.clone())
            .map(Some)
            .map_err(|error| RenderFrameworkError::Backend(error.to_string()))
    }

    pub(super) fn new(render_framework_core: Option<CoreHandle>) -> Self {
        Self {
            render_framework: None,
            render_framework_core,
            #[cfg(test)]
            test_render_framework: None,
            jobs: None,
            render_framework_cancel: None,
            render_framework_task: None,
            viewport: None,
            latest_generation: None,
            last_error: None,
            last_world_space_ui_surfaces: Vec::new(),
            world_space_ui_pointer_capture: None,
        }
    }
}
