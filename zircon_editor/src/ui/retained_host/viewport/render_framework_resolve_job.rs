use crate::core::jobs::{EditorJob, JobContext, JobError};
use crate::scene::viewport::RenderFramework;
use zircon_runtime::core::manager::ManagerServiceHandle;

use super::render_framework_access::ViewportRenderFrameworkAccess;

pub(super) struct RenderFrameworkResolveJob {
    render_framework_access: ViewportRenderFrameworkAccess,
}

impl RenderFrameworkResolveJob {
    pub(super) fn new(render_framework_access: ViewportRenderFrameworkAccess) -> Self {
        Self {
            render_framework_access,
        }
    }
}

impl EditorJob for RenderFrameworkResolveJob {
    type Output = ManagerServiceHandle<dyn RenderFramework>;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        zircon_runtime::profile_scope!("editor", "viewport", "async_resolve_render_framework");
        self.render_framework_access
            .render_framework_handle()
            .map_err(JobError::failed)
    }
}
