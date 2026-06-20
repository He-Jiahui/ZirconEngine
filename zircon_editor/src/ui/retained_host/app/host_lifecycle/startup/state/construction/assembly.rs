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
        runtime_client,
        native_plugin_live_host,
        viewport,
        startup_session,
        viewport_size,
        shell_size,
        template_bridges,
    } = input;
    let StartupManagers {
        asset_manager,
        editor_asset_manager,
        resource_manager,
        editor_manager,
        asset_change_events,
        editor_asset_change_events,
        resource_change_events,
    } = startup_managers;
    let interaction = StartupInteractionState::new(viewport_size);

    RetainedEditorHost {
        ui,
        self_handle: None,
        runtime,
        editor_manager,
        #[cfg(feature = "profiling")]
        runtime_client,
        module_plugin_live_host_backend: Box::new(native_plugin_live_host),
        desktop_export_reports: BTreeMap::new(),
        desktop_export_jobs: build_export_actions::DesktopExportJobQueue::default(),
        desktop_export_output_overrides: BTreeMap::new(),
        desktop_export_wizard_sessions:
            build_export_wizard_session::DesktopExportWizardSessions::default(),
        viewport,
        asset_manager,
        editor_asset_manager,
        resource_manager,
        asset_change_events,
        editor_asset_change_events,
        resource_change_events,
        startup_session,
        viewport_size,
        viewport_pointer_bridge: interaction.viewport_pointer_bridge,
        builtin_template_runtime: template_bridges.builtin_template_runtime,
        template_bridge: template_bridges.template_bridge,
        workbench_window_bridge: template_bridges.workbench_window_bridge,
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
        welcome_recent_pointer_state: interaction.welcome_recent_pointer_state,
        welcome_recent_pointer_size: interaction.welcome_recent_pointer_size,
        hierarchy_pointer_bridge: interaction.hierarchy_pointer_bridge,
        hierarchy_pointer_state: interaction.hierarchy_pointer_state,
        hierarchy_pointer_size: interaction.hierarchy_pointer_size,
        console_scroll_surface: interaction.console_scroll_surface,
        inspector_scroll_surface: interaction.inspector_scroll_surface,
        browser_asset_details_scroll_surface: interaction.browser_asset_details_scroll_surface,
        activity_asset_pointer: interaction.activity_asset_pointer,
        browser_asset_pointer: interaction.browser_asset_pointer,
        active_asset_drag_payload: None,
        active_scene_drag_payload: None,
        active_object_drag_payload: None,
        native_window_presenters: NativeWindowPresenterStore::default(),
        floating_window_projection_bundle: FloatingWindowProjectionBundle::default(),
        callback_source_window: None,
        last_focused_callback_window: None,
        active_layout_preset: None,
        shell_size,
        chrome_metrics: WorkbenchChromeMetrics::default(),
        shell_geometry: None,
        transient_region_preferred: BTreeMap::new(),
        active_drawer_resize: None,
        pending_close_prompt: None,
        invalidation: HostInvalidationRoot::with_initial_full_rebuild(),
        pending_ui_perf_scenario: None,
        presentation_dirty: true,
        layout_dirty: true,
        window_metrics_dirty: true,
        render_dirty: true,
    }
}
