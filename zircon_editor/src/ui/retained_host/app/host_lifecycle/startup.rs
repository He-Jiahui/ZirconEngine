use super::super::*;
use crate::core::gui_startup_request::EditorGuiStartupRequest;
use crate::ui::host::editor_asset_manager::resolve_editor_asset_manager;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::workbench::startup::NewProjectDraft;
use zircon_runtime::asset::pipeline::manager::resolve_asset_manager;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn new(
        core: CoreHandle,
        runtime_client: SharedEditorRuntimeClient,
        ui: UiHostWindow,
        startup_request: Option<EditorGuiStartupRequest>,
    ) -> Result<Self, Box<dyn Error>> {
        zircon_runtime::profile_scope!("editor", "retained_host", "new");
        let viewport = {
            zircon_runtime::profile_scope!("editor", "retained_host", "new_viewport_controller");
            RetainedViewportController::new(core.clone())?
        };
        Self::new_with_viewport(core, runtime_client, ui, viewport, startup_request)
    }

    #[cfg(test)]
    pub(in crate::ui::retained_host::app) fn new_for_test(
        core: CoreHandle,
        ui: UiHostWindow,
    ) -> Result<Self, Box<dyn Error>> {
        Self::new_with_viewport(
            core,
            Arc::new(crate::ui::host::DetachedEditorRuntimeClient),
            ui,
            RetainedViewportController::new_test_stub(),
            None,
        )
    }

    fn new_with_viewport(
        core: CoreHandle,
        runtime_client: SharedEditorRuntimeClient,
        ui: UiHostWindow,
        viewport: RetainedViewportController,
        startup_request: Option<EditorGuiStartupRequest>,
    ) -> Result<Self, Box<dyn Error>> {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_with_viewport");
        #[cfg(not(feature = "profiling"))]
        let _ = &runtime_client;

        let resolver = ManagerResolver::new(core.clone());
        let asset_manager = {
            zircon_runtime::profile_scope!("editor", "retained_host", "new_resolve_asset_manager");
            resolve_asset_manager(resolver.core())?
        };
        let editor_asset_manager = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "new_resolve_editor_asset_manager"
            );
            resolve_editor_asset_manager(resolver.core())?
        };
        let resource_manager = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "new_resolve_resource_manager"
            );
            resolver.resource()?
        };
        let editor_manager = {
            zircon_runtime::profile_scope!("editor", "retained_host", "new_resolve_editor_manager");
            core.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)?
        };
        let asset_change_events = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "new_subscribe_asset_changes"
            );
            asset_manager.subscribe_asset_changes()
        };
        let editor_asset_change_events = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "new_subscribe_editor_asset_changes"
            );
            editor_asset_manager.subscribe_editor_asset_changes()
        };
        let resource_change_events = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "new_subscribe_resource_changes"
            );
            resource_manager.subscribe_resource_changes()
        };

        let viewport_size = UVec2::new(1280, 720);
        let startup_session = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "new_resolve_startup_session"
            );
            resolve_editor_startup_session(&editor_manager, startup_request)?
        };
        let state = {
            zircon_runtime::profile_scope!("editor", "retained_host", "new_resolve_startup_state");
            resolve_startup_state(editor_manager.as_ref(), &startup_session, viewport_size)?
        };
        let bootstrap = {
            zircon_runtime::profile_scope!("editor", "retained_host", "new_read_window_bootstrap");
            ui.get_host_window_bootstrap()
        };
        let shell_size = ShellSizePx::new(
            bootstrap.shell_frame.width.max(1.0),
            bootstrap.shell_frame.height.max(1.0),
        );
        let builtin_template_runtime = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "new_load_shared_builtin_templates"
            );
            Arc::new(callback_dispatch::load_startup_builtin_template_runtime()?)
        };
        let template_bridge = {
            zircon_runtime::profile_scope!("editor", "retained_host", "new_template_bridge");
            callback_dispatch::BuiltinHostWindowTemplateBridge::new_with_runtime(
                builtin_template_runtime.clone(),
                UiSize::new(shell_size.width, shell_size.height),
            )?
        };
        let workbench_window_bridge = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "new_workbench_window_bridge"
            );
            callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge::new_with_runtime(
                builtin_template_runtime.clone(),
                UiSize::new(shell_size.width, shell_size.height),
            )?
        };
        let floating_window_source_bridge = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "new_floating_window_source_bridge"
            );
            callback_dispatch::BuiltinFloatingWindowSourceTemplateBridge::new_with_runtime(
                builtin_template_runtime.as_ref(),
                UiSize::new(shell_size.width, shell_size.height),
            )?
        };
        let viewport_toolbar_bridge = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "new_viewport_toolbar_bridge"
            );
            callback_dispatch::BuiltinViewportToolbarTemplateBridge::new_with_runtime(
                builtin_template_runtime.clone(),
            )?
        };
        let inspector_surface_bridge = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "new_inspector_surface_bridge"
            );
            callback_dispatch::BuiltinInspectorSurfaceTemplateBridge::new_with_runtime(
                builtin_template_runtime.as_ref(),
            )?
        };
        let pane_surface_bridge = {
            zircon_runtime::profile_scope!("editor", "retained_host", "new_pane_surface_bridge");
            callback_dispatch::BuiltinPaneSurfaceTemplateBridge::new_with_runtime(
                builtin_template_runtime.as_ref(),
            )?
        };
        let component_showcase_runtime = {
            zircon_runtime::profile_scope!("editor", "retained_host", "new_component_runtime");
            EditorUiHostRuntime::default()
        };
        let native_plugin_live_host = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "new_native_plugin_live_host"
            );
            Arc::new(zircon_runtime::plugin::native::NativePluginLiveHost::default())
        };
        let runtime = {
            zircon_runtime::profile_scope!("editor", "retained_host", "new_editor_event_runtime");
            EditorEventRuntime::new(state, editor_manager.clone())
        };
        {
            zircon_runtime::profile_scope!("editor", "retained_host", "new_set_play_mode_backend");
            runtime.set_runtime_play_mode_backend(Arc::new(
                NativePluginEditorRuntimePlayModeBackend::new(native_plugin_live_host.clone()),
            ));
        }

        let mut host = {
            zircon_runtime::profile_scope!("editor", "retained_host", "new_construct_host_state");
            Self {
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
                viewport_pointer_bridge: callback_dispatch::SharedViewportPointerBridge::new(
                    UiFrame::new(0.0, 0.0, viewport_size.x as f32, viewport_size.y as f32),
                ),
                builtin_template_runtime,
                template_bridge,
                workbench_window_bridge,
                floating_window_source_bridge,
                viewport_toolbar_bridge,
                viewport_toolbar_pointer_bridge: ViewportToolbarPointerBridge::new(),
                asset_surface_bridge: None,
                welcome_surface_bridge: None,
                inspector_surface_bridge,
                pane_surface_bridge,
                component_showcase_runtime,
                component_showcase_runtime_loaded: false,
                shell_pointer_bridge: HostShellPointerBridge::new(),
                activity_rail_pointer_bridge: HostActivityRailPointerBridge::new(),
                host_page_pointer_bridge: HostPagePointerBridge::new(),
                document_tab_pointer_bridge: HostDocumentTabPointerBridge::new(),
                drawer_header_pointer_bridge: HostDrawerHeaderPointerBridge::new(),
                menu_pointer_bridge: HostMenuPointerBridge::new(),
                menu_pointer_state: HostMenuPointerState::default(),
                menu_pointer_layout: HostMenuPointerLayout::default(),
                welcome_recent_pointer_bridge: WelcomeRecentPointerBridge::new(),
                welcome_recent_pointer_state: WelcomeRecentPointerState::default(),
                welcome_recent_pointer_size: UiSize::new(0.0, 0.0),
                hierarchy_pointer_bridge: HierarchyPointerBridge::new(),
                hierarchy_pointer_state: HierarchyPointerState::default(),
                hierarchy_pointer_size: UiSize::new(0.0, 0.0),
                console_scroll_surface: ScrollSurfaceHostState::new(
                    "zircon.editor.console.pointer",
                    "editor.console",
                ),
                inspector_scroll_surface: ScrollSurfaceHostState::new(
                    "zircon.editor.inspector.pointer",
                    "editor.inspector",
                ),
                browser_asset_details_scroll_surface: ScrollSurfaceHostState::new(
                    "zircon.editor.asset_details.pointer",
                    "editor.asset_details",
                ),
                activity_asset_pointer: AssetSurfacePointerState::new(),
                browser_asset_pointer: AssetSurfacePointerState::new(),
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
        };
        host.sync_asset_workspace();
        host.drain_initial_asset_refresh_events();
        host.publish_refresh_invalidation_diagnostics();
        Ok(host)
    }
}

fn resolve_editor_startup_session(
    editor_manager: &EditorManager,
    startup_request: Option<EditorGuiStartupRequest>,
) -> Result<EditorStartupSessionDocument, crate::ui::host::EditorError> {
    match startup_request {
        Some(EditorGuiStartupRequest::OpenProject { project_path }) => {
            editor_manager.open_project_and_remember(project_path)
        }
        Some(EditorGuiStartupRequest::OpenBuiltinView { descriptor_id }) => {
            Ok(EditorStartupSessionDocument {
                mode: EditorSessionMode::Welcome,
                project: None,
                open_builtin_view: Some(descriptor_id.clone()),
                recent_projects: Vec::new(),
                draft: NewProjectDraft::renderable_empty_default(),
                status_message: format!("Opened {descriptor_id}"),
            })
        }
        Some(EditorGuiStartupRequest::CreateProject(draft)) => {
            editor_manager.create_project_and_open(draft)
        }
        None => editor_manager.resolve_startup_session(),
    }
}

#[cfg(not(test))]
fn resolve_startup_state(
    editor_manager: &EditorManager,
    session: &EditorStartupSessionDocument,
    viewport_size: UVec2,
) -> Result<EditorState, Box<dyn Error>> {
    build_startup_state(editor_manager, session, viewport_size)
}

#[cfg(test)]
fn resolve_startup_state(
    editor_manager: &EditorManager,
    session: &EditorStartupSessionDocument,
    viewport_size: UVec2,
) -> Result<EditorState, Box<dyn Error>> {
    build_startup_state(editor_manager, session, viewport_size).or_else(|error| {
        let message = error.to_string();
        if message.contains("SceneModule.Manager.DefaultLevelManager") {
            let mut state =
                EditorState::welcome(viewport_size, session.welcome_pane_snapshot(false));
            state.set_status_line(session.status_message.clone());
            Ok(state)
        } else {
            Err(error)
        }
    })
}
