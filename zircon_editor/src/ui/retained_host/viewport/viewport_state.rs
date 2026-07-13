use std::sync::Arc;

use crate::core::jobs::{
    CancellationToken, EditorJobSpec, EditorJobSystem, JobCategory, JobTicket,
};
use crate::scene::viewport::{RenderFramework, RenderFrameworkError};
use crate::ui::retained_host::host_contract::WorldSpaceUiSurfaceSubmission;
use crate::ui::retained_host::primitives::Image;
use zircon_runtime::core::CoreHandle;

use super::active_viewport::ActiveViewport;
use super::render_framework_resolve_job::RenderFrameworkResolveJob;

pub(super) struct ViewportState {
    pub(super) render_framework: Option<Arc<dyn RenderFramework>>,
    render_framework_core: Option<CoreHandle>,
    pub(super) jobs: Option<EditorJobSystem>,
    pub(super) render_framework_cancel: Option<CancellationToken>,
    pub(super) render_framework_task: Option<JobTicket<Arc<dyn RenderFramework>>>,
    pub(super) viewport: Option<ActiveViewport>,
    pub(super) latest_generation: Option<u64>,
    pub(super) latest_image: Option<Image>,
    pub(super) last_error: Option<String>,
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
        if let Some(render_framework) = &self.render_framework {
            return Ok(Some(render_framework.clone()));
        }

        if self.render_framework_task.is_some() {
            let result = self
                .render_framework_task
                .as_ref()
                .and_then(JobTicket::try_take);
            match result {
                None => return Ok(None),
                Some(Ok(render_framework)) => {
                    self.render_framework = Some(render_framework.clone());
                    self.render_framework_cancel = None;
                    self.render_framework_task = None;
                    return Ok(Some(render_framework));
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

    pub(super) fn new(
        render_framework: Option<Arc<dyn RenderFramework>>,
        render_framework_core: Option<CoreHandle>,
    ) -> Self {
        Self {
            render_framework,
            render_framework_core,
            jobs: None,
            render_framework_cancel: None,
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
