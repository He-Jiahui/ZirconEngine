use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::RenderSubmissionConfig;

use super::*;

#[test]
fn realtime_ibl_timing_drain_finishes_pipelined_submission_before_state_access() {
    let source = include_str!("../wgpu_render_framework.rs");
    let drain = source
        .split("pub fn take_realtime_ibl_gpu_timing_reports")
        .nth(1)
        .and_then(|source| {
            source
                .split("/// Invalidates one on-demand planar probe")
                .next()
        })
        .expect("realtime IBL timing drain");
    let finish_submission = drain
        .find("self.finish_submission()?;")
        .expect("timing drain must finish pending submission");
    let operation_lock = drain
        .find("let _operation_guard = self.lock_operation();")
        .expect("timing drain must serialize renderer access");

    assert!(finish_submission < operation_lock);
}

#[test]
fn realtime_ibl_status_finishes_pipelined_submission_before_state_access() {
    let source = include_str!("../wgpu_render_framework.rs");
    let status = source
        .split("pub fn realtime_ibl_status_report")
        .nth(1)
        .and_then(|source| {
            source
                .split("/// Drains completed realtime IBL timestamp reports")
                .next()
        })
        .expect("realtime IBL status report");
    let finish_submission = status
        .find("self.finish_submission()?;")
        .expect("status report must finish pending submission");
    let operation_lock = status
        .find("let _operation_guard = self.lock_operation();")
        .expect("status report must serialize renderer access");

    assert!(finish_submission < operation_lock);
    assert!(!source.contains("pub fn realtime_ibl_failure_report"));
}

#[test]
fn realtime_ibl_cpu_timing_drain_finishes_pipelined_submission_before_state_access() {
    let source = include_str!("../wgpu_render_framework.rs");
    let drain = source
        .split("pub fn take_realtime_ibl_cpu_timing_reports")
        .nth(1)
        .and_then(|source| {
            source
                .split("/// Invalidates one on-demand planar probe")
                .next()
        })
        .expect("realtime IBL CPU timing drain");
    let finish_submission = drain
        .find("self.finish_submission()?;")
        .expect("CPU timing drain must finish pending submission");
    let operation_lock = drain
        .find("let _operation_guard = self.lock_operation();")
        .expect("CPU timing drain must serialize renderer access");

    assert!(finish_submission < operation_lock);
}

#[test]
fn device_admission_uses_the_shared_atomic_gate_without_renderer_state_access() {
    let source = include_str!("../wgpu_render_framework.rs");
    let admission = source
        .split("fn ensure_device_admission")
        .nth(1)
        .and_then(|source| source.split("fn lock_operation").next())
        .expect("framework core must retain device admission");

    assert!(admission.contains("self.device_fault_gate"));
    assert!(admission.contains(".ensure_admission()"));
    assert!(
        !admission.contains("self.lock_state()"),
        "the healthy frame admission path must not take the renderer-state lock"
    );
}

#[test]
fn frame_timing_drain_stays_inside_the_framework_lock_boundary() {
    let source = include_str!("../wgpu_render_framework.rs");
    let drain = source
        .split("pub fn take_completed_gpu_timing_report")
        .nth(1)
        .and_then(|source| source.split("/// Polls the specialized").next())
        .expect("frame timing drain");
    let finish_submission = drain
        .find("self.finish_submission()?;")
        .expect("timing drain must finish pending submission");
    let operation_lock = drain
        .find("let _operation_guard = self.lock_operation();")
        .expect("timing drain must serialize renderer access");
    let renderer_drain = drain
        .find(".take_completed_gpu_timing_report()")
        .expect("timing drain must consume the renderer-owned report");

    assert!(finish_submission < operation_lock && operation_lock < renderer_drain);
}

#[test]
fn frame_timing_report_attaches_only_a_same_generation_submission_snapshot() {
    let mesh_submission = crate::core::framework::render::RenderMeshSubmissionProfile {
        opaque_command_count: 2,
        advanced_pbr_opaque_command_count: 1,
        cached_command_hit_count: 4,
        command_rebuild_count: 0,
        dynamic_command_count: 1,
        ..Default::default()
    };
    let report = crate::graphics::SceneRendererGpuTimingReport::new(7, 1.0, []);
    let matching = RenderFrameProfile {
        frame_generation: 7,
        mesh_submission: mesh_submission.clone(),
        ..Default::default()
    };
    let mismatched = RenderFrameProfile {
        frame_generation: 8,
        mesh_submission,
        ..Default::default()
    };

    assert_eq!(
        attach_matching_mesh_submission_profile(report.clone(), Some(&matching))
            .mesh_submission_profile(),
        Some(&matching.mesh_submission)
    );
    assert_eq!(
        attach_matching_mesh_submission_profile(report, Some(&mismatched))
            .mesh_submission_profile(),
        None
    );
}

#[test]
fn wgpu_render_framework_accessors_recover_poisoned_locks() {
    let framework = WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default()))
        .expect("framework should initialize for lock recovery test");

    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _guard = framework.core.operation_lock.lock().unwrap();
        panic!("poison operation lock");
    }));
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _guard = framework.core.state.lock().unwrap();
        panic!("poison render framework state lock");
    }));

    drop(framework.lock_operation());
    assert!(framework.lock_state().viewports.is_empty());
}

#[test]
fn submission_config_switches_between_sync_and_pipelined_execution() {
    let framework = WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default()))
        .expect("framework should initialize for submission configuration test");

    assert_eq!(
        framework.submission_config(),
        RenderSubmissionConfig::synchronous()
    );
    assert!(!framework.lock_state().renderer.gpu_pass_timing_enabled());
    assert!(!framework
        .lock_state()
        .renderer
        .hzb_diagnostics_readback_enabled());
    assert_eq!(
        framework
            .lock_state()
            .renderer
            .parallel_record_min_passes_per_bucket(),
        None
    );
    let parallel_config = RenderSubmissionConfig::synchronous().with_parallel_recording(3);
    framework
        .set_submission_config(parallel_config)
        .expect("parallel recording configuration should reach the scene renderer");
    assert_eq!(framework.submission_config(), parallel_config);
    assert_eq!(
        framework
            .lock_state()
            .renderer
            .parallel_record_min_passes_per_bucket(),
        Some(3)
    );
    let timing_config = RenderSubmissionConfig::synchronous().with_gpu_timing();
    framework
        .set_submission_config(timing_config)
        .expect("GPU timing configuration should lazily create the timer when supported");
    assert_eq!(framework.submission_config(), timing_config);
    let state = framework.lock_state();
    assert_eq!(
        state.renderer.gpu_pass_timing_enabled(),
        state.stats.capabilities.supports_gpu_timestamp
    );
    drop(state);
    framework
        .set_submission_config(RenderSubmissionConfig::synchronous())
        .expect("disabling GPU timing should release the timer");
    assert!(!framework.lock_state().renderer.gpu_pass_timing_enabled());
    framework
        .set_submission_config(RenderSubmissionConfig::pipelined())
        .expect("pipelined configuration should initialize the worker");
    assert_eq!(
        framework.submission_config(),
        RenderSubmissionConfig::pipelined()
    );
    framework
        .set_submission_config(RenderSubmissionConfig::synchronous())
        .expect("synchronous configuration should drain and close the worker");

    let async_config = RenderSubmissionConfig::synchronous().with_async_pipeline_compile();
    framework
        .set_submission_config(async_config)
        .expect("async pipeline configuration should be accepted");
    assert!(framework
        .lock_state()
        .renderer
        .async_pipeline_compile_enabled());
    let hzb_readback_config = RenderSubmissionConfig::synchronous().with_hzb_diagnostics_readback();
    framework
        .set_submission_config(hzb_readback_config)
        .expect("HZB indirect readback configuration should be accepted");
    assert!(framework
        .lock_state()
        .renderer
        .hzb_diagnostics_readback_enabled());
}
