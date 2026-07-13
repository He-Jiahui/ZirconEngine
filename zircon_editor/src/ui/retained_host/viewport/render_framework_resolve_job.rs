use std::sync::Arc;

use crate::core::jobs::{EditorJob, JobContext, JobError};
use crate::scene::viewport::RenderFramework;
use zircon_runtime::core::manager::resolve_render_framework;
use zircon_runtime::core::CoreHandle;

pub(super) struct RenderFrameworkResolveJob {
    core: CoreHandle,
}

impl RenderFrameworkResolveJob {
    pub(super) fn new(core: CoreHandle) -> Self {
        Self { core }
    }
}

impl EditorJob for RenderFrameworkResolveJob {
    type Output = Arc<dyn RenderFramework>;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        zircon_runtime::profile_scope!("editor", "viewport", "async_resolve_render_framework");
        resolve_render_framework(&self.core).map_err(JobError::failed)
    }
}
