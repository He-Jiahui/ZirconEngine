use std::sync::Arc;
use std::time::{Duration, Instant};

use zircon_runtime_interface::world_sync::{
    AssetReloadFrameApplyReportDto, InvalidationBatch, WorldFact,
};
use zircon_runtime_interface::{
    ui::accessibility::UiAccessibilityTreeSnapshot, RuntimeInputDiagnosticsSnapshot,
    ZrRuntimeAccessibilityTreeRequestV1, ZrRuntimeFrameRequestV1, ZrRuntimeHostRequestBatchV1,
    ZrRuntimeHostRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_ACCESSIBILITY_TREE_OUTPUT_LIMIT_V1, ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1,
};

use crate::core::framework::channel::ChannelWakeCallback;
use crate::core::framework::input::{InputEvent, InputManager};
use crate::core::framework::render::RenderViewportSurfaceDescriptor;
use crate::core::manager::{resolve_manager_service, ManagerServiceHandle};
use crate::core::math::{UVec2, Vec2};
use crate::core::CoreRuntime;
use crate::diagnostic_log::{
    write_diagnostic_store_snapshot, write_log, write_log_lazy, DiagnosticStoreLogSchedule,
    DynamicProcessLogLease,
};
use crate::operation::RuntimeOperationService;
use crate::plugin::RuntimePluginRegistrationReport;
use crate::runtime_diagnostics::collect_runtime_diagnostics;
use crate::scene::{
    DynamicSceneAssetReloadFrameApplyReport, DynamicSceneAssetReloadQueue, LevelSystem,
};

use super::super::bounded_json::BoundedJsonError;
use super::super::camera_controller::RuntimeCameraController;
use super::super::frame::{encode_frame, encode_host_request_batch, EncodedRuntimeFrame};
use super::super::runtime_loop::RuntimeRenderBridge;
use super::construction;
use super::event_mirror;
use super::host_requests::{
    runtime_cursor_host_request, runtime_gamepad_rumble_request, runtime_ime_host_request,
};
use super::preview::{dynamic_preview_accessibility_snapshot, empty_captured_frame};
use super::profile::RuntimeDynamicSessionProfile;
use super::project::RuntimeProjectConfig;
use super::registry::RuntimeFrameDemand;
use super::runtime_ui::RuntimeUiSurfaceSet;
use super::scene_asset_reload_diagnostics::record_scene_asset_reload_frame_report;
use super::{RuntimeDynamicSessionError, RuntimeDynamicSessionResult, DEFAULT_VIEWPORT};

const DYNAMIC_RUNTIME_DIAGNOSTIC_LOG_SCOPE: &str = "runtime_diagnostics";
const DYNAMIC_SESSION_DESTROY_DRAIN_TIMEOUT: Duration = Duration::ZERO;

#[derive(Default)]
pub(super) struct RuntimeInputDiagnostics {
    viewport_resize_count: u64,
    pointer_move_count: u64,
    mouse_button_press_count: u64,
    mouse_button_release_count: u64,
    keyboard_press_count: u64,
    keyboard_release_count: u64,
}

impl RuntimeInputDiagnostics {
    fn record_viewport_resize(&mut self) {
        self.viewport_resize_count = self.viewport_resize_count.saturating_add(1);
    }

    fn record_pointer_move(&mut self) {
        self.pointer_move_count = self.pointer_move_count.saturating_add(1);
    }

    fn record_mouse_button_press(&mut self) {
        self.mouse_button_press_count = self.mouse_button_press_count.saturating_add(1);
    }

    fn record_mouse_button_release(&mut self) {
        self.mouse_button_release_count = self.mouse_button_release_count.saturating_add(1);
    }

    fn record_keyboard_press(&mut self) {
        self.keyboard_press_count = self.keyboard_press_count.saturating_add(1);
    }

    fn record_keyboard_release(&mut self) {
        self.keyboard_release_count = self.keyboard_release_count.saturating_add(1);
    }

    pub(super) fn snapshot(&self) -> RuntimeInputDiagnosticsSnapshot {
        RuntimeInputDiagnosticsSnapshot {
            viewport_resize_count: self.viewport_resize_count,
            pointer_move_count: self.pointer_move_count,
            mouse_button_press_count: self.mouse_button_press_count,
            mouse_button_release_count: self.mouse_button_release_count,
            keyboard_press_count: self.keyboard_press_count,
            keyboard_release_count: self.keyboard_release_count,
        }
    }
}

pub(super) struct RuntimeDynamicSession {
    pub(super) runtime: CoreRuntime,
    pub(super) profile: RuntimeDynamicSessionProfile,
    pub(super) diagnostic_log_schedule: DiagnosticStoreLogSchedule,
    pub(super) render_bridge: Option<RuntimeRenderBridge>,
    pub(super) level: LevelSystem,
    pub(super) scene_asset_reload_queue: Option<DynamicSceneAssetReloadQueue>,
    pub(super) last_scene_asset_reload_report: Option<DynamicSceneAssetReloadFrameApplyReport>,
    pub(super) project_identity: Option<String>,
    pub(super) scene_uri: Option<String>,
    pub(super) selected_model_resource_id: Option<String>,
    pub(super) selected_material_resource_id: Option<String>,
    pub(super) camera_controller: RuntimeCameraController,
    pub(super) extract_cache: super::extract_cache::RuntimeFrameExtractCache,
    pub(super) cursor: Vec2,
    pub(super) input_manager: ManagerServiceHandle<dyn InputManager>,
    pub(super) input_diagnostics: RuntimeInputDiagnostics,
    pub(super) pending_host_request_output: Option<ZrRuntimeHostRequestBatchV1>,
    pub(super) host_request_output_commit_count: usize,
    pub(super) host_request_output_in_flight: bool,
    pub(super) pending_world_invalidation_output: Option<Vec<InvalidationBatch>>,
    pub(super) world_invalidation_output_page: Option<Vec<InvalidationBatch>>,
    pub(super) world_invalidation_output_in_flight: bool,
    pub(super) next_plugin_event_subscription: u64,
    pub(super) plugin_event_subscriptions:
        std::collections::HashMap<u64, event_mirror::RuntimePluginEventSubscriptionState>,
    pub(super) operations: RuntimeOperationService,
    pub(super) project_watchers_shutdown: bool,
    pub(super) dynamic_process_log: Option<DynamicProcessLogLease>,
    pub(super) runtime_ui: RuntimeUiSurfaceSet,
}

impl Drop for RuntimeDynamicSession {
    fn drop(&mut self) {
        let _ = self.shutdown_before_library_unload();
    }
}

impl RuntimeDynamicSession {
    pub(super) fn with_dynamic_process_log_lease(
        mut self,
        dynamic_process_log: DynamicProcessLogLease,
    ) -> Self {
        self.dynamic_process_log = Some(dynamic_process_log);
        self
    }

    pub(super) fn with_runtime_frame_wake(mut self, wake: ChannelWakeCallback) -> Self {
        if let Some(queue) = self.scene_asset_reload_queue.as_mut() {
            queue.install_runtime_frame_wake(wake);
        }
        self
    }

    pub(super) fn shutdown_before_library_unload(&mut self) -> bool {
        let event_mirrors_shutdown = self.shutdown_plugin_event_subscriptions();
        if !event_mirrors_shutdown {
            return false;
        }
        if !self.project_watchers_shutdown {
            // Watch callbacks may still emit diagnostics, so they must stop before the final lease.
            let core = self.runtime.handle();
            if let Ok(handle) = crate::asset::project_asset_manager_handle(&core) {
                if let Ok(manager) = resolve_manager_service(&core, handle) {
                    manager.shutdown_project_watchers();
                }
            }
            self.project_watchers_shutdown = true;
        }
        if self
            .runtime
            .shutdown_registered_modules_with_drain_timeout(DYNAMIC_SESSION_DESTROY_DRAIN_TIMEOUT)
            .is_err()
        {
            return false;
        }
        let process_log_shutdown = if let Some(process_log) = self.dynamic_process_log.as_mut() {
            let shutdown = process_log.shutdown();
            if shutdown {
                self.dynamic_process_log = None;
            }
            shutdown
        } else {
            true
        };
        process_log_shutdown
    }

    pub(super) fn new(
        profile: RuntimeDynamicSessionProfile,
        project_config: Option<RuntimeProjectConfig>,
    ) -> RuntimeDynamicSessionResult<Self> {
        construction::build(profile, project_config, Vec::new())
    }

    pub(super) fn new_with_linked_plugins(
        profile: RuntimeDynamicSessionProfile,
        project_config: Option<RuntimeProjectConfig>,
        linked_plugin_registrations: Vec<RuntimePluginRegistrationReport>,
    ) -> RuntimeDynamicSessionResult<Self> {
        construction::build(profile, project_config, linked_plugin_registrations)
    }

    pub(super) fn tick_frame(&mut self) -> RuntimeDynamicSessionResult<()> {
        let advance = {
            crate::profile_scope!("runtime", "frame", "runtime_frame_time_update");
            self.runtime
                .tick_time(self.profile.max_fixed_steps_per_frame())
        };
        self.tick_scene_asset_reload();
        {
            crate::profile_scope!("runtime", "frame", "runtime_frame_update");
            self.level
                .tick(&self.runtime.handle(), advance)
                .map_err(|source| RuntimeDynamicSessionError::CoreStep {
                    step: "tick loaded level",
                    source,
                })?;
        }
        {
            crate::profile_scope!("runtime", "frame", "runtime_operation_owner_apply");
            let core = self.runtime.handle();
            let operations = &self.operations;
            self.level
                .with_world_mut(|world| operations.tick(&core, world));
        }
        self.resolve_input_manager()
            .map_err(|source| RuntimeDynamicSessionError::CoreStep {
                step: "resolve input for frame",
                source,
            })?
            .begin_frame();
        if self.diagnostic_log_schedule.tick(advance.real_delta()) {
            let snapshot = collect_runtime_diagnostics(&self.runtime.handle()).store;
            write_diagnostic_store_snapshot(DYNAMIC_RUNTIME_DIAGNOSTIC_LOG_SCOPE, &snapshot);
        }
        Ok(())
    }

    pub(super) fn frame_demand(&self) -> RuntimeFrameDemand {
        asset_reload_frame_demand(
            self.scene_asset_reload_queue
                .as_ref()
                .is_some_and(DynamicSceneAssetReloadQueue::has_pending_work),
        )
        .unwrap_or_else(|| animation_frame_demand(&self.level))
    }

    pub(super) fn reset_frame_demand_after_failed_tick(&self) {
        self.level.record_animation_requires_continuous_frame(false);
    }

    fn tick_scene_asset_reload(&mut self) {
        let Some(queue) = &mut self.scene_asset_reload_queue else {
            self.last_scene_asset_reload_report = None;
            return;
        };
        let report = queue.tick_into_level(self.runtime.handle().scheduler(), &self.level);
        if let Some(fact) = asset_reload_world_fact(&report) {
            self.level.record_world_fact(fact);
        }
        record_scene_asset_reload_frame_report(&self.runtime, &report);
        if report.events_drained() > 0
            || report.applied_count() > 0
            || report.failed_count() > 0
            || report.stale_count() > 0
            || report.superseded_pending_count() > 0
        {
            write_log_lazy("runtime_session", || {
                format!(
                    "runtime_scene_asset_reload_frame drained={} scheduled={} applied={} failed={} stale={} superseded={} pending={}",
                    report.events_drained(),
                    report.scheduled_count(),
                    report.applied_count(),
                    report.failed_count(),
                    report.stale_count(),
                    report.superseded_pending_count(),
                    report.pending_count()
                )
            });
        }
        self.last_scene_asset_reload_report = Some(report);
    }

    pub(super) fn prepare_host_request_output(&mut self) -> Result<Vec<u8>, BoundedJsonError> {
        if self.host_request_output_in_flight {
            return Err(BoundedJsonError::Json(
                "runtime host request output is already in flight".to_string(),
            ));
        }
        if self.pending_host_request_output.is_none() {
            self.pending_host_request_output = Some(self.collect_host_requests());
        }
        let pending = self
            .pending_host_request_output
            .as_ref()
            .expect("pending host request output was initialized");
        let page_limit = pending
            .requests
            .len()
            .min(ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1.max_items);
        let (bytes, commit_count) = if page_limit == 0 {
            (Vec::new(), 0)
        } else {
            encode_largest_host_request_prefix(pending, page_limit)?
        };
        self.host_request_output_commit_count = commit_count;
        self.host_request_output_in_flight = true;
        Ok(bytes)
    }

    pub(super) fn commit_host_request_output(&mut self) {
        debug_assert!(self.host_request_output_in_flight);
        let pending = self
            .pending_host_request_output
            .as_mut()
            .expect("an in-flight host request output must retain its pending batch");
        pending
            .requests
            .drain(..self.host_request_output_commit_count);
        if pending.requests.is_empty() {
            self.pending_host_request_output = None;
        }
        self.host_request_output_commit_count = 0;
        self.host_request_output_in_flight = false;
    }

    pub(super) fn rollback_host_request_output(&mut self) {
        debug_assert!(self.host_request_output_in_flight);
        self.host_request_output_commit_count = 0;
        self.host_request_output_in_flight = false;
    }

    fn collect_host_requests(&mut self) -> ZrRuntimeHostRequestBatchV1 {
        let input_manager = match self.resolve_input_manager() {
            Ok(input_manager) => input_manager,
            Err(error) => {
                write_log_lazy("runtime_session", || {
                    format!("runtime_input_manager_stale error={error}")
                });
                return ZrRuntimeHostRequestBatchV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
            }
        };
        let requests = input_manager
            .drain_ime_host_requests()
            .into_iter()
            .map(|request| runtime_ime_host_request(request, DEFAULT_VIEWPORT))
            .map(ZrRuntimeHostRequestV1::ime)
            .chain(
                input_manager
                    .drain_gamepad_rumble_requests()
                    .into_iter()
                    .map(runtime_gamepad_rumble_request)
                    .map(ZrRuntimeHostRequestV1::gamepad_rumble),
            )
            .chain(
                input_manager
                    .drain_cursor_host_requests()
                    .into_iter()
                    .map(runtime_cursor_host_request)
                    .map(ZrRuntimeHostRequestV1::cursor),
            )
            .collect();
        ZrRuntimeHostRequestBatchV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, requests)
    }

    pub(super) fn resolve_input_manager(
        &self,
    ) -> Result<Arc<dyn InputManager>, crate::core::CoreError> {
        resolve_manager_service(&self.runtime.handle(), self.input_manager.clone())
    }

    pub(super) fn submit_input_event(&self, event: InputEvent) -> bool {
        match self.resolve_input_manager() {
            Ok(input_manager) => {
                input_manager.submit_event(event);
                true
            }
            Err(error) => {
                write_log_lazy("runtime_session", || {
                    format!("runtime_input_event_dropped_for_stale_manager error={error}")
                });
                false
            }
        }
    }

    pub(super) fn record_submitted_pointer_move(&mut self) {
        self.input_diagnostics.record_pointer_move();
    }

    pub(super) fn record_viewport_resize(&mut self) {
        self.input_diagnostics.record_viewport_resize();
    }

    pub(super) fn record_submitted_mouse_button_press(&mut self) {
        self.input_diagnostics.record_mouse_button_press();
    }

    pub(super) fn record_submitted_mouse_button_release(&mut self) {
        self.input_diagnostics.record_mouse_button_release();
    }

    pub(super) fn record_submitted_keyboard_press(&mut self) {
        self.input_diagnostics.record_keyboard_press();
    }

    pub(super) fn record_submitted_keyboard_release(&mut self) {
        self.input_diagnostics.record_keyboard_release();
    }

    pub(super) fn capture_frame(
        &mut self,
        request: ZrRuntimeFrameRequestV1,
    ) -> RuntimeDynamicSessionResult<EncodedRuntimeFrame> {
        let requested = UVec2::new(request.size.width.max(1), request.size.height.max(1));
        self.resize_viewport(requested);
        let extract = self.current_extract();
        let ui = self.current_ui_extract()?;
        let frame = if let Some(render_bridge) = &mut self.render_bridge {
            render_bridge
                .submit_extract_with_ui(extract, self.camera_controller.viewport_size(), ui)
                .map_err(|source| RuntimeDynamicSessionError::RenderBridgeStep {
                    step: "submit render extract",
                    source,
                })?
                .unwrap_or_else(|| empty_captured_frame(requested))
        } else {
            empty_captured_frame(requested)
        };
        Ok(encode_frame(frame))
    }

    pub(super) fn bind_viewport_surface(
        &mut self,
        descriptor: RenderViewportSurfaceDescriptor,
    ) -> RuntimeDynamicSessionResult<()> {
        self.resize_viewport(descriptor.size);
        let Some(render_bridge) = &mut self.render_bridge else {
            return Ok(());
        };
        render_bridge.bind_surface(descriptor).map_err(|source| {
            RuntimeDynamicSessionError::RenderBridgeStep {
                step: "bind viewport surface",
                source,
            }
        })
    }

    pub(super) fn unbind_viewport_surface(&mut self) -> RuntimeDynamicSessionResult<()> {
        let Some(render_bridge) = &mut self.render_bridge else {
            return Ok(());
        };
        render_bridge.unbind_surface().map_err(|source| {
            RuntimeDynamicSessionError::RenderBridgeStep {
                step: "unbind viewport surface",
                source,
            }
        })
    }

    pub(super) fn present_viewport(
        &mut self,
        request: ZrRuntimeFrameRequestV1,
    ) -> RuntimeDynamicSessionResult<()> {
        let requested = UVec2::new(request.size.width.max(1), request.size.height.max(1));
        self.resize_viewport(requested);
        let extract = self.current_extract();
        let ui = self.current_ui_extract()?;
        let Some(render_bridge) = &mut self.render_bridge else {
            return Ok(());
        };
        render_bridge
            .present_extract_with_ui(extract, self.camera_controller.viewport_size(), ui)
            .map_err(|source| RuntimeDynamicSessionError::RenderBridgeStep {
                step: "present render extract",
                source,
            })
    }

    pub(super) fn capture_accessibility_tree(
        &mut self,
        request: ZrRuntimeAccessibilityTreeRequestV1,
    ) -> Result<UiAccessibilityTreeSnapshot, BoundedJsonError> {
        self.resize_viewport(UVec2::new(
            request.size.width.max(1),
            request.size.height.max(1),
        ));
        self.runtime_ui
            .accessibility_snapshot(
                self.camera_controller.viewport_size(),
                ZR_RUNTIME_ACCESSIBILITY_TREE_OUTPUT_LIMIT_V1,
            )
            .map(|snapshot| snapshot.unwrap_or_else(dynamic_preview_accessibility_snapshot))
    }
}

fn encode_largest_host_request_prefix(
    pending: &ZrRuntimeHostRequestBatchV1,
    page_limit: usize,
) -> Result<(Vec<u8>, usize), BoundedJsonError> {
    let started = Instant::now();
    let encode = |count: usize| -> Result<Vec<u8>, BoundedJsonError> {
        check_host_request_encoding_deadline(started)?;
        let bytes = encode_host_request_batch(&ZrRuntimeHostRequestBatchV1::new(
            pending.abi_version,
            pending.requests[..count].to_vec(),
        ))?;
        check_host_request_encoding_deadline(started)?;
        Ok(bytes)
    };
    match encode(page_limit) {
        Ok(bytes) => return Ok((bytes, page_limit)),
        Err(error) if !deterministic_host_request_failure(&error) => return Err(error),
        Err(_) => {}
    }

    let first = encode(1)?;
    let mut best = (first, 1_usize);
    let mut low = 2_usize;
    let mut high = page_limit;
    while low < high {
        let candidate = low + (high - low) / 2;
        match encode(candidate) {
            Ok(bytes) => {
                best = (bytes, candidate);
                low = candidate + 1;
            }
            Err(error) if deterministic_host_request_failure(&error) => high = candidate,
            Err(error) => return Err(error),
        }
    }
    Ok(best)
}

fn check_host_request_encoding_deadline(started: Instant) -> Result<(), BoundedJsonError> {
    let limit_micros = ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1.max_processing_time_micros;
    if started.elapsed() > Duration::from_micros(limit_micros) {
        return Err(BoundedJsonError::ProcessingTime { limit_micros });
    }
    Ok(())
}

fn deterministic_host_request_failure(error: &BoundedJsonError) -> bool {
    matches!(
        error,
        BoundedJsonError::EncodedBytes { .. }
            | BoundedJsonError::Items { .. }
            | BoundedJsonError::NestingDepth { .. }
    )
}

fn asset_reload_world_fact(report: &DynamicSceneAssetReloadFrameApplyReport) -> Option<WorldFact> {
    let has_activity = report.events_drained() > 0
        || report.applied_count() > 0
        || report.failed_count() > 0
        || report.stale_count() > 0
        || report.superseded_pending_count() > 0;
    has_activity.then(|| {
        WorldFact::AssetReloadApplied(AssetReloadFrameApplyReportDto {
            applied: report.applied_count() as u64,
            failed: report.failed_count() as u64,
            stale: report.stale_count() as u64,
            pending_count: report.pending_count() as u64,
        })
    })
}

fn asset_reload_frame_demand(has_pending_work: bool) -> Option<RuntimeFrameDemand> {
    has_pending_work.then_some(RuntimeFrameDemand::Immediate)
}

pub(super) fn animation_frame_demand(level: &LevelSystem) -> RuntimeFrameDemand {
    if level.animation_requires_continuous_frame() {
        RuntimeFrameDemand::Immediate
    } else {
        RuntimeFrameDemand::Idle
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::world_sync::{AssetReloadFrameApplyReportDto, WorldFact};

    use crate::core::LifecycleState;
    use crate::scene::DynamicSceneAssetReloadFrameApplyReport;

    use super::super::profile::RuntimeDynamicSessionProfile;
    use super::{
        asset_reload_frame_demand, asset_reload_world_fact, RuntimeDynamicSession,
        RuntimeInputDiagnostics,
    };

    #[test]
    fn dynamic_session_shutdown_runs_core_module_cleanup_before_library_unload() {
        let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
            .expect("headless dynamic session");
        let handle = session.runtime.handle();
        let running_modules = handle
            .inner
            .modules
            .lock()
            .expect("test module registry")
            .values()
            .filter(|entry| entry.lifecycle == LifecycleState::Running)
            .count();
        assert!(
            running_modules > 0,
            "dynamic session must own running core modules"
        );

        assert!(session.shutdown_before_library_unload());
        assert!(handle
            .inner
            .modules
            .lock()
            .expect("test module registry")
            .values()
            .all(|entry| entry.lifecycle == LifecycleState::Unloaded));
    }

    #[test]
    fn input_diagnostics_accumulate_successfully_submitted_product_events() {
        let mut diagnostics = RuntimeInputDiagnostics::default();

        diagnostics.record_viewport_resize();
        diagnostics.record_pointer_move();
        diagnostics.record_mouse_button_press();
        diagnostics.record_mouse_button_release();
        diagnostics.record_keyboard_press();
        diagnostics.record_keyboard_release();

        assert_eq!(
            diagnostics.snapshot(),
            zircon_runtime_interface::RuntimeInputDiagnosticsSnapshot {
                viewport_resize_count: 1,
                pointer_move_count: 1,
                mouse_button_press_count: 1,
                mouse_button_release_count: 1,
                keyboard_press_count: 1,
                keyboard_release_count: 1,
            }
        );
    }

    #[test]
    fn asset_reload_activity_maps_once_to_the_world_sync_fact_contract() {
        let mut report = DynamicSceneAssetReloadFrameApplyReport::default();
        assert_eq!(asset_reload_world_fact(&report), None);

        report.drain.events_drained = 1;
        report.apply.pending_count = 7;
        assert_eq!(
            asset_reload_world_fact(&report),
            Some(WorldFact::AssetReloadApplied(
                AssetReloadFrameApplyReportDto {
                    applied: 0,
                    failed: 0,
                    stale: 0,
                    pending_count: 7,
                }
            ))
        );
    }

    #[test]
    fn pending_asset_reload_work_keeps_the_reactive_loop_alive_until_completion() {
        assert_eq!(asset_reload_frame_demand(false), None);
        assert_eq!(
            asset_reload_frame_demand(true),
            Some(super::RuntimeFrameDemand::Immediate)
        );
        assert_eq!(asset_reload_frame_demand(false), None);
    }
}
