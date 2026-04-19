use super::*;
use crate::core::host::asset_editor::resolve_editor_asset_manager;
use crate::ui::slint_host::floating_window_projection::{
    build_floating_window_projection_bundle_with_shared_source,
    resolve_floating_window_projection_base_outer_frame,
    resolve_floating_window_projection_shared_source, FloatingWindowProjectionBundle,
};
use crate::ui::slint_host::root_shell_projection::resolve_root_viewport_content_frame;
use zircon_runtime::asset::pipeline::manager::resolve_asset_manager;

impl SlintEditorHost {
    pub(super) fn new(core: CoreHandle, ui: UiHostWindow) -> Result<Self, Box<dyn Error>> {
        Self::new_with_viewport(core.clone(), ui, SlintViewportController::new(core)?)
    }

    #[cfg(test)]
    pub(super) fn new_for_test(core: CoreHandle, ui: UiHostWindow) -> Result<Self, Box<dyn Error>> {
        Self::new_with_viewport(core, ui, SlintViewportController::new_test_stub())
    }

    fn new_with_viewport(
        core: CoreHandle,
        ui: UiHostWindow,
        viewport: SlintViewportController,
    ) -> Result<Self, Box<dyn Error>> {
        let resolver = ManagerResolver::new(core.clone());
        let asset_server = resolve_asset_manager(resolver.core())?;
        let editor_asset_server = resolve_editor_asset_manager(resolver.core())?;
        let resource_server = resolver.resource()?;
        let editor_manager = core.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)?;
        let asset_change_events = asset_server.subscribe_asset_changes();
        let editor_asset_change_events = editor_asset_server.subscribe_editor_asset_changes();
        let resource_change_events = resource_server.subscribe_resource_changes();

        let viewport_size = UVec2::new(1280, 720);
        let startup_session = editor_manager.resolve_startup_session()?;
        let state =
            resolve_startup_state(editor_manager.as_ref(), &startup_session, viewport_size)?;
        let bootstrap = ui.get_host_window_bootstrap();
        let shell_size = ShellSizePx::new(
            bootstrap.shell_frame.width.max(1.0),
            bootstrap.shell_frame.height.max(1.0),
        );
        let template_bridge = callback_dispatch::BuiltinWorkbenchTemplateBridge::new(UiSize::new(
            shell_size.width,
            shell_size.height,
        ))?;
        let floating_window_source_bridge =
            callback_dispatch::BuiltinFloatingWindowSourceTemplateBridge::new(UiSize::new(
                shell_size.width,
                shell_size.height,
            ))?;
        let viewport_toolbar_bridge =
            callback_dispatch::BuiltinViewportToolbarTemplateBridge::new()?;
        let asset_surface_bridge = callback_dispatch::BuiltinAssetSurfaceTemplateBridge::new()?;
        let welcome_surface_bridge = callback_dispatch::BuiltinWelcomeSurfaceTemplateBridge::new()?;
        let inspector_surface_bridge =
            callback_dispatch::BuiltinInspectorSurfaceTemplateBridge::new()?;
        let pane_surface_bridge = callback_dispatch::BuiltinPaneSurfaceTemplateBridge::new()?;

        let mut host = Self {
            ui,
            self_handle: None,
            runtime: EditorEventRuntime::new(state, editor_manager.clone()),
            editor_manager,
            viewport,
            asset_server,
            editor_asset_server,
            resource_server,
            asset_change_events,
            editor_asset_change_events,
            resource_change_events,
            startup_session,
            viewport_size,
            viewport_pointer_bridge: callback_dispatch::SharedViewportPointerBridge::new(
                UiFrame::new(0.0, 0.0, viewport_size.x as f32, viewport_size.y as f32),
            ),
            template_bridge,
            floating_window_source_bridge,
            viewport_toolbar_bridge,
            viewport_toolbar_pointer_bridge: ViewportToolbarPointerBridge::new(),
            asset_surface_bridge,
            welcome_surface_bridge,
            inspector_surface_bridge,
            pane_surface_bridge,
            shell_pointer_bridge: WorkbenchShellPointerBridge::new(),
            activity_rail_pointer_bridge: WorkbenchActivityRailPointerBridge::new(),
            host_page_pointer_bridge: WorkbenchHostPagePointerBridge::new(),
            document_tab_pointer_bridge: WorkbenchDocumentTabPointerBridge::new(),
            drawer_header_pointer_bridge: WorkbenchDrawerHeaderPointerBridge::new(),
            menu_pointer_bridge: WorkbenchMenuPointerBridge::new(),
            menu_pointer_state: WorkbenchMenuPointerState::default(),
            menu_pointer_layout: WorkbenchMenuPointerLayout::default(),
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
            presentation_dirty: true,
            layout_dirty: true,
            window_metrics_dirty: true,
            render_dirty: true,
        };
        host.sync_asset_workspace();
        Ok(host)
    }

    pub(super) fn tick(&mut self) {
        if let Err(error) = self.refresh_project_assets() {
            self.set_status_line(error);
        }

        self.sync_shell_size();
        self.recompute_if_dirty();

        if self.render_dirty {
            if let Some(extract) = self.runtime.render_frame_extract() {
                if let Err(error) = self.viewport.submit_extract(extract, self.viewport_size) {
                    self.set_status_line(format!("Viewport submit failed: {error}"));
                }
            }
            self.render_dirty = false;
        }

        if let Some(image) = self.viewport.poll_image() {
            self.ui
                .global::<crate::ui::slint_host::PaneSurfaceHostContext>()
                .set_viewport_image(image);
        }
        if let Some(error) = self.viewport.take_error() {
            self.set_status_line(error);
            self.recompute_if_dirty();
        }
    }

    pub(super) fn refresh_ui(&mut self) {
        self.recompute_if_dirty();
    }

    pub(super) fn build_chrome(&self) -> crate::EditorChromeSnapshot {
        self.runtime.chrome_snapshot()
    }

    pub(super) fn sync_shell_size(&mut self) {
        let bootstrap = self.ui.get_host_window_bootstrap();
        let next = ShellSizePx::new(
            bootstrap.shell_frame.width.max(1.0),
            bootstrap.shell_frame.height.max(1.0),
        );
        if (next.width - self.shell_size.width).abs() <= 0.5
            && (next.height - self.shell_size.height).abs() <= 0.5
        {
            return;
        }
        self.shell_size = next;
        self.window_metrics_dirty = true;
        self.presentation_dirty = true;
    }

    pub(super) fn recompute_if_dirty(&mut self) {
        if !self.presentation_dirty && !self.layout_dirty && !self.window_metrics_dirty {
            return;
        }

        let layout = self.runtime.current_layout();
        let descriptors = self.runtime.descriptors();
        let mut chrome = self.build_chrome();
        let mut model = WorkbenchViewModel::build(&chrome);
        let geometry = compute_workbench_shell_geometry(
            &model,
            &chrome,
            &layout,
            &descriptors,
            self.shell_size,
            &self.chrome_metrics,
            if self.transient_region_preferred.is_empty() {
                None
            } else {
                Some(&self.transient_region_preferred)
            },
        );
        let _ = self.template_bridge.recompute_layout_with_workbench_model(
            UiSize::new(self.shell_size.width, self.shell_size.height),
            &model,
            &self.chrome_metrics,
        );
        let _ = self
            .floating_window_source_bridge
            .recompute_layout(UiSize::new(self.shell_size.width, self.shell_size.height));
        let root_shell_frames = self.template_bridge.root_shell_frames();
        let floating_window_shared_source = resolve_floating_window_projection_shared_source(
            &self.floating_window_source_bridge.source_frames(),
        );
        for (window_index, window) in model.floating_windows.iter().enumerate() {
            let frame = resolve_floating_window_projection_base_outer_frame(
                window,
                window_index,
                floating_window_shared_source,
            );
            self.editor_manager.sync_native_window_projection_bounds(
                &window.window_id,
                [frame.x, frame.y, frame.width, frame.height],
            );
        }
        let native_window_hosts = self.editor_manager.native_window_hosts();
        let floating_window_projection_bundle =
            build_floating_window_projection_bundle_with_shared_source(
                &model,
                floating_window_shared_source,
                &self.chrome_metrics,
                &native_window_hosts,
            );
        let viewport_content_frame = resolve_root_viewport_content_frame(
            &geometry,
            Some(&root_shell_frames),
            active_document_shows_viewport_toolbar(&model),
        );

        if let Some(next_viewport_size) = viewport_size_from_frame(viewport_content_frame) {
            if next_viewport_size != self.viewport_size {
                self.viewport_size = next_viewport_size;
                self.apply_dispatch_result(callback_dispatch::dispatch_viewport_event(
                    &self.runtime,
                    EditorViewportEvent::Resized {
                        width: self.viewport_size.x,
                        height: self.viewport_size.y,
                    },
                ));
                chrome = self.build_chrome();
                model = WorkbenchViewModel::build(&chrome);
            }
        }
        self.viewport_pointer_bridge
            .update_viewport_frame(UiFrame::new(
                0.0,
                0.0,
                viewport_content_frame.width.max(0.0),
                viewport_content_frame.height.max(0.0),
            ));
        self.shell_pointer_bridge
            .update_layout_with_root_shell_frames(
                self.shell_size,
                &geometry,
                model.drawer_ring.visible,
                &model.floating_windows,
                Some(&root_shell_frames),
                Some(&floating_window_projection_bundle),
            );
        self.sync_activity_rail_pointer_layout(&model, &geometry);
        self.sync_host_page_pointer_layout(&model);
        self.sync_document_tab_pointer_layout(
            &model,
            &geometry,
            &floating_window_projection_bundle,
        );
        self.sync_drawer_header_pointer_layout(&model, &geometry);

        let preset_names = self.runtime.preset_names();
        let ui_asset_panes = self.collect_ui_asset_panes();
        apply_presentation(
            &self.ui,
            &model,
            &chrome,
            &geometry,
            &preset_names,
            self.active_layout_preset.as_deref(),
            &ui_asset_panes,
            Some(&root_shell_frames),
            &floating_window_projection_bundle,
        );
        self.sync_native_window_presenters(
            &model,
            &chrome,
            &geometry,
            &preset_names,
            &ui_asset_panes,
            &floating_window_projection_bundle,
        );
        self.sync_menu_pointer_layout(&chrome, &preset_names);
        self.sync_welcome_recent_pointer_layout(&chrome.welcome);
        self.sync_hierarchy_pointer_layout(&chrome.scene_entries);
        self.sync_detail_pointer_layouts(&chrome);
        self.sync_asset_pointer_layouts(&chrome);
        self.floating_window_projection_bundle = floating_window_projection_bundle;
        self.shell_geometry = Some(geometry);
        self.presentation_dirty = false;
        self.layout_dirty = false;
        self.window_metrics_dirty = false;
    }

    fn collect_ui_asset_panes(&self) -> BTreeMap<String, crate::UiAssetEditorPanePresentation> {
        self.runtime
            .current_view_instances()
            .into_iter()
            .filter(|instance| instance.descriptor_id.0 == "editor.ui_asset")
            .filter_map(|instance| {
                self.editor_manager
                    .ui_asset_editor_pane_presentation(&instance.instance_id)
                    .ok()
                    .map(|presentation| (instance.instance_id.0, presentation))
            })
            .collect()
    }

    pub(super) fn set_status_line(&mut self, message: impl Into<String>) {
        self.runtime.set_status_line(message);
        self.presentation_dirty = true;
    }

    pub(super) fn apply_dispatch_effects(&mut self, effects: SlintDispatchEffects) {
        if let Some(name) = effects.active_layout_preset_name.clone() {
            self.active_layout_preset = Some(name);
        }
        if effects.reset_active_layout_preset {
            self.active_layout_preset = None;
        }
        if effects.layout_dirty {
            self.layout_dirty = true;
        }
        if effects.render_dirty {
            self.render_dirty = true;
        }
        if effects.presentation_dirty {
            self.presentation_dirty = true;
        }
        if effects.sync_asset_workspace {
            self.sync_asset_workspace();
        }
        if effects.refresh_asset_details {
            self.refresh_selected_asset_details();
        }
        if effects.refresh_visible_asset_previews {
            self.refresh_visible_asset_previews();
        }
        if effects.import_model_requested {
            if let Err(error) = self.import_model_into_project() {
                self.set_status_line(error);
            }
        }
        if effects.present_welcome_surface {
            if let Err(error) = self.present_welcome_surface(
                "Open an existing project or create a renderable empty project.",
            ) {
                self.set_status_line(error);
            }
        }
    }

    pub(super) fn apply_dispatch_result(&mut self, result: Result<SlintDispatchEffects, String>) {
        match result {
            Ok(effects) => self.apply_dispatch_effects(effects),
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn mark_layout_dirty(&mut self) {
        self.layout_dirty = true;
        self.presentation_dirty = true;
    }

    pub(super) fn mark_render_and_presentation_dirty(&mut self) {
        self.render_dirty = true;
        self.presentation_dirty = true;
    }

    fn sync_native_window_presenters(
        &mut self,
        model: &WorkbenchViewModel,
        chrome: &crate::EditorChromeSnapshot,
        geometry: &WorkbenchShellGeometry,
        preset_names: &[String],
        ui_asset_panes: &BTreeMap<String, crate::UiAssetEditorPanePresentation>,
        floating_window_projection_bundle: &FloatingWindowProjectionBundle,
    ) {
        let targets =
            collect_native_floating_window_targets(model, floating_window_projection_bundle);
        let active_preset_name = self.active_layout_preset.as_deref();
        let host_handle = self.self_handle.as_ref().and_then(Weak::upgrade);
        if let Err(error) = self.native_window_presenters.sync_targets(
            &targets,
            |ui, target| {
                if let Some(host) = host_handle.as_ref() {
                    wire_callbacks(ui, host);
                    let host_weak = Rc::downgrade(host);
                    let window_id = target.window_id.clone();
                    ui.window().on_close_requested(move || {
                        if let Some(host) = host_weak.upgrade() {
                            host.borrow_mut()
                                .native_floating_window_close_requested(&window_id)
                        } else {
                            slint::CloseRequestResponse::KeepWindowShown
                        }
                    });
                }
            },
            |ui, target| {
                apply_presentation(
                    ui,
                    model,
                    chrome,
                    geometry,
                    preset_names,
                    active_preset_name,
                    ui_asset_panes,
                    None,
                    floating_window_projection_bundle,
                );
                configure_native_floating_window_presentation(ui, target);
            },
        ) {
            self.set_status_line(format!("Native window sync failed: {error}"));
        }
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

fn active_document_shows_viewport_toolbar(model: &WorkbenchViewModel) -> bool {
    model
        .document_tabs
        .iter()
        .find(|tab| tab.active)
        .or_else(|| model.document_tabs.first())
        .is_some_and(|tab| tab.content_kind == ViewContentKind::Scene)
}
