#[path = "../state/tests/normal_catalog_prepared_project.rs"]
mod normal_catalog_prepared_project;
#[path = "../state/tests/shutdown_consumer.rs"]
mod shutdown_consumer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use zircon_runtime_interface::world_sync::{AssetReloadFrameApplyReportDto, WorldFact};

use crate::core::LifecycleState;
use crate::diagnostic_log::DynamicProcessLogLease;
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
fn astra_life_a4_process_log_shutdown_consumes_remaining_deadline_and_retries() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless dynamic session");
    let released = Arc::new(AtomicBool::new(false));
    let released_for_shutdown = Arc::clone(&released);
    session.dynamic_process_log =
        Some(DynamicProcessLogLease::from_test_shutdown(move |timeout| {
            if released_for_shutdown.load(Ordering::Acquire) {
                return true;
            }
            std::thread::sleep(timeout);
            false
        }));

    let started_at = Instant::now();
    assert!(!session.shutdown_before_library_unload_with_timeout(Duration::from_millis(50)));
    assert!(started_at.elapsed() < Duration::from_secs(1));
    assert!(session.dynamic_process_log.is_some());

    released.store(true, Ordering::Release);
    assert!(session.shutdown_before_library_unload_with_timeout(Duration::from_secs(1)));
    assert!(session.dynamic_process_log.is_none());
}

#[test]
fn dynamic_session_records_the_activation_frame_clock_rebase_receipt() {
    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless dynamic session");

    assert_eq!(session.frame_clock_activation_rebase.generation(), 1);
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
