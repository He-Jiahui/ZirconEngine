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
use zircon_runtime_interface::hub_protocol::{
    HubEditorReadyReceiptV1, HubEditorStartupFailureCodeV1,
};
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
use crate::core::hub_link::{HubEditorHandshake, HubFocusBinding, HubHandshakeError};
use crate::core::logging::{EditorLogService, LogEntry, LogSeverity, LogSource};
use crate::core::play::NativePluginBridgeActivation;
use crate::ui::binding_dispatch::WelcomeHostEvent;
use crate::ui::host::editor_asset_manager::{EditorAssetChange, EditorAssetChangeSubscription};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::resource_access::resolve_ready_handle;
use crate::ui::host::{EditorError, EditorManager};
use crate::ui::host::{EditorHostEventController, EditorRuntimeSessionShutdownReceipt};
use crate::ui::retained_host::ui_perf::UiPerfScenario;
use crate::ui::template_runtime::EditorUiHostRuntime;
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
    asset_details_scroll_layout, console_scroll_layout, console_snapshot_content_extent,
    inspector_scroll_layout,
};
use super::document_tab_pointer::{
    build_host_document_tab_pointer_layout, HostDocumentTabPointerBridge,
};
use super::drawer_header_pointer::{
    build_host_drawer_header_pointer_layout, HostDrawerHeaderPointerBridge,
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
use super::tab_drag::{
    host_shell_pointer_route_group_key, host_shell_pointer_route_matches_group_key,
};
use super::ui::apply_presentation;
use super::viewport::RetainedViewportController;
use super::viewport_toolbar_pointer::ViewportToolbarPointerBridge;
use super::welcome_recent_pointer::{
    WelcomeRecentPointerAction, WelcomeRecentPointerBridge, WelcomeRecentPointerLayout,
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
mod document_save;
mod helpers;
mod hierarchy_filter;
mod hierarchy_pointer;
pub(in crate::ui::retained_host) mod hierarchy_rename;
mod hierarchy_world_watch;
mod host_lifecycle;
mod hub_focus_binding;
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
mod play_preview_redraw;
mod play_viewport_pick;
mod plugin_template_documents;
mod pointer_layout;
mod product_frame_diagnostics;
mod profiling;
mod project_close;
mod project_save;
mod reference_drop_payload;
mod runtime_diagnostics_visibility;
mod runtime_lease;
mod runtime_shutdown;
mod scene_picker_actions;
mod scene_picker_session;
#[cfg(test)]
mod scene_picker_session_tests;
mod settings_window_actions;
mod showcase_event_inputs;
mod simulate_camera_sync;
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
mod workbench_tooltip;
mod workspace_docking;
use super::run_config::EditorHostRunConfig;
use asset_runtime_access::RetainedHostAssetRuntimeAccess;
pub use automation::{run_retained_host_automation, RetainedHostAutomationResult};
use callback_wiring::wire_callbacks;
pub(super) use helpers::{
    asset_surface_visible, compute_window_menu_popup_height, resolve_callback_source_window_id,
    shell_region_group_key, viewport_size_from_frame,
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
use runtime_diagnostics_visibility::RuntimeDiagnosticsRefreshTarget;
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
    let play_backend = config.play_backend();
    let exit_after_first_presented_frame = config.exit_after_first_presented_frame();
    let startup_scene_uri = config.startup_scene_uri().cloned();
    let startup_layout_preset = config.startup_layout_preset().map(str::to_owned);
    let (
        startup_request,
        first_presented_frame_capture_path,
        editor_plugin_registrations,
        project_runtime_build_set,
        hub_handshake,
    ) = config.into_parts();
    if matches!(
        startup_request.as_ref(),
        Some(EditorGuiStartupRequest::Project { .. })
    ) && project_runtime_build_set.is_none()
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "project startup requires an App-preflighted runtime BuildSet",
        )
        .into());
    }
    if matches!(
        startup_request.as_ref(),
        Some(EditorGuiStartupRequest::Project { .. })
    ) && play_backend.is_none()
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "project startup requires an App-owned embedded Play backend",
        )
        .into());
    }
    let hub_startup_reporter = HubStartupReporter::new(hub_handshake);
    let product_frame_evidence_requested = first_presented_frame_capture_path.is_some();
    let ui = UiHostWindow::new().map_err(|error| hub_startup_reporter.report_failure(error))?;
    ui.set_exit_after_first_presented_frame(exit_after_first_presented_frame);
    ui.set_first_presented_frame_capture_path(first_presented_frame_capture_path);
    let mut retained_host = match RetainedEditorHost::new(
        core,
        runtime_gateway,
        ui.clone_strong(),
        startup_request,
        project_runtime_build_set,
        hub_startup_reporter.session(),
    ) {
        Ok(retained_host) => retained_host,
        Err(error) => {
            if error
                .downcast_ref::<EditorError>()
                .and_then(EditorError::hub_focus_forwarded_process_id)
                .is_some()
            {
                return Err(hub_startup_reporter.report_failure_with_code(
                    HubEditorStartupFailureCodeV1::FocusInboxBinding,
                    error,
                ));
            }
            return Err(hub_startup_reporter.report_failure(error));
        }
    };
    if let Some(play_backend) = play_backend {
        retained_host.runtime.set_play_backend(play_backend);
    }
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

    let hub_focus_host = Rc::downgrade(&host);
    ui.on_native_window_focused(move || {
        let Some(host) = hub_focus_host.upgrade() else {
            return;
        };
        if let Err(error) = host.borrow().acknowledge_hub_window_focus() {
            eprintln!(
                "[zircon_editor] failed to publish owner-confirmed Hub focus acknowledgement: {error}"
            );
        }
    })
    .map_err(|error| {
        hub_startup_reporter
            .report_failure_with_code(HubEditorStartupFailureCodeV1::FocusInboxBinding, error)
    })?;

    host.borrow_mut().refresh_ui();
    host.borrow_mut()
        .sync_hub_focus_binding()
        .map_err(|error| {
            hub_startup_reporter
                .report_failure_with_code(HubEditorStartupFailureCodeV1::FocusInboxBinding, error)
        })?;
    let hub_focus_target = host
        .borrow()
        .editor_manager
        .active_project_session_focus_target();
    let hub_ready_receipt = if hub_startup_reporter.is_pending() {
        hub_focus_target
            .as_ref()
            .map(|(_, instance_id, session_generation)| {
                HubEditorReadyReceiptV1::after_first_present(
                    std::process::id(),
                    instance_id,
                    *session_generation,
                )
            })
            .transpose()
            .map_err(|error| hub_startup_reporter.report_failure(error))?
    } else {
        None
    };
    if hub_startup_reporter.is_pending() && !host.borrow().has_hub_focus_binding() {
        let error = std::io::Error::other(
            "Hub launch reached retained host startup without an active project session focus watcher",
        );
        return Err(hub_startup_reporter
            .report_failure_with_code(HubEditorStartupFailureCodeV1::FocusInboxBinding, error)
            .into());
    }
    if let Some(receipt) = hub_ready_receipt {
        let reporter = hub_startup_reporter.clone();
        ui.on_first_presented(move || match reporter.report_ready(receipt) {
            Ok(()) => Ok(()),
            Err(error) => Err(reporter
                .report_failure_with_code(HubEditorStartupFailureCodeV1::MailboxPublish, error)
                .to_string()),
        })
        .map_err(|error| hub_startup_reporter.report_failure(error))?;
    }
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
    host.borrow_mut().shutdown_runtime_session();
    let (final_autosave_requests, final_autosave_preparation_error) = match {
        let retained_host = host.borrow();
        retained_host.final_autosave_requests()
    } {
        Ok(requests) => (requests, None),
        Err(error) => (Vec::new(), Some(error)),
    };
    let autosave_shutdown = {
        let retained_host = host.borrow();
        retained_host
            .editor_manager
            .context()
            .autosave()
            .shutdown_with_final_autosave(
                final_autosave_requests,
                std::time::Instant::now() + std::time::Duration::from_secs(5),
            )
    };
    let settings_mutations = {
        let retained_host = host.borrow();
        Arc::clone(retained_host.editor_manager.context().settings_mutations())
    };
    let settings_shutdown = match settings_mutations.flush_then_shutdown() {
        Ok(closeout) => closeout,
        Err(error) => {
            let guard = settings_mutations.shutdown();
            drop(guard);
            return Err(error.into());
        }
    };
    let settings_shutdown_result = settings_shutdown.finish();
    if let Some(error) = ui.take_fatal_failure() {
        return Err(hub_startup_reporter
            .report_failure_with_code(HubEditorStartupFailureCodeV1::HostWindow, error)
            .into());
    }
    run_result.map_err(|error| {
        hub_startup_reporter
            .report_failure_with_code(HubEditorStartupFailureCodeV1::HostWindow, error)
    })?;
    settings_shutdown_result?;
    if let Some(error) = final_autosave_preparation_error {
        return Err(std::io::Error::other(format!(
            "failed to prepare final autosave requests: {error}"
        ))
        .into());
    }
    if !autosave_shutdown.diagnostic_persistence_issues().is_empty() {
        let issues = autosave_shutdown
            .diagnostic_persistence_issues()
            .iter()
            .map(|issue| {
                format!(
                    "{} in {}: {}",
                    issue.document().as_str(),
                    issue.project_root().display(),
                    issue.message()
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        return Err(std::io::Error::other(format!(
            "final autosave diagnostic persistence failed: {issues}"
        ))
        .into());
    }
    let incomplete_outcomes = autosave_shutdown
        .outcomes()
        .iter()
        .filter(|outcome| outcome.failure_stage().is_some())
        .map(|outcome| {
            format!(
                "{} ({})",
                outcome.document().as_str(),
                outcome
                    .failure_stage()
                    .expect("filtered autosave outcome has a failure stage")
            )
        })
        .collect::<Vec<_>>();
    if !incomplete_outcomes.is_empty() {
        return Err(std::io::Error::other(format!(
            "final autosave did not reach a saved terminal outcome: {}",
            incomplete_outcomes.join(", ")
        ))
        .into());
    }
    if !autosave_shutdown.unfinished_jobs().is_empty() {
        let jobs = autosave_shutdown
            .unfinished_jobs()
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
    // A clean final autosave, settings flush, event-loop exit, and capture
    // verification are required before this normal path removes the
    // OS-backed project admission lease.
    let project_close_result = host
        .borrow_mut()
        .commit_project_close()
        .map_err(std::io::Error::other);
    project_close_result?;
    Ok(())
}

/// Holds the Hub handshake unresolved until the host reaches its complete startup gate.
///
/// A mailbox failure never replaces the startup error: the local error remains authoritative for
/// the editor process, while the best-effort mailbox outcome prevents Hub from waiting silently.
#[derive(Clone)]
struct HubStartupReporter {
    handshake: Rc<RefCell<Option<HubEditorHandshake>>>,
}

impl HubStartupReporter {
    fn new(handshake: Option<HubEditorHandshake>) -> Self {
        Self {
            handshake: Rc::new(RefCell::new(handshake)),
        }
    }

    fn is_pending(&self) -> bool {
        self.handshake.borrow().is_some()
    }

    fn session(&self) -> Option<zircon_runtime_interface::hub_protocol::HubSessionToken> {
        self.handshake
            .borrow()
            .as_ref()
            .map(HubEditorHandshake::session)
    }

    fn report_failure<E>(&self, error: E) -> E
    where
        E: Display,
    {
        self.report_failure_with_code(HubEditorStartupFailureCodeV1::Startup, error)
    }

    fn report_failure_with_code<E>(&self, code: HubEditorStartupFailureCodeV1, error: E) -> E
    where
        E: Display,
    {
        if let Some(handshake) = self.handshake.borrow_mut().take() {
            if let Err(mailbox_error) = handshake.publish_failed(code) {
                eprintln!(
                    "[zircon_editor] failed to publish Hub startup failure while handling `{error}`: {mailbox_error}"
                );
            }
        }
        error
    }

    fn report_ready(&self, receipt: HubEditorReadyReceiptV1) -> Result<(), HubHandshakeError> {
        let Some(handshake) = self.handshake.borrow().clone() else {
            return Ok(());
        };
        handshake.publish_ready(receipt)?;
        self.handshake.borrow_mut().take();
        Ok(())
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
    runtime_shutdown_receipt: Option<EditorRuntimeSessionShutdownReceipt>,
    editor_manager: Arc<EditorManager>,
    hub_focus_binding: HubFocusBinding,
    hub_focus_request_attention: Arc<dyn Fn() + Send + Sync>,
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
    pending_active_scene_reload: Option<assets::PendingActiveSceneReload>,
    active_scene_reload_admission: Option<assets::ActiveSceneReloadAdmissionState>,
    active_scene_reload_conflict: Option<assets::ActiveSceneReloadConflict>,
    active_scene_reload_decision_sequence: u64,
    pending_model_import: Option<assets::PendingModelImport>,
    pending_asset_deletion: Option<assets::PendingAssetDeletion>,
    pending_asset_relocation: Option<assets::PendingAssetRelocation>,
    startup_session: EditorStartupSessionDocument,
    welcome_project_probe: welcome_session::WelcomeProjectProbeState,
    viewport_size: UVec2,
    viewport_pointer_bridge: callback_dispatch::SharedViewportPointerBridge,
    play_preview_input_focus_active: bool,
    play_preview_view_focus_active: bool,
    play_viewport_pick: play_viewport_pick::PlayViewportPickConsumer,
    last_simulate_camera: Option<(
        crate::core::play::PlayInstanceId,
        zircon_runtime_interface::ZrRuntimeViewportCameraV1,
    )>,
    builtin_template_runtime: Arc<EditorUiHostRuntime>,
    plugin_template_generation: u64,
    plugin_template_capabilities: Vec<String>,
    template_bridge: callback_dispatch::BuiltinHostWindowTemplateBridge,
    workbench_window_bridge: callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    host_chrome_projection_cache:
        crate::ui::layouts::windows::workbench_host_window::HostChromeProjectionCache,
    console_pane_projection_cache: crate::ui::retained_host::ui::ConsolePaneProjectionCache,
    module_plugins_pane_projection_cache:
        crate::ui::retained_host::ui::ModulePluginsPaneProjectionCache,
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
    menu_pointer_layout: Arc<HostMenuPointerLayout>,
    welcome_recent_pointer_bridge: WelcomeRecentPointerBridge,
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
    project_close_coordinator: crate::ui::host::ProjectCloseCoordinator,
    pending_close_prompt: Option<close_prompt::PendingClosePrompt>,
    pending_document_save_all: bool,
    queued_document_save_all: bool,
    scene_picker_session: Option<scene_picker_session::ScenePickerSession>,
    invalidation: HostInvalidationRoot,
    pending_ui_perf_scenario: Option<UiPerfScenario>,
    pending_activity_projection_refresh: bool,
    runtime_diagnostics_refresh_target: RuntimeDiagnosticsRefreshTarget,
    presentation_dirty: bool,
    layout_dirty: bool,
    window_metrics_dirty: bool,
    render_dirty: bool,
}

impl Drop for RetainedEditorHost {
    fn drop(&mut self) {
        self.editor_manager.context().autosave().begin_shutdown();
        // Drop only retires local UI ownership. The explicit session coordinator owns remote
        // unwatch and preserves its `WorldSyncShutdownReceipt` for diagnostics.
        self.hierarchy_world_watch.take();
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
