use super::resources::resolve_startup_managers;
use super::state::{construct_startup_host, StartupHostConstruction};
use super::template_bridges::create_startup_template_bridges;
use super::*;
use crate::core::gui_startup_request::EditorGuiStartupRequest;

mod finalize;
mod runtime_backend;
mod session_state;
mod shell_bootstrap;

use finalize::finalize_startup_host;
use runtime_backend::create_startup_runtime_backend;
use session_state::resolve_startup_session_state;
use shell_bootstrap::{resolve_startup_shell_scale_factor, resolve_startup_shell_size};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) fn new_with_viewport(
        core: CoreHandle,
        runtime_client: SharedEditorRuntimeClient,
        ui: UiHostWindow,
        viewport: RetainedViewportController,
        startup_request: Option<EditorGuiStartupRequest>,
    ) -> Result<Self, Box<dyn Error>> {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_with_viewport");
        #[cfg(not(feature = "profiling"))]
        let _ = &runtime_client;

        let startup_managers = resolve_startup_managers(&core)?;
        let viewport_size = UVec2::new(1280, 720);
        let startup_session_state = resolve_startup_session_state(
            startup_managers.editor_manager.as_ref(),
            startup_request,
            viewport_size,
        );
        let startup_session_state = startup_session_state?;
        let shell_size = resolve_startup_shell_size(&ui);
        let shell_scale_factor = resolve_startup_shell_scale_factor(&ui);
        let template_bridges = create_startup_template_bridges(shell_size)?;
        let runtime_backend = create_startup_runtime_backend(
            startup_session_state.state,
            startup_managers.editor_manager.clone(),
        );

        let mut host = construct_startup_host(StartupHostConstruction {
            ui,
            runtime: runtime_backend.runtime,
            startup_managers,
            #[cfg(feature = "profiling")]
            runtime_client,
            native_plugin_live_host: runtime_backend.native_plugin_live_host,
            viewport,
            startup_session: startup_session_state.startup_session,
            viewport_size,
            shell_size,
            shell_scale_factor,
            template_bridges,
        });
        finalize_startup_host(&mut host);
        Ok(host)
    }
}
