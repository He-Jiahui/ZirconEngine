use super::resources::resolve_startup_managers;
use super::state::{construct_startup_host, StartupHostConstruction};
use super::template_bridges::create_startup_template_bridges;
use super::*;
use crate::core::gui_startup_request::EditorGuiStartupRequest;
use zircon_runtime_interface::hub_protocol::HubSessionToken;
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

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
        runtime_lease: RetainedHostRuntimeLease,
        runtime_gateway: SharedEditorRuntimeGateway,
        ui: UiHostWindow,
        viewport: RetainedViewportController,
        startup_request: Option<EditorGuiStartupRequest>,
        project_runtime_build_set: Option<ZrRuntimeBuildSetId>,
        hub_launch_session: Option<HubSessionToken>,
    ) -> Result<Self, Box<dyn Error>> {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_with_viewport");
        #[cfg(not(feature = "profiling"))]
        let _ = &runtime_gateway;

        ui.set_runtime_presenter_factory(viewport.runtime_presenter_factory());

        let startup_access = runtime_lease.startup_access();
        let startup_managers =
            resolve_startup_managers(&startup_access, ui.background_event_wake_callback())?;
        startup_managers
            .editor_manager
            .configure_hub_launch_session(hub_launch_session);
        startup_managers
            .editor_manager
            .configure_project_runtime_build_set(project_runtime_build_set);
        let editor_jobs = startup_managers.editor_manager.context().jobs().clone();
        ui.bind_profile_artifact_jobs(editor_jobs.clone());
        ui.bind_visual_asset_jobs(editor_jobs, ui.background_visual_asset_wake_callback());
        let viewport_size = UVec2::new(1280, 720);
        let startup_session_state = resolve_startup_session_state(
            startup_managers.editor_manager.clone(),
            startup_request,
            viewport_size,
        );
        let startup_session_state = startup_session_state?;
        let (startup_session, runtime) = startup_session_state.into_parts();
        let shell_size = resolve_startup_shell_size(&ui);
        let shell_scale_factor = resolve_startup_shell_scale_factor(&ui);
        let template_bridges = create_startup_template_bridges(shell_size)?;
        let runtime_backend = create_startup_runtime_backend(runtime);

        let mut host = construct_startup_host(StartupHostConstruction {
            ui,
            runtime: runtime_backend.runtime,
            startup_managers,
            #[cfg(feature = "profiling")]
            runtime_gateway,
            runtime_lease,
            native_plugin_host: runtime_backend.native_plugin_host,
            viewport,
            startup_session,
            viewport_size,
            shell_size,
            shell_scale_factor,
            template_bridges,
        });
        finalize_startup_host(&mut host);
        Ok(host)
    }
}
