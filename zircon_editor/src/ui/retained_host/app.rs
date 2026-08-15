use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use zircon_runtime::asset::project::{ProjectManager, ProjectPaths};
use zircon_runtime::asset::watch::AssetChange;
use zircon_runtime::core::framework::channel::ChannelReceiver;
use zircon_runtime::core::resource::ResourceEventReceiver;
use zircon_runtime::core::CoreHandle;
use zircon_runtime::scene::{NodeId, Scene, WorldInspectionHierarchyRow};
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceLocator,
};
use zircon_runtime_interface::ui::{
    binding::UiBindingValue,
    binding::UiEventKind,
    component::{
        UiComponentBindingTarget, UiComponentEvent, UiComponentEventEnvelope, UiDragPayload,
        UiDragPayloadKind, UiDragSourceMetadata, UiValue,
    },
    dispatch::UiPointerComponentEvent,
    layout::UiFrame,
    layout::UiPoint,
    layout::UiSize,
};

use crate::core::editing::paths::canonical_model_source_path;
use crate::core::editor_event::EditorViewportEvent;
use crate::core::gateway::SharedEditorRuntimeGateway;
use crate::core::gui_startup_request::EditorGuiStartupRequest;
use crate::core::hub_link::{HubEditorHandshake, HubFocusSignalWatch, HubHandshakeError};
use crate::core::logging::{EditorLogService, LogEntry, LogSeverity, LogSource};
use crate::core::play::NativePluginBridgeActivation;
use crate::ui::binding_dispatch::WelcomeHostEvent;
use crate::ui::host::editor_asset_manager::{EditorAssetChange, EditorAssetChangeSubscription};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::resource_access::resolve_ready_handle;
use crate::ui::host::EditorHostEventController;
use crate::ui::host::{EditorError, EditorManager};
use crate::ui::retained_host::ui_perf::UiPerfScenario;
use crate::ui::template_runtime::{EditorUiHostRuntime, EditorUiHostRuntimeError};
use crate::ui::v2_design_tokens::install_editor_v2_design_tokens;
use crate::ui::workbench::autolayout::{
    ResolutionContext, ResolutionScaleMode, ShellRegionId, ShellSizePx, WorkbenchChromeMetrics,
    WorkbenchShellGeometry,
};
use crate::ui::workbench::layout::{ActivityDrawerMode, LayoutCommand, MainPageId};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::project::EditorProjectDocument;
use crate::ui::workbench::snapshot::{SceneEntries, ViewContentKind};
use crate::ui::workbench::startup::{EditorSessionMode, EditorStartupSessionDocument};
use crate::ui::workbench::state::EditorState;

use super::activity_rail_pointer::{
    build_host_activity_rail_pointer_layout_with_workbench_layout_frames,
    HostActivityRailPointerBridge, HostActivityRailPointerSide,
};
use super::asset_pointer::{
    AssetContentListPointerBridge, AssetContentListPointerLayout, AssetFolderTreePointerBridge,
    AssetFolderTreePointerLayout, AssetListPointerState, AssetPointerContentRoute,
    AssetPointerReferenceRoute, AssetReferenceListPointerBridge, AssetReferenceListPointerLayout,
};
use super::callback_dispatch;
use super::detail_pointer::{
    asset_details_scroll_layout, console_content_extent, console_scroll_layout,
    inspector_scroll_layout,
};
use super::document_tab_pointer::{
    build_host_document_tab_pointer_layout_with_workbench_layout_frames,
    HostDocumentTabPointerBridge,
};
use super::drawer_header_pointer::{
    build_host_drawer_header_pointer_layout_with_workbench_layout_frames,
    HostDrawerHeaderPointerBridge,
};
use super::drawer_resize::dispatch_resize_to_group;
use super::event_bridge::UiHostEventEffects;
use super::floating_window_projection::FloatingWindowProjectionBundle;
use super::hierarchy_pointer::{
    HierarchyPointerBridge, HierarchyPointerLayout, HierarchyPointerState,
};
use super::host_page_pointer::{build_host_page_pointer_layout, HostPagePointerBridge};
use super::menu_pointer::{HostMenuPointerBridge, HostMenuPointerLayout, HostMenuPointerState};
use super::scroll_surface_host::ScrollSurfaceHostState;
use super::shell_pointer::{HostShellPointerBridge, HostShellPointerRoute};
use super::tab_drag::host_shell_pointer_route_group_key;
use super::ui::apply_presentation;
use super::viewport::RetainedViewportController;
use super::viewport_toolbar_pointer::ViewportToolbarPointerBridge;
use super::welcome_recent_pointer::{
    WelcomeRecentPointerAction, WelcomeRecentPointerBridge, WelcomeRecentPointerLayout,
    WelcomeRecentPointerState,
};
use super::{
    apply_host_appearance_from_tokens, apply_host_paint_scale_factor, FrameRect,
    HostWindowDiagnostic, HostWindowDiagnosticSeverity, UiHostWindow,
    WorkbenchContextMenuRequestData,
};

mod activity_log_jump;
mod asset_content_pointer;
mod asset_drag_payload;
mod asset_reference_pointer;
mod asset_runtime_access;
mod asset_surface_pointer_state;
mod asset_tree_pointer;
mod assets;
mod automation;
mod autosave;
pub(crate) mod backend_refresh;
mod build_export_actions;
mod build_export_projection;
mod build_export_wizard_session;
mod callback_wiring;
mod close_prompt;
mod command_palette_actions;
mod committed_shell_state;
mod component_showcase_runtime;
mod detail_scroll_pointer;
mod helpers;
mod hierarchy_filter;
mod hierarchy_pointer;
pub(in crate::ui::retained_host) mod hierarchy_rename;
mod hierarchy_world_watch;
mod host_lifecycle;
mod inspector;
mod invalidation;
mod job_progress;
mod menu_pointer;
mod module_plugin_actions;
mod module_plugin_projection;
mod native_keyboard_actions;
mod native_window_close;
mod native_windows;
mod pane_payload_visibility;
mod pane_surface_actions;
mod pointer_layout;
mod product_frame_diagnostics;
mod profiling;
mod reference_drop_payload;
mod runtime_diagnostics_visibility;
mod runtime_lease;
mod scene_picker_actions;
mod scene_picker_session;
#[cfg(test)]
mod scene_picker_session_tests;
mod showcase_event_inputs;
mod startup;
#[cfg(test)]
mod tests;
mod ui_asset_editor;
mod ui_asset_editor_detail_events;
mod ui_asset_editor_detail_routes;
mod viewport;
mod viewport_image_redraw;
mod viewport_toolbar_projection;
mod welcome_recent_pointer;
mod welcome_session;
mod workbench_context_menu;
mod workbench_notifications;
mod workbench_pointer;
mod workbench_snapshot_access;
mod workspace_docking;
use super::run_config::EditorHostRunConfig;
use asset_runtime_access::RetainedHostAssetRuntimeAccess;
pub use automation::{run_retained_host_automation, RetainedHostAutomationResult};
use callback_wiring::wire_callbacks;
pub(super) use helpers::{
    asset_surface_visible, compute_window_menu_popup_height,
    derive_animation_assets_from_model_source, resolve_callback_source_window_id,
    shell_region_group_key, stage_model_source, viewport_size_from_frame,
};
use hierarchy_world_watch::HierarchyWorldWatch;
pub(crate) use invalidation::HostInvalidationMask;
use invalidation::HostInvalidationRoot;
pub(crate) use native_windows::NativeWindowPresenterStore;
#[cfg(test)]
pub(crate) use native_windows::{
    collect_native_floating_window_targets, configure_native_floating_window_presentation,
    NativeFloatingWindowTarget,
};
use product_frame_diagnostics::{editor_product_frame_diagnostics, emit_product_frame_log};
use runtime_lease::RetainedHostRuntimeLease;
pub(crate) use startup::build_startup_state;

pub fn run_editor(
    core: CoreHandle,
    runtime_gateway: SharedEditorRuntimeGateway,
) -> Result<(), Box<dyn Error>> {
    run_editor_with_config(core, runtime_gateway, EditorHostRunConfig::new())
}

pub fn run_editor_with_startup_request(
    core: CoreHandle,
    runtime_gateway: SharedEditorRuntimeGateway,
    startup_request: Option<EditorGuiStartupRequest>,
) -> Result<(), Box<dyn Error>> {
    run_editor_with_config(
        core,
        runtime_gateway,
        EditorHostRunConfig::new().with_startup_request(startup_request),
    )
}

pub fn run_editor_with_config(
    core: CoreHandle,
    runtime_gateway: SharedEditorRuntimeGateway,
    config: EditorHostRunConfig,
) -> Result<(), Box<dyn Error>> {
    let exit_after_first_presented_frame = config.exit_after_first_presented_frame();
    let startup_scene_uri = config.startup_scene_uri().cloned();
    let startup_layout_preset = config.startup_layout_preset().map(str::to_owned);
    let (
        startup_request,
        prepared_project,
        first_presented_frame_capture_path,
        editor_plugin_registrations,
        hub_handshake,
    ) = config.into_parts();
    let mut hub_startup_reporter = HubStartupReporter::new(hub_handshake);
    let product_frame_evidence_requested = first_presented_frame_capture_path.is_some();
    let ui = UiHostWindow::new().map_err(|error| hub_startup_reporter.report_failure(error))?;
    ui.set_exit_after_first_presented_frame(exit_after_first_presented_frame);
    ui.set_first_presented_frame_capture_path(first_presented_frame_capture_path);
    let mut retained_host = match RetainedEditorHost::new(
        core,
        runtime_gateway,
        ui.clone_strong(),
        startup_request,
        prepared_project,
        hub_startup_reporter.session(),
    ) {
        Ok(retained_host) => retained_host,
        Err(error) => {
            if let Some(process_id) = error
                .downcast_ref::<EditorError>()
                .and_then(EditorError::hub_focus_forwarded_process_id)
            {
                hub_startup_reporter.report_ready_for_process(process_id)?;
                return Ok(());
            }
            return Err(hub_startup_reporter.report_failure(error));
        }
    };
    let settings_snapshot = retained_host.editor_manager.context().settings().snapshot();
    apply_host_appearance_from_tokens(settings_snapshot.design_tokens());
    apply_host_paint_scale_factor(
        ResolutionContext::from_physical_size_with_scale_mode(
            retained_host.shell_size,
            retained_host.shell_scale_factor,
            retained_host.shell_scale_mode,
        )
        .effective_scale_factor(),
    );
    ui.sync_host_paint_theme();
    install_editor_v2_design_tokens(settings_snapshot.as_ref());
    for registration in editor_plugin_registrations {
        retained_host
            .runtime
            .register_editor_plugin_registration(registration)
            .map_err(|error| hub_startup_reporter.report_failure(error))?;
    }
    retained_host
        .sync_plugin_template_documents_if_changed()
        .map_err(|error| hub_startup_reporter.report_failure(error))?;
    if let Some(scene_uri) = startup_scene_uri {
        retained_host
            .open_startup_scene(scene_uri)
            .map_err(std::io::Error::other)
            .map_err(|error| hub_startup_reporter.report_failure(error))?;
    }
    if let Some(name) = startup_layout_preset {
        // Project presets and plugin-provided views must be visible before the stored layout is applied.
        retained_host
            .editor_manager
            .apply_layout_command(LayoutCommand::LoadPreset { name })
            .map_err(|error| hub_startup_reporter.report_failure(error))?;
    }
    let host = Rc::new(RefCell::new(retained_host));
    wire_callbacks(&ui, &host);
    let host_weak = Rc::downgrade(&host);
    ui.window().on_close_requested(move || {
        if let Some(host) = host_weak.upgrade() {
            host.borrow_mut().native_main_window_close_requested()
        } else {
            crate::ui::retained_host::primitives::CloseRequestResponse::KeepWindowShown
        }
    });
    host.borrow_mut().self_handle = Some(Rc::downgrade(&host));

    host.borrow_mut().refresh_ui();
    let _hub_focus_watch = host
        .borrow()
        .editor_manager
        .active_project_session_focus_target()
        .map(|(project_root, instance_id)| {
            let attention = ui.window().window_attention();
            HubFocusSignalWatch::start(project_root, instance_id, move || attention.request())
        })
        .transpose()
        .map_err(|error| hub_startup_reporter.report_failure(error))?;
    if hub_startup_reporter.is_pending() && _hub_focus_watch.is_none() {
        let error = std::io::Error::other(
            "Hub launch reached retained host startup without an active project session focus watcher",
        );
        return Err(hub_startup_reporter.report_failure(error).into());
    }
    hub_startup_reporter.report_ready()?;
    if product_frame_evidence_requested {
        let retained_host = host.borrow();
        let diagnostic =
            editor_product_frame_diagnostics(&retained_host.runtime.editor_snapshot())?;
        emit_product_frame_log(retained_host.runtime.context().logs(), diagnostic);
    }
    let run_result = ui.run();
    let host_window_diagnostics = ui.take_host_diagnostics();
    {
        let retained_host = host.borrow();
        emit_host_window_diagnostics(
            retained_host.runtime.context().logs(),
            host_window_diagnostics,
        );
    }
    let unfinished_editor_jobs = {
        let retained_host = host.borrow();
        retained_host
            .editor_manager
            .context()
            .autosave()
            .shutdown(std::time::Instant::now() + std::time::Duration::from_secs(5))
    };
    // The runtime project is still available here, so normal editor shutdown can release its
    // OS-backed admission lease before settings and host state are torn down.
    let project_close_result = {
        let retained_host = host.borrow();
        retained_host.editor_manager.close_project()
    };
    let settings_persistence = {
        let retained_host = host.borrow();
        retained_host
            .editor_manager
            .context()
            .settings_persistence()
            .clone()
    };
    let settings_shutdown = match settings_persistence.flush_then_shutdown() {
        Ok(closeout) => closeout,
        Err(error) => {
            let guard = settings_persistence.shutdown();
            drop(guard);
            return Err(error.into());
        }
    };
    let settings_shutdown_result = settings_shutdown.finish();
    if let Some(error) = ui.take_fatal_failure() {
        return Err(error.into());
    }
    run_result?;
    settings_shutdown_result?;
    project_close_result?;
    if !unfinished_editor_jobs.is_empty() {
        let jobs = unfinished_editor_jobs
            .iter()
            .map(|job| format!("{} ({:?})", job.label(), job.category()))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(std::io::Error::other(format!(
            "editor job shutdown deadline expired with unfinished jobs: {jobs}"
        ))
        .into());
    }
    if let Some(error) = ui.take_first_presented_frame_capture_error() {
        return Err(std::io::Error::other(error).into());
    }
    Ok(())
}

/// Holds the Hub handshake unresolved until the host reaches its complete startup gate.
///
/// A mailbox failure never replaces the startup error: the local error remains authoritative for
/// the editor process, while the best-effort mailbox outcome prevents Hub from waiting silently.
struct HubStartupReporter {
    handshake: Option<HubEditorHandshake>,
}

impl HubStartupReporter {
    fn new(handshake: Option<HubEditorHandshake>) -> Self {
        Self { handshake }
    }

    fn is_pending(&self) -> bool {
        self.handshake.is_some()
    }

    fn session(&self) -> Option<zircon_runtime_interface::hub_protocol::HubSessionToken> {
        self.handshake.as_ref().map(HubEditorHandshake::session)
    }

    fn report_failure<E>(&mut self, error: E) -> E
    where
        E: Display,
    {
        if let Some(handshake) = self.handshake.take() {
            if let Err(mailbox_error) = handshake.publish_failed(error.to_string()) {
                eprintln!(
                    "[zircon_editor] failed to publish Hub startup failure while handling `{error}`: {mailbox_error}"
                );
            }
        }
        error
    }

    fn report_ready(&mut self) -> Result<(), HubHandshakeError> {
        self.report_ready_for_process(std::process::id())
    }

    fn report_ready_for_process(&mut self, process_id: u32) -> Result<(), HubHandshakeError> {
        let Some(handshake) = self.handshake.take() else {
            return Ok(());
        };
        handshake.publish_ready(process_id)
    }
}

// Native window code reports DTOs without depending on editor core; the composition root owns
// their conversion into structured editor records after the event loop releases its UI borrows.
fn emit_host_window_diagnostics(logs: &EditorLogService, diagnostics: Vec<HostWindowDiagnostic>) {
    for diagnostic in diagnostics {
        let severity = match diagnostic.severity() {
            HostWindowDiagnosticSeverity::Info => LogSeverity::Info,
            HostWindowDiagnosticSeverity::Warning => LogSeverity::Warning,
            HostWindowDiagnosticSeverity::Error => LogSeverity::Error,
        };
        let entry = LogEntry::new(LogSource::editor(), severity, diagnostic.message(), 0, None)
            .or_else(|_| {
                LogEntry::new(
                    LogSource::editor(),
                    severity,
                    "editor_host_window diagnostic exceeds the log-entry limit.",
                    0,
                    None,
                )
            });
        if let Ok(entry) = entry {
            let _ = logs.emit(entry);
        }
    }
}

#[cfg(test)]
mod host_window_diagnostic_log_tests {
    use super::{emit_host_window_diagnostics, HostWindowDiagnostic, HostWindowDiagnosticSeverity};
    use crate::core::logging::{EditorLogService, LogFilter, LogSeverity, LogSource};

    #[test]
    fn native_window_diagnostics_keep_their_severity_at_the_editor_log_boundary() {
        let logs = EditorLogService::default();

        emit_host_window_diagnostics(
            &logs,
            vec![
                HostWindowDiagnostic::new(HostWindowDiagnosticSeverity::Info, "frame ready"),
                HostWindowDiagnostic::new(HostWindowDiagnosticSeverity::Error, "present failed"),
            ],
        );

        let records = logs.snapshot(&LogFilter::default());
        assert_eq!(records.len(), 2);
        assert!(records
            .iter()
            .all(|record| record.entry().source() == &LogSource::editor()));
        assert_eq!(records[0].entry().severity(), LogSeverity::Info);
        assert_eq!(records[0].entry().message(), "frame ready");
        assert_eq!(records[1].entry().severity(), LogSeverity::Error);
        assert_eq!(records[1].entry().message(), "present failed");
    }

    #[test]
    fn oversized_native_window_diagnostic_uses_a_bounded_severity_preserving_fallback() {
        let logs = EditorLogService::default();

        emit_host_window_diagnostics(
            &logs,
            vec![HostWindowDiagnostic::new(
                HostWindowDiagnosticSeverity::Warning,
                "x".repeat(9 * 1024),
            )],
        );

        let records = logs.snapshot(&LogFilter::default());
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].entry().severity(), LogSeverity::Warning);
        assert_eq!(
            records[0].entry().message(),
            "editor_host_window diagnostic exceeds the log-entry limit."
        );
    }
}

struct RetainedEditorHost {
    ui: UiHostWindow,
    self_handle: Option<Weak<RefCell<RetainedEditorHost>>>,
    runtime_lease: RetainedHostRuntimeLease,
    runtime: EditorHostEventController,
    editor_manager: Arc<EditorManager>,
    module_plugin_projection_cache:
        RefCell<module_plugin_projection::ModulePluginPaneProjectionCache>,
    #[cfg(feature = "profiling")]
    runtime_gateway: SharedEditorRuntimeGateway,
    module_plugin_live_host_backend: Box<dyn module_plugin_actions::ModulePluginLiveHostBackend>,
    desktop_export_reports: BTreeMap<String, build_export_actions::DesktopExportExecutionSummary>,
    desktop_export_jobs: build_export_actions::DesktopExportJobQueue,
    desktop_export_output_overrides: BTreeMap<String, std::path::PathBuf>,
    desktop_export_wizard_sessions: build_export_wizard_session::DesktopExportWizardSessions,
    viewport: RetainedViewportController,
    asset_runtime_access: RetainedHostAssetRuntimeAccess,
    asset_change_events: ChannelReceiver<AssetChange>,
    editor_asset_change_events: EditorAssetChangeSubscription,
    resource_change_events: ResourceEventReceiver,
    asset_refresh_queue_age: assets::AssetRefreshQueueAgeState,
    asset_refresh_accumulator: assets::AssetRefreshAccumulator,
    startup_session: EditorStartupSessionDocument,
    welcome_project_probe: welcome_session::WelcomeProjectProbeState,
    viewport_size: UVec2,
    viewport_pointer_bridge: callback_dispatch::SharedViewportPointerBridge,
    builtin_template_runtime: Arc<EditorUiHostRuntime>,
    plugin_template_generation: u64,
    plugin_template_capabilities: Vec<String>,
    template_bridge: callback_dispatch::BuiltinHostWindowTemplateBridge,
    workbench_window_bridge: callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    host_chrome_projection_cache:
        crate::ui::layouts::windows::workbench_host_window::HostChromeProjectionCache,
    floating_window_source_bridge: callback_dispatch::BuiltinFloatingWindowSourceTemplateBridge,
    viewport_toolbar_bridge: callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    viewport_toolbar_pointer_bridge: ViewportToolbarPointerBridge,
    asset_surface_bridge: Option<callback_dispatch::BuiltinAssetSurfaceTemplateBridge>,
    welcome_surface_bridge: Option<callback_dispatch::BuiltinWelcomeSurfaceTemplateBridge>,
    inspector_surface_bridge: callback_dispatch::BuiltinInspectorSurfaceTemplateBridge,
    pane_surface_bridge: callback_dispatch::BuiltinPaneSurfaceTemplateBridge,
    component_showcase_runtime: EditorUiHostRuntime,
    component_showcase_runtime_loaded: bool,
    shell_pointer_bridge: HostShellPointerBridge,
    activity_rail_pointer_bridge: HostActivityRailPointerBridge,
    host_page_pointer_bridge: HostPagePointerBridge,
    document_tab_pointer_bridge: HostDocumentTabPointerBridge,
    drawer_header_pointer_bridge: HostDrawerHeaderPointerBridge,
    menu_pointer_bridge: HostMenuPointerBridge,
    menu_pointer_state: HostMenuPointerState,
    menu_pointer_layout: HostMenuPointerLayout,
    welcome_recent_pointer_bridge: WelcomeRecentPointerBridge,
    welcome_recent_pointer_state: WelcomeRecentPointerState,
    welcome_recent_pointer_size: UiSize,
    hierarchy_pointer_bridge: HierarchyPointerBridge,
    hierarchy_pointer_state: HierarchyPointerState,
    hierarchy_pointer_size: UiSize,
    hierarchy_scene_entries: Arc<[WorldInspectionHierarchyRow]>,
    hierarchy_world_watch: Option<HierarchyWorldWatch>,
    hierarchy_filter_query: String,
    console_scroll_surface: ScrollSurfaceHostState,
    inspector_scroll_surface: ScrollSurfaceHostState,
    browser_asset_details_scroll_surface: ScrollSurfaceHostState,
    activity_asset_pointer: AssetSurfacePointerState,
    browser_asset_pointer: AssetSurfacePointerState,
    active_asset_drag_payload: Option<UiDragPayload>,
    active_scene_drag_payload: Option<UiDragPayload>,
    active_hierarchy_drag_node_ids: Vec<NodeId>,
    last_hierarchy_rename_click: Option<hierarchy_rename::HierarchyRenameClick>,
    active_object_drag_payload: Option<UiDragPayload>,
    native_window_presenters: NativeWindowPresenterStore,
    floating_window_projection_bundle: FloatingWindowProjectionBundle,
    callback_source_window: Option<MainPageId>,
    last_focused_callback_window: Option<MainPageId>,
    active_layout_preset: Option<String>,
    shell_size: ShellSizePx,
    shell_scale_factor: f32,
    shell_scale_mode: ResolutionScaleMode,
    chrome_metrics: WorkbenchChromeMetrics,
    shell_geometry: Option<WorkbenchShellGeometry>,
    committed_shell_state: Option<committed_shell_state::CommittedShellState>,
    shell_token_region_defaults: Option<(
        Arc<zircon_runtime_interface::ui::design_tokens::EditorDesignTokens>,
        BTreeMap<ShellRegionId, f32>,
    )>,
    transient_region_preferred: BTreeMap<ShellRegionId, f32>,
    active_drawer_resize: Option<ActiveDrawerResize>,
    pending_close_prompt: Option<close_prompt::PendingClosePrompt>,
    scene_picker_session: Option<scene_picker_session::ScenePickerSession>,
    invalidation: HostInvalidationRoot,
    pending_ui_perf_scenario: Option<UiPerfScenario>,
    pending_activity_projection_refresh: bool,
    runtime_diagnostics_visible: bool,
    presentation_dirty: bool,
    layout_dirty: bool,
    window_metrics_dirty: bool,
    render_dirty: bool,
}

impl Drop for RetainedEditorHost {
    fn drop(&mut self) {
        self.editor_manager.context().autosave().begin_shutdown();
        let Some(watch) = self.hierarchy_world_watch.take() else {
            return;
        };
        if watch.belongs_to_gateway_generation(self.runtime.edit_world_gateway_generation()) {
            let _ = self.runtime.unwatch_edit_world_for_view(watch.token());
        }
    }
}

impl RetainedEditorHost {
    fn sync_settings_projections(&mut self) {
        let snapshot = self.editor_manager.context().settings().snapshot();
        if install_editor_v2_design_tokens(snapshot.as_ref()) {
            apply_host_appearance_from_tokens(snapshot.design_tokens());
            self.ui.sync_host_paint_theme();
            self.mark_presentation_dirty();
        }
    }

    fn sync_plugin_template_documents_if_changed(
        &mut self,
    ) -> Result<(), EditorUiHostRuntimeError> {
        let (generation, enabled_capabilities) = self.runtime.plugin_template_revision();
        if self.plugin_template_generation == generation
            && self.plugin_template_capabilities == enabled_capabilities
        {
            return Ok(());
        }

        let (generation, enabled_capabilities, templates_by_owner) =
            self.runtime.enabled_plugin_template_descriptors();
        self.builtin_template_runtime
            .sync_plugin_v2_template_descriptor_sets(&templates_by_owner)?;

        // The runtime publishes the complete owner set atomically before this revision advances.
        self.plugin_template_generation = generation;
        self.plugin_template_capabilities = enabled_capabilities;
        self.mark_presentation_dirty();
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct ActiveDrawerResize {
    region: ShellRegionId,
    start_x: f32,
    start_y: f32,
    base_preferred: f32,
}

struct AssetSurfacePointerState {
    snapshot: Option<Arc<crate::ui::workbench::snapshot::AssetWorkspaceSnapshot>>,
    tree_bridge: AssetFolderTreePointerBridge,
    tree_state: AssetListPointerState,
    tree_size: UiSize,
    content_bridge: AssetContentListPointerBridge,
    content_state: AssetListPointerState,
    content_size: UiSize,
    references: AssetReferenceListSurfacePointerState,
    used_by: AssetReferenceListSurfacePointerState,
}

struct AssetReferenceListSurfacePointerState {
    bridge: AssetReferenceListPointerBridge,
    state: AssetListPointerState,
    size: UiSize,
}
