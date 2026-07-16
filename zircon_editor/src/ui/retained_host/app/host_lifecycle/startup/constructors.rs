use super::*;
use crate::core::gui_startup_request::EditorGuiStartupRequest;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn new(
        core: CoreHandle,
        runtime_gateway: SharedEditorRuntimeGateway,
        ui: UiHostWindow,
        startup_request: Option<EditorGuiStartupRequest>,
    ) -> Result<Self, Box<dyn Error>> {
        zircon_runtime::profile_scope!("editor", "retained_host", "new");
        let viewport = {
            zircon_runtime::profile_scope!("editor", "retained_host", "new_viewport_controller");
            RetainedViewportController::new(core.clone())?
        };
        Self::new_with_viewport(core, runtime_gateway, ui, viewport, startup_request)
    }

    #[cfg(test)]
    pub(in crate::ui::retained_host::app) fn new_for_test(
        core: CoreHandle,
        ui: UiHostWindow,
    ) -> Result<Self, Box<dyn Error>> {
        Self::new_with_viewport(
            core,
            Arc::new(crate::core::gateway::DetachedEditorRuntimeGateway),
            ui,
            RetainedViewportController::new_test_stub(),
            None,
        )
    }
}
