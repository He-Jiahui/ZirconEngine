use super::*;
use crate::ui::host::editor_asset_manager::resolve_editor_asset_manager;
use crate::ui::slint_host::floating_window_projection::{
    build_floating_window_projection_bundle_with_shared_source,
    resolve_floating_window_projection_base_outer_frame,
    resolve_floating_window_projection_shared_source, FloatingWindowProjectionBundle,
};
use crate::ui::slint_host::root_shell_projection::resolve_root_viewport_content_frame;
use zircon_runtime::asset::pipeline::manager::resolve_asset_manager;

impl SlintEditorHost {
    pub(super) fn new(
        core: CoreHandle,
        runtime_client: SharedEditorRuntimeClient,
        ui: UiHostWindow,
    ) -> Result<Self, Box<dyn Error>> {
        Self::new_with_viewport(
            core.clone(),
            runtime_client,
            ui,
            SlintViewportController::new(core)?,
        )
    }

    #[cfg(test)]
    pub(super) fn new_for_test(core: CoreHandle, ui: UiHostWindow) -> Result<Self, Box<dyn Error>> {
        Self::new_with_viewport(
            core,
            Arc::new(crate::ui::host::DetachedEditorRuntimeClient),
            ui,
            SlintViewportController::new_test_stub(),
        )
    }

    fn new_with_viewport(
        core: CoreHandle,
        runtime_client: SharedEditorRuntimeClient,
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
        let template_bridge = callback_dispatch::BuiltinHostWindowTemplateBridge::new(
            UiSize::new(shell_size.width, shell_size.height),
        )?;
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
        let mut component_showcase_runtime = EditorUiHostRuntime::default();
        component_showcase_runtime.load_builtin_host_templates()?;

        let native_plugin_live_host = Arc::new(zircon_runtime::plugin::NativePluginLiveHost::default());
        let runtime = EditorEventRuntime::new(state, editor_manager.clone());
        runtime.set_runtime_play_mode_backend(Arc::new(
            NativePluginEditorRuntimePlayModeBackend::new(native_plugin_live_host.clone()),
        ));

        let mut host = Self {
            ui,
            self_handle: None,
            runtime,
            runtime_client,
            editor_manager,
            module_plugin_live_host_backend: Box::new(native_plugin_live_host),
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
            component_showcase_runtime,
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
            if let Some(submission) = self.runtime.render_frame_submission() {
                if let Err(error) = self.viewport.submit_extract_with_ui(
                    submission.extract,
                    submission.ui,
                    self.viewport_size,
                ) {
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

    pub(super) fn build_chrome(&self) -> crate::ui::workbench::snapshot::EditorChromeSnapshot {
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
        let animation_panes = self.collect_animation_editor_panes();
        let runtime_diagnostics = self.editor_manager.runtime_diagnostics();
        let module_plugins = self.module_plugins_pane_data(&chrome);
        apply_presentation(
            &self.ui,
            &model,
            &chrome,
            &geometry,
            &preset_names,
            self.active_layout_preset.as_deref(),
            &ui_asset_panes,
            &animation_panes,
            Some(&runtime_diagnostics),
            &module_plugins,
            Some(&root_shell_frames),
            &floating_window_projection_bundle,
            Some(&self.component_showcase_runtime),
        );
        let world_space_ui_surfaces =
            crate::ui::slint_host::build_world_space_ui_surface_submissions_from_host_scene(
                &self.ui.get_host_presentation().host_scene_data,
            );
        self.viewport
            .submit_world_space_ui_surfaces(world_space_ui_surfaces);
        self.sync_native_window_presenters(
            &model,
            &chrome,
            &geometry,
            &preset_names,
            &ui_asset_panes,
            &animation_panes,
            &runtime_diagnostics,
            &floating_window_projection_bundle,
        );
        self.sync_menu_pointer_layout(&model, &chrome, &preset_names);
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

    fn collect_ui_asset_panes(
        &self,
    ) -> BTreeMap<String, crate::ui::asset_editor::UiAssetEditorPanePresentation> {
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

    fn collect_animation_editor_panes(
        &self,
    ) -> BTreeMap<String, crate::ui::animation_editor::AnimationEditorPanePresentation> {
        self.runtime
            .current_view_instances()
            .into_iter()
            .filter(|instance| {
                matches!(
                    instance.descriptor_id.0.as_str(),
                    "editor.animation_sequence" | "editor.animation_graph"
                )
            })
            .filter_map(|instance| {
                self.editor_manager
                    .animation_editor_pane_presentation(&instance.instance_id)
                    .ok()
                    .map(|presentation| (instance.instance_id.0, presentation))
            })
            .collect()
    }

    fn module_plugins_pane_data(
        &self,
        chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
    ) -> crate::ui::layouts::windows::workbench_host_window::ModulePluginsPaneViewData {
        use crate::ui::layouts::common::model_rc;
        use crate::ui::layouts::windows::workbench_host_window::{
            ModulePluginStatusViewData, ModulePluginsPaneViewData,
        };
        use crate::ui::workbench::project::project_root_path;
        use zircon_runtime::asset::project::ProjectManifest;

        let mut diagnostics = Vec::new();
        let report = project_root_path(&chrome.project_path)
            .map_err(|error| error.to_string())
            .and_then(|project_root| {
                let manifest_path = project_root.join("zircon-project.toml");
                ProjectManifest::load(&manifest_path)
                    .map(|manifest| {
                        self.editor_manager
                            .native_plugin_status_report(&project_root, &manifest)
                    })
                    .map_err(|error| {
                        format!(
                            "plugin status uses builtin catalog because project manifest is unavailable: {error}"
                        )
                    })
            })
            .unwrap_or_else(|error| {
                diagnostics.push(error);
                self.editor_manager
                    .plugin_status_report(&fallback_project_manifest())
            });

        diagnostics.extend(report.diagnostics.clone());
        ModulePluginsPaneViewData {
            plugins: model_rc(
                report
                    .plugins
                    .into_iter()
                    .map(|plugin| {
                        let primary_action = module_plugin_primary_action(
                            &plugin.plugin_id,
                            plugin.enabled,
                            plugin.required,
                        );
                        let packaging_action_label =
                            format!("Cycle {}", packaging_label(plugin.packaging));
                        let packaging_action_id =
                            module_plugin_action_id("Plugin.Packaging.Next", &plugin.plugin_id);
                        let target_modes_action_id =
                            module_plugin_action_id("Plugin.TargetModes.Next", &plugin.plugin_id);
                        let unload_action_id =
                            module_plugin_action_id("Plugin.Unload", &plugin.plugin_id);
                        let hot_reload_action_id =
                            module_plugin_action_id("Plugin.HotReload", &plugin.plugin_id);
                        let feature_action =
                            module_plugin_feature_action(&plugin.optional_features);
                        ModulePluginStatusViewData {
                            plugin_id: plugin.plugin_id.into(),
                            display_name: plugin.display_name.into(),
                            package_source: plugin.package_source.into(),
                            load_state: plugin.load_state.into(),
                            enabled: plugin.enabled,
                            required: plugin.required,
                            target_modes: plugin
                                .target_modes
                                .iter()
                                .map(target_mode_label)
                                .collect::<Vec<_>>()
                                .join(", ")
                                .into(),
                            packaging: packaging_label(plugin.packaging).into(),
                            runtime_crate: plugin.runtime_crate.unwrap_or_default().into(),
                            editor_crate: plugin.editor_crate.unwrap_or_default().into(),
                            runtime_capabilities: plugin.runtime_capabilities.join(", ").into(),
                            editor_capabilities: plugin.editor_capabilities.join(", ").into(),
                            optional_features: module_plugin_optional_feature_summary(
                                &plugin.optional_features,
                            )
                            .into(),
                            feature_action_label: feature_action.0.into(),
                            feature_action_id: feature_action.1.into(),
                            diagnostics: plugin.diagnostics.join("\n").into(),
                            primary_action_label: primary_action.0.into(),
                            primary_action_id: primary_action.1.into(),
                            packaging_action_label: packaging_action_label.into(),
                            packaging_action_id: packaging_action_id.into(),
                            target_modes_action_label: "Cycle targets".into(),
                            target_modes_action_id: target_modes_action_id.into(),
                            unload_action_label: "Unload".into(),
                            unload_action_id: unload_action_id.into(),
                            hot_reload_action_label: "Hot Reload".into(),
                            hot_reload_action_id: hot_reload_action_id.into(),
                        }
                    })
                    .collect(),
            ),
            diagnostics: diagnostics.join("\n").into(),
        }
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
        chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
        geometry: &WorkbenchShellGeometry,
        preset_names: &[String],
        ui_asset_panes: &BTreeMap<String, crate::ui::asset_editor::UiAssetEditorPanePresentation>,
        animation_panes: &BTreeMap<
            String,
            crate::ui::animation_editor::AnimationEditorPanePresentation,
        >,
        runtime_diagnostics: &zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot,
        floating_window_projection_bundle: &FloatingWindowProjectionBundle,
    ) {
        let module_plugins = self.module_plugins_pane_data(chrome);
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
                    animation_panes,
                    Some(runtime_diagnostics),
                    &module_plugins,
                    None,
                    floating_window_projection_bundle,
                    Some(&self.component_showcase_runtime),
                );
                configure_native_floating_window_presentation(ui, target);
            },
        ) {
            self.set_status_line(format!("Native window sync failed: {error}"));
        }
    }
}

fn module_plugin_optional_feature_summary(
    features: &[crate::ui::host::EditorPluginFeatureStatus],
) -> String {
    features
        .iter()
        .map(|feature| {
            let state = if feature.enabled {
                if feature.available {
                    "enabled"
                } else {
                    "blocked"
                }
            } else if feature.available {
                "ready"
            } else {
                "blocked"
            };
            let dependencies = feature
                .dependencies
                .iter()
                .map(|dependency| {
                    let dependency_state =
                        match (dependency.plugin_enabled, dependency.capability_available) {
                            (true, true) => "ok",
                            (false, _) => "missing plugin",
                            (true, false) => "missing capability",
                        };
                    let role = if dependency.primary { "primary " } else { "" };
                    format!(
                        "{role}{}:{} ({dependency_state})",
                        dependency.plugin_id, dependency.capability
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            if dependencies.is_empty() {
                format!("{} [{state}]", feature.display_name)
            } else {
                format!("{} [{state}] deps: {dependencies}", feature.display_name)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn module_plugin_feature_action(
    features: &[crate::ui::host::EditorPluginFeatureStatus],
) -> (String, String) {
    if let Some(feature) = features
        .iter()
        .find(|feature| !feature.enabled && !feature.available)
    {
        return (
            "Enable Deps".to_string(),
            module_plugin_feature_action_id(
                "Plugin.Feature.EnableDependencies",
                &feature.owner_plugin_id,
                &feature.id,
            ),
        );
    }
    if let Some(feature) = features
        .iter()
        .find(|feature| !feature.enabled && feature.available)
    {
        return (
            "Enable Feature".to_string(),
            module_plugin_feature_action_id(
                "Plugin.Feature.Enable",
                &feature.owner_plugin_id,
                &feature.id,
            ),
        );
    }
    if let Some(feature) = features
        .iter()
        .find(|feature| feature.enabled && !feature.required)
    {
        return (
            "Disable Feature".to_string(),
            module_plugin_feature_action_id(
                "Plugin.Feature.Disable",
                &feature.owner_plugin_id,
                &feature.id,
            ),
        );
    }
    (String::new(), String::new())
}

fn module_plugin_feature_action_id(prefix: &str, plugin_id: &str, feature_id: &str) -> String {
    format!("{prefix}.{plugin_id}.{feature_id}")
}

fn module_plugin_primary_action(
    plugin_id: &str,
    enabled: bool,
    required: bool,
) -> (String, String) {
    if required {
        return ("Required".to_string(), String::new());
    }

    if enabled {
        (
            "Disable".to_string(),
            module_plugin_action_id("Plugin.Disable", plugin_id),
        )
    } else {
        (
            "Enable".to_string(),
            module_plugin_action_id("Plugin.Enable", plugin_id),
        )
    }
}

fn module_plugin_action_id(prefix: &str, plugin_id: &str) -> String {
    format!("{prefix}.{plugin_id}")
}

#[cfg(test)]
mod module_plugin_action_projection_tests {
    use super::*;

    #[test]
    fn module_plugin_primary_action_respects_required_and_enabled_state() {
        assert_eq!(
            module_plugin_primary_action("physics", true, false),
            ("Disable".to_string(), "Plugin.Disable.physics".to_string())
        );
        assert_eq!(
            module_plugin_primary_action("physics", false, false),
            ("Enable".to_string(), "Plugin.Enable.physics".to_string())
        );
        assert_eq!(
            module_plugin_primary_action("core", true, true),
            ("Required".to_string(), String::new())
        );
    }

    #[test]
    fn module_plugin_optional_feature_summary_lists_dependency_state() {
        let summary =
            module_plugin_optional_feature_summary(&[crate::ui::host::EditorPluginFeatureStatus {
                id: "sound.timeline_animation_track".to_string(),
                display_name: "Sound Timeline Animation Track".to_string(),
                owner_plugin_id: "sound".to_string(),
                enabled: false,
                required: false,
                available: false,
                target_modes: vec![zircon_runtime::RuntimeTargetMode::EditorHost],
                packaging: zircon_runtime::plugin::ExportPackagingStrategy::LibraryEmbed,
                runtime_crate: Some("zircon_plugin_sound_timeline_animation_runtime".to_string()),
                editor_crate: Some("zircon_plugin_sound_timeline_animation_editor".to_string()),
                provided_capabilities: vec![
                    "runtime.feature.sound.timeline_animation_track".to_string()
                ],
                dependencies: vec![
                    crate::ui::host::EditorPluginFeatureDependencyStatus {
                        plugin_id: "sound".to_string(),
                        capability: "runtime.plugin.sound".to_string(),
                        primary: true,
                        plugin_enabled: true,
                        capability_available: true,
                    },
                    crate::ui::host::EditorPluginFeatureDependencyStatus {
                        plugin_id: "animation".to_string(),
                        capability: "runtime.feature.animation.timeline_event_track".to_string(),
                        primary: false,
                        plugin_enabled: false,
                        capability_available: false,
                    },
                ],
                diagnostics: Vec::new(),
            }]);

        assert!(summary.contains("Sound Timeline Animation Track [blocked]"));
        assert!(summary.contains("primary sound:runtime.plugin.sound (ok)"));
        assert!(summary
            .contains("animation:runtime.feature.animation.timeline_event_track (missing plugin)"));
    }

    #[test]
    fn module_plugin_feature_action_prefers_dependency_gate_then_enable() {
        let blocked = crate::ui::host::EditorPluginFeatureStatus {
            id: "sound.timeline_animation_track".to_string(),
            display_name: "Sound Timeline Animation Track".to_string(),
            owner_plugin_id: "sound".to_string(),
            enabled: false,
            required: false,
            available: false,
            target_modes: vec![zircon_runtime::RuntimeTargetMode::EditorHost],
            packaging: zircon_runtime::plugin::ExportPackagingStrategy::LibraryEmbed,
            runtime_crate: None,
            editor_crate: None,
            provided_capabilities: Vec::new(),
            dependencies: Vec::new(),
            diagnostics: Vec::new(),
        };
        assert_eq!(
            module_plugin_feature_action(&[blocked.clone()]),
            (
                "Enable Deps".to_string(),
                "Plugin.Feature.EnableDependencies.sound.sound.timeline_animation_track"
                    .to_string()
            )
        );

        let ready = crate::ui::host::EditorPluginFeatureStatus {
            available: true,
            ..blocked
        };
        assert_eq!(
            module_plugin_feature_action(&[ready]),
            (
                "Enable Feature".to_string(),
                "Plugin.Feature.Enable.sound.sound.timeline_animation_track".to_string()
            )
        );
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

fn target_mode_label(mode: &zircon_runtime::RuntimeTargetMode) -> &'static str {
    match mode {
        zircon_runtime::RuntimeTargetMode::ClientRuntime => "client",
        zircon_runtime::RuntimeTargetMode::ServerRuntime => "server",
        zircon_runtime::RuntimeTargetMode::EditorHost => "editor",
    }
}

fn packaging_label(strategy: zircon_runtime::plugin::ExportPackagingStrategy) -> &'static str {
    match strategy {
        zircon_runtime::plugin::ExportPackagingStrategy::SourceTemplate => "source-template",
        zircon_runtime::plugin::ExportPackagingStrategy::LibraryEmbed => "library-embed",
        zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic => "native-dynamic",
    }
}

fn fallback_project_manifest() -> zircon_runtime::asset::project::ProjectManifest {
    zircon_runtime::asset::project::ProjectManifest::new(
        "Unsaved",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml")
            .expect("fallback project asset URI is valid"),
        1,
    )
}
