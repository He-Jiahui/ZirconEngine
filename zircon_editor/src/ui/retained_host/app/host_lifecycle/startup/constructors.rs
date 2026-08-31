use super::*;
use crate::core::gui_startup_request::EditorGuiStartupRequest;
use zircon_runtime_interface::hub_protocol::HubSessionToken;
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn new(
        core: CoreHandle,
        runtime_gateway: SharedEditorRuntimeGateway,
        ui: UiHostWindow,
        startup_request: Option<EditorGuiStartupRequest>,
        project_runtime_build_set: Option<ZrRuntimeBuildSetId>,
        hub_launch_session: Option<HubSessionToken>,
    ) -> Result<Self, Box<dyn Error>> {
        zircon_runtime::profile_scope!("editor", "retained_host", "new");
        let runtime_lease = RetainedHostRuntimeLease::new(core);
        let viewport = {
            zircon_runtime::profile_scope!("editor", "retained_host", "new_viewport_controller");
            RetainedViewportController::new(runtime_lease.viewport_render_framework_access())
        };
        Self::new_with_viewport(
            runtime_lease,
            runtime_gateway,
            ui,
            viewport,
            startup_request,
            project_runtime_build_set,
            hub_launch_session,
        )
    }

    #[cfg(test)]
    pub(in crate::ui::retained_host::app) fn new_for_test(
        core: CoreHandle,
        ui: UiHostWindow,
    ) -> Result<Self, Box<dyn Error>> {
        let runtime_lease = RetainedHostRuntimeLease::new(core);
        Self::new_with_viewport(
            runtime_lease,
            Arc::new(crate::core::gateway::DetachedEditorRuntimeGateway),
            ui,
            RetainedViewportController::new_test_stub(),
            None,
            None,
            None,
        )
    }
}
