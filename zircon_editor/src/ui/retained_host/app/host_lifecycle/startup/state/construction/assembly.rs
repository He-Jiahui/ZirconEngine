use std::collections::BTreeMap;

use super::super::super::super::super::*;
use super::super::super::resources::StartupManagers;
use super::super::interaction::StartupInteractionState;
use super::input::StartupHostConstruction;

pub(in crate::ui::retained_host::app::host_lifecycle::startup) fn construct_startup_host(
    input: StartupHostConstruction,
) -> RetainedEditorHost {
    zircon_runtime::profile_scope!("editor", "retained_host", "new_construct_host_state");
    let StartupHostConstruction {
        ui,
        runtime,
        startup_managers,
        #[cfg(feature = "profiling")]
        runtime_gateway,
        runtime_lease,
        native_plugin_host,
        viewport,
        startup_session,
        viewport_size,
        shell_size,
        shell_scale_factor,
        template_bridges,
    } = input;
    let StartupManagers {
        asset_runtime_access,
        editor_manager,
        asset_change_events,
        editor_asset_change_events,
        resource_change_events,
    } = startup_managers;
    let interaction = StartupInteractionState::new(viewport_size);
    let editor_jobs = editor_manager.context().jobs().clone();
    let editor_tools = editor_manager.context().tools().clone();
    viewport.bind_jobs(editor_jobs.clone());
    let hub_focus_request_attention: Arc<dyn Fn() + Send + Sync> = {
        let attention = ui.window().window_attention();
        Arc::new(move || attention.request())
    };
    let native_plugin_watch_wake = ui.background_event_wake_callback();

    let mut host = RetainedEditorHost {
        ui,
        self_handle: None,
        runtime_lease,
        runtime,
        runtime_shutdown_receipt: None,
        editor_manager,
        hub_focus_binding: HubFocusBinding::default(),
        hub_focus_request_attention,
        module_plugin_projection_cache: Default::default(),
        #[cfg(feature = "profiling")]
        runtime_gateway,
        module_plugin_live_host_backend: Box::new(
            module_plugin_actions::NativePluginDevelopmentLiveHostBackend::new(
                native_plugin_host,
                editor_jobs.clone(),
                native_plugin_watch_wake,
            ),
        ),
        desktop_export_reports: BTreeMap::new(),
        desktop_export_jobs: build_export_actions::DesktopExportJobQueue::new(editor_jobs.clone()),
        desktop_export_output_overrides: BTreeMap::new(),
        desktop_export_wizard_sessions:
            build_export_wizard_session::DesktopExportWizardSessions::new_with_tools(
                editor_jobs,
                editor_tools,
            ),
        viewport,
        asset_runtime_access,
        asset_change_events,
        editor_asset_change_events,
        resource_change_events,
        asset_refresh_queue_age: Default::default(),
        asset_refresh_accumulator: Default::default(),
        pending_active_scene_reload: None,
        active_scene_reload_admission: None,
        active_scene_reload_conflict: None,
        active_scene_reload_decision_sequence: 0,
        pending_model_import: None,
        pending_asset_deletion: None,
        pending_asset_relocation: None,
        startup_session,
        welcome_project_probe: welcome_session::WelcomeProjectProbeState::default(),
        viewport_size,
        viewport_pointer_bridge: interaction.viewport_pointer_bridge,
        play_preview_input_focus_active: false,
        play_preview_view_focus_active: false,
        play_viewport_pick: Default::default(),
        last_simulate_camera: None,
        builtin_template_runtime: template_bridges.builtin_template_runtime,
        plugin_template_generation: 0,
        plugin_template_capabilities: Vec::new(),
        template_bridge: template_bridges.template_bridge,
        workbench_window_bridge: template_bridges.workbench_window_bridge,
        host_chrome_projection_cache: Default::default(),
        console_pane_projection_cache: Default::default(),
        module_plugins_pane_projection_cache: Default::default(),
        floating_window_source_bridge: template_bridges.floating_window_source_bridge,
        viewport_toolbar_bridge: template_bridges.viewport_toolbar_bridge,
        viewport_toolbar_pointer_bridge: interaction.viewport_toolbar_pointer_bridge,
        asset_surface_bridge: None,
        welcome_surface_bridge: None,
        inspector_surface_bridge: template_bridges.inspector_surface_bridge,
        pane_surface_bridge: template_bridges.pane_surface_bridge,
        component_showcase_runtime: template_bridges.component_showcase_runtime,
        component_showcase_runtime_loaded: false,
        shell_pointer_bridge: interaction.shell_pointer_bridge,
        activity_rail_pointer_bridge: interaction.activity_rail_pointer_bridge,
        host_page_pointer_bridge: interaction.host_page_pointer_bridge,
        document_tab_pointer_bridge: interaction.document_tab_pointer_bridge,
        drawer_header_pointer_bridge: interaction.drawer_header_pointer_bridge,
        menu_pointer_bridge: interaction.menu_pointer_bridge,
        menu_pointer_state: interaction.menu_pointer_state,
        menu_pointer_layout: interaction.menu_pointer_layout,
        welcome_recent_pointer_bridge: interaction.welcome_recent_pointer_bridge,
        welcome_recent_pointer_size: interaction.welcome_recent_pointer_size,
        hierarchy_pointer_bridge: interaction.hierarchy_pointer_bridge,
        hierarchy_pointer_state: interaction.hierarchy_pointer_state,
        hierarchy_pointer_size: interaction.hierarchy_pointer_size,
        hierarchy_scene_entries: interaction.hierarchy_scene_entries,
        hierarchy_world_watch: None,
        hierarchy_filter_query: String::new(),
        console_scroll_surface: interaction.console_scroll_surface,
        inspector_scroll_surface: interaction.inspector_scroll_surface,
        browser_asset_details_scroll_surface: interaction.browser_asset_details_scroll_surface,
        activity_asset_pointer: interaction.activity_asset_pointer,
        browser_asset_pointer: interaction.browser_asset_pointer,
        active_asset_drag_payload: None,
        active_scene_drag_payload: None,
        active_hierarchy_drag_node_ids: Vec::new(),
        last_hierarchy_rename_click: None,
        active_object_drag_payload: None,
        native_window_presenters: NativeWindowPresenterStore::default(),
        floating_window_projection_bundle: FloatingWindowProjectionBundle::default(),
        callback_source_window: None,
        last_focused_callback_window: None,
        active_layout_preset: None,
        shell_size,
        shell_scale_factor,
        shell_scale_mode: ResolutionScaleMode::ConstantPhysical,
        chrome_metrics: WorkbenchChromeMetrics::default(),
        shell_geometry: None,
        committed_shell_state: None,
        shell_token_region_defaults: None,
        transient_region_preferred: BTreeMap::new(),
        active_drawer_resize: None,
        project_close_coordinator: ProjectCloseCoordinator::default(),
        pending_close_prompt: None,
        pending_document_save_all: false,
        queued_document_save_all: false,
        scene_picker_session: None,
        invalidation: HostInvalidationRoot::with_initial_full_rebuild(),
        pending_ui_perf_scenario: None,
        pending_activity_projection_refresh: false,
        runtime_diagnostics_refresh_target: RuntimeDiagnosticsRefreshTarget::None,
        presentation_dirty: true,
        layout_dirty: true,
        window_metrics_dirty: true,
        render_dirty: true,
    };
    if host.startup_session.mode == EditorSessionMode::Welcome {
        host.schedule_welcome_project_probe();
    }
    let hierarchy_domain = host.runtime.active_hierarchy_world_domain();
    if let Err(error) = host.ensure_hierarchy_world_watch(hierarchy_domain) {
        host.set_status_line(error);
    }
    host
}
