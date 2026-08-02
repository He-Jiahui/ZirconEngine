use std::sync::Arc;

use zircon_runtime_interface::{
    ui::accessibility::UiAccessibilityTreeSnapshot, RuntimeInputDiagnosticsSnapshot,
    ZrRuntimeAccessibilityTreeRequestV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1,
    ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use crate::core::framework::input::{InputEvent, InputManager};
use crate::core::framework::render::RenderViewportSurfaceDescriptor;
use crate::core::manager::{resolve_manager_service, ManagerServiceHandle};
use crate::core::math::{UVec2, Vec2};
use crate::core::CoreRuntime;
use crate::diagnostic_log::{
    write_diagnostic_store_snapshot, write_log, write_log_lazy, DiagnosticStoreLogSchedule,
};
use crate::operation::RuntimeOperationService;
use crate::plugin::RuntimePluginRegistrationReport;
use crate::runtime_diagnostics::collect_runtime_diagnostics;
use crate::scene::{
    DynamicSceneAssetReloadFrameApplyReport, DynamicSceneAssetReloadQueue, LevelSystem,
};

use super::super::camera_controller::RuntimeCameraController;
use super::super::frame::encode_frame;
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
use super::scene_asset_reload_diagnostics::record_scene_asset_reload_frame_report;
use super::{RuntimeDynamicSessionError, RuntimeDynamicSessionResult, DEFAULT_VIEWPORT};

const DYNAMIC_RUNTIME_DIAGNOSTIC_LOG_SCOPE: &str = "runtime_diagnostics";

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
    pub(super) next_plugin_event_subscription: u64,
    pub(super) plugin_event_subscriptions:
        std::collections::HashMap<u64, event_mirror::RuntimePluginEventSubscriptionState>,
    pub(super) operations: RuntimeOperationService,
}

impl Drop for RuntimeDynamicSession {
    fn drop(&mut self) {
        let core = self.runtime.handle();
        let Ok(handle) = crate::asset::project_asset_manager_handle(&core) else {
            return;
        };
        let Ok(manager) = resolve_manager_service(&core, handle) else {
            return;
        };
        manager.shutdown_project_watchers();
    }
}

impl RuntimeDynamicSession {
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
        animation_frame_demand(&self.level)
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

    pub(super) fn drain_host_requests(&mut self) -> ZrRuntimeHostRequestBatchV1 {
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
    ) -> RuntimeDynamicSessionResult<ZrRuntimeFrameV1> {
        let requested = UVec2::new(request.size.width.max(1), request.size.height.max(1));
        self.resize_viewport(requested);
        let extract = self.current_extract();
        let ui = self.current_ui_extract();
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
        let ui = self.current_ui_extract();
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
    ) -> RuntimeDynamicSessionResult<UiAccessibilityTreeSnapshot> {
        self.resize_viewport(UVec2::new(
            request.size.width.max(1),
            request.size.height.max(1),
        ));
        Ok(dynamic_preview_accessibility_snapshot())
    }
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
    use super::RuntimeInputDiagnostics;

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
}
