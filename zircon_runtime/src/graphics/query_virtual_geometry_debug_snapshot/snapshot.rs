#[cfg(feature = "test-support")]
use crate::core::framework::render::{
    RenderFrameExtract, RenderFramework, RenderViewportHandle, RenderVirtualGeometryReadbackOutputs,
};
use crate::core::framework::render::{RenderFrameworkError, RenderVirtualGeometryDebugSnapshot};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn query_virtual_geometry_debug_snapshot(
    framework: &WgpuRenderFramework,
) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
    framework.finish_submission()?;
    let state = framework.lock_state();
    let snapshot = state.last_virtual_geometry_debug_snapshot.clone();
    let mut physical = None;
    if let Some(original) = snapshot.as_ref() {
        let epoch = state.renderer.virtual_geometry_device_epoch();
        for record in state.viewports.values() {
            if let Some(segments) = record.physical_submission_segments(original, epoch) {
                if physical.is_some() {
                    return Err(RenderFrameworkError::Backend(
                        "physical submission snapshot has ambiguous viewport ownership".into(),
                    ));
                }
                physical = Some(segments.to_vec());
            }
        }
    }
    drop(state);
    let mut snapshot = snapshot.as_deref().cloned();
    if let (Some(snapshot), Some(segments)) = (snapshot.as_mut(), physical) {
        snapshot.submission_records = segments
            .iter()
            .map(
                |segment| crate::core::framework::render::RenderVirtualGeometrySubmissionRecord {
                    instance_index: segment.instance_index,
                    entity: segment.entity,
                    page_id: segment.page_id,
                    draw_ref_index: Some(segment.draw_ref_index),
                    submission_index: segment
                        .submission_index
                        .expect("validated terminal submission"),
                    draw_ref_rank: segment.draw_ref_rank.expect("validated terminal rank"),
                    original_index: segment.original_index,
                },
            )
            .collect();
        snapshot.execution_segments = segments;
    }
    Ok(snapshot)
}

pub(in crate::graphics::runtime::render_framework) fn query_virtual_geometry_debug_snapshot_available(
    framework: &WgpuRenderFramework,
) -> Result<bool, RenderFrameworkError> {
    framework.finish_submission()?;
    let state = framework.lock_state();
    Ok(state.last_virtual_geometry_debug_snapshot.is_some())
}

#[cfg(feature = "test-support")]
impl WgpuRenderFramework {
    pub fn test_support_virtual_geometry_native_probe_scope(
        &self,
        viewport: RenderViewportHandle,
    ) -> Result<Option<u64>, RenderFrameworkError> {
        let _operation = self.lock_operation();
        let mut state = self.lock_state();
        let super::super::render_framework_state::RenderFrameworkState {
            renderer,
            viewports,
            ..
        } = &mut *state;
        let record = viewports
            .get_mut(&viewport)
            .ok_or(RenderFrameworkError::UnknownViewport {
                viewport: viewport.raw(),
            })?;
        record.observe_virtual_geometry_readbacks(renderer);
        record.apply_ready_virtual_geometry_readbacks();
        let epoch = renderer.virtual_geometry_device_epoch();
        if let Some(error) = record.virtual_geometry_readback_error(epoch) {
            return Err(error);
        }
        Ok(record.native_probe_scope_id(epoch))
    }

    pub fn test_support_virtual_geometry_native_probe_source(
        &self,
        viewport: RenderViewportHandle,
        scope: u64,
        role: u32,
        drop_source: bool,
    ) -> Result<bool, String> {
        let _operation = self.lock_operation();
        let mut state = self.lock_state();
        let super::super::render_framework_state::RenderFrameworkState {
            renderer,
            viewports,
            ..
        } = &mut *state;
        let record = viewports
            .get_mut(&viewport)
            .ok_or("native probe viewport destroyed")?;
        record.observe_virtual_geometry_readbacks(renderer);
        record.apply_ready_virtual_geometry_readbacks();
        let epoch = renderer.virtual_geometry_device_epoch();
        if let Some(error) = record.virtual_geometry_readback_error(epoch) {
            return Err(error.to_string());
        }
        record
            .native_probe_handle(scope, epoch)?
            .native_probe_source(role, drop_source)
    }

    pub fn test_support_read_virtual_geometry_native_probe(
        &self,
        viewport: RenderViewportHandle,
        scope: u64,
        kind: u32,
    ) -> Result<Vec<u8>, String> {
        let _operation = self.lock_operation();
        let mut state = self.lock_state();
        let super::super::render_framework_state::RenderFrameworkState {
            renderer,
            viewports,
            ..
        } = &mut *state;
        let record = viewports
            .get_mut(&viewport)
            .ok_or("native probe viewport destroyed")?;
        record.observe_virtual_geometry_readbacks(renderer);
        record.apply_ready_virtual_geometry_readbacks();
        let epoch = renderer.virtual_geometry_device_epoch();
        if let Some(error) = record.virtual_geometry_readback_error(epoch) {
            return Err(error.to_string());
        }
        let handle = record.native_probe_handle(scope, epoch)?;
        renderer.read_virtual_geometry_native_probe(&handle, kind)
    }
    /// Submits one ordinary product frame through the public runtime framework contract, waits
    /// for the real submission to complete, and returns its renderer-produced debug snapshot.
    /// This is available only to tests and diagnostic support builds.
    pub fn test_support_submit_virtual_geometry_frame(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        RenderFramework::submit_frame_extract(self, viewport, extract)?;
        self.finish_submission()?;
        self.test_support_finish_virtual_geometry_readbacks(viewport)?;
        RenderFramework::query_virtual_geometry_debug_snapshot(self)
    }

    /// Submits one frame through the normal runtime collector/provider/feedback path while using
    /// the exact compiled graph supplied by the test. The override is one-shot and is removed
    /// even when preflight or submission fails.
    pub fn test_support_submit_compiled_virtual_geometry_frame(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        pipeline: &crate::graphics::CompiledRenderPipeline,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        self.test_support_submit_compiled_virtual_geometry_frame_with_inputs(
            viewport,
            extract,
            pipeline,
            Default::default(),
        )
    }

    pub fn test_support_submit_compiled_virtual_geometry_frame_with_inputs(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        pipeline: &crate::graphics::CompiledRenderPipeline,
        inputs: crate::core::framework::render::RenderVirtualGeometryExecutionInputs,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        self.finish_submission()?;
        let viewport_generation = self
            .lock_state()
            .viewports
            .get(&viewport)
            .ok_or(RenderFrameworkError::UnknownViewport {
                viewport: viewport.raw(),
            })?
            .generation();
        self.core.arm_test_virtual_geometry_override(
            viewport,
            viewport_generation,
            pipeline,
            inputs,
        )?;
        let submitted = RenderFramework::submit_frame_extract(self, viewport, extract);
        let finished = self.finish_submission();
        let _ = self.core.take_test_compiled_pipeline_override();
        submitted?;
        finished?;
        self.test_support_finish_virtual_geometry_readbacks(viewport)?;
        RenderFramework::query_virtual_geometry_debug_snapshot(self)
    }

    /// Waits only in explicit test support after a successful product submission.
    fn test_support_finish_virtual_geometry_readbacks(
        &self,
        viewport: RenderViewportHandle,
    ) -> Result<(), RenderFrameworkError> {
        let _operation = self.lock_operation();
        let deadline = web_time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            let mut state = self.lock_state();
            state
                .renderer
                .finish_virtual_geometry_native_readbacks_for_test()
                .map_err(
                    super::super::render_framework_backend_error::render_framework_backend_error,
                )?;
            let super::super::render_framework_state::RenderFrameworkState {
                renderer,
                viewports,
                ..
            } = &mut *state;
            let Some(record) = viewports.get_mut(&viewport) else {
                return Err(RenderFrameworkError::UnknownViewport {
                    viewport: viewport.raw(),
                });
            };
            record.observe_virtual_geometry_readbacks(renderer);
            record.apply_ready_virtual_geometry_readbacks();
            if let Some(error) =
                record.virtual_geometry_readback_error(renderer.virtual_geometry_device_epoch())
            {
                return Err(error);
            }
            if record.virtual_geometry_pending_readback_count() == 0 {
                return Ok(());
            }
            if web_time::Instant::now() >= deadline {
                return Err(RenderFrameworkError::UnsupportedCapability {
                    capability:
                        "virtual geometry native readback did not complete before test deadline"
                            .to_string(),
                });
            }
            drop(state);
            std::thread::yield_now();
        }
    }

    /// Returns the actual renderer readback payload captured immediately before the runtime
    /// provider consumes it. Available only to tests and diagnostic support builds.
    pub fn test_support_last_virtual_geometry_readback_outputs(
        &self,
    ) -> Result<Option<RenderVirtualGeometryReadbackOutputs>, RenderFrameworkError> {
        self.finish_submission()?;
        let state = self.lock_state();
        Ok(state
            .renderer
            .test_support_last_virtual_geometry_readback_outputs())
    }

    pub fn test_support_last_virtual_geometry_native_inspection(
        &self,
    ) -> Result<
        Option<crate::core::framework::render::RenderVirtualGeometryNativeReadbackInspection>,
        RenderFrameworkError,
    > {
        self.finish_submission()?;
        let state = self.lock_state();
        Ok(state
            .renderer
            .test_support_last_virtual_geometry_native_inspection())
    }

    pub fn test_support_last_virtual_geometry_gpu_completion(
        &self,
    ) -> Result<Option<crate::graphics::VirtualGeometryGpuCompletion>, RenderFrameworkError> {
        self.finish_submission()?;
        let state = self.lock_state();
        Ok(state
            .renderer
            .test_support_last_virtual_geometry_gpu_completion())
    }
}

#[cfg(test)]
#[path = "../../tests/query_virtual_geometry_debug_snapshot_snapshot.rs"]
mod tests;

#[cfg(all(test, feature = "native-host", feature = "test-support"))]
mod terminal_failure_tests {
    use super::*;
    use crate::asset::pipeline::manager::ProjectAssetManager;
    use crate::core::framework::render::{
        RenderFrameTiming, RenderQualityProfile, RenderViewportDescriptor,
        RenderVirtualGeometryExtract, RenderWorldSnapshotHandle,
    };
    use crate::core::math::UVec2;
    use crate::graphics::{
        RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
        RenderPassExecutionContext, RenderPassExecutorRegistration, RenderPassGpuRecordingContext,
        RenderPassGpuResourceFactory, RenderPassStage, VirtualGeometryRuntimeFeedback,
        VirtualGeometryRuntimePrepareInput, VirtualGeometryRuntimePrepareOutput,
        VirtualGeometryRuntimeProvider, VirtualGeometryRuntimeProviderRegistration,
        VirtualGeometryRuntimeState, VirtualGeometryRuntimeUpdate,
    };
    use crate::render_graph::QueueLane;
    use std::sync::Arc;

    impl WgpuRenderFramework {
        pub(in crate::graphics::runtime::render_framework) fn terminal_readback_test_fixture(
            updates: Arc<std::sync::atomic::AtomicUsize>,
        ) -> (Self, RenderViewportHandle) {
            framework_with_feedback_counter(updates)
        }

        pub(in crate::graphics::runtime::render_framework) fn terminal_readback_test_extract(
            mode: u64,
        ) -> RenderFrameExtract {
            extract(mode)
        }
    }

    #[derive(Debug)]
    struct EmptyProvider(Arc<std::sync::atomic::AtomicUsize>);
    #[derive(Debug)]
    struct EmptyState(Arc<std::sync::atomic::AtomicUsize>);
    impl VirtualGeometryRuntimeProvider for EmptyProvider {
        fn create_state(&self) -> Box<dyn VirtualGeometryRuntimeState> {
            Box::new(EmptyState(Arc::clone(&self.0)))
        }
    }
    impl VirtualGeometryRuntimeState for EmptyState {
        fn prepare_frame(
            &mut self,
            _: VirtualGeometryRuntimePrepareInput<'_>,
        ) -> VirtualGeometryRuntimePrepareOutput {
            Default::default()
        }
        fn update_after_render(
            &mut self,
            _: VirtualGeometryRuntimeFeedback,
        ) -> VirtualGeometryRuntimeUpdate {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Default::default()
        }
    }

    fn real_readback_executor(context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        let gpu = context.require_gpu()?;
        let mode = gpu.frame_extract().timing.outer_frame_index();
        let reservation = gpu
            .reserve_virtual_geometry_readback(16)?
            .ok_or("test requires an actual current-frame native reservation")?;
        let staging = {
            let native = gpu.native_context();
            let buffer =
                native
                    .resource_factory()
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("vg-public-framework-rejected-native-readback"),
                        contents: &[0_u8; 16],
                        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    });
            native.encoder.clear_buffer(&buffer, 0, None);
            buffer
        };
        gpu.enqueue_reserved_virtual_geometry_readback_with_inspection(
            reservation,
            staging,
            0..16,
            Box::new(move |bytes| {
                if bytes != [0_u8; 16] {
                    return Err("unexpected physical test bytes".into());
                }
                if mode == 1 {
                    Err("public terminal decoder rejection".into())
                } else {
                    Ok(Default::default())
                }
            }),
            if mode == 2 {
                Some(Box::new(|_, _| {
                    Err("public inspection decoder rejection".into())
                }))
            } else {
                None
            },
        )?;
        let retained = gpu.retain_virtual_geometry_native_probe(&[None], 16, |native, _, _| {
            let staging =
                native
                    .resource_factory()
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("vg-terminal-clear-only-native-probe"),
                        contents: &[0_u8; 16],
                        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    });
            native.command_encoder().clear_buffer(&staging, 0, None);
            Ok((staging, 0..16))
        })?;
        if !retained {
            return Err("actual clear-only native probe reservation required".into());
        }
        Ok(())
    }

    fn framework() -> (WgpuRenderFramework, RenderViewportHandle) {
        framework_with_feedback_counter(Arc::new(std::sync::atomic::AtomicUsize::new(0)))
    }

    fn framework_with_feedback_counter(
        updates: Arc<std::sync::atomic::AtomicUsize>,
    ) -> (WgpuRenderFramework, RenderViewportHandle) {
        let feature = RenderFeatureDescriptor::new(
            "virtual_geometry",
            vec!["view".into(), "geometry".into()],
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "vg-terminal-failure-test",
                QueueLane::Graphics,
            )
            .with_executor_id("vg.test.real-readback")
            .write_texture("scene-color")],
        )
        .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry);
        let framework = WgpuRenderFramework::new_for_test_with_plugin_render_features(
            Arc::new(ProjectAssetManager::default()),
            [feature],
            [RenderPassExecutorRegistration::new(
                "vg.test.real-readback",
                real_readback_executor,
            )],
            [VirtualGeometryRuntimeProviderRegistration::new(
                "vg.test.empty",
                Arc::new(EmptyProvider(updates)),
            )],
        )
        .unwrap();
        let viewport = framework
            .create_viewport(RenderViewportDescriptor::new(UVec2::new(16, 16)))
            .unwrap();
        framework
            .set_quality_profile(
                viewport,
                RenderQualityProfile::new("vg-native-terminal-failure")
                    .with_virtual_geometry(true)
                    .with_hybrid_global_illumination(false)
                    .with_clustered_lighting(false)
                    .with_screen_space_ambient_occlusion(false)
                    .with_temporal_history(false)
                    .with_bloom(false)
                    .with_color_grading(false)
                    .with_reflection_probes(false)
                    .with_baked_lighting(false)
                    .with_particle_rendering(false)
                    .with_async_compute(false),
            )
            .unwrap();
        (framework, viewport)
    }

    fn extract(mode: u64) -> RenderFrameExtract {
        let mut snapshot = crate::scene::world::World::new().to_render_snapshot();
        snapshot.scene.meshes.clear();
        snapshot.scene.directional_lights.clear();
        snapshot.scene.point_lights.clear();
        snapshot.scene.spot_lights.clear();
        snapshot.scene.ambient_lights.clear();
        snapshot.scene.rect_lights.clear();
        snapshot.preview.lighting_enabled = false;
        snapshot.preview.skybox_enabled = false;
        snapshot.environment = Default::default();
        let mut extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(71), snapshot)
                .with_viewport_size(UVec2::new(16, 16));
        extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract::default());
        extract.timing = RenderFrameTiming::new(mode, 0.0);
        extract
    }

    fn assert_failure(error: RenderFrameworkError, needle: &str) {
        match error {
            RenderFrameworkError::Backend(message) => {
                assert!(message.contains(needle), "{message}");
                for identity in [
                    "viewport=",
                    "viewport_generation=",
                    "frame_generation=",
                    "scope=",
                ] {
                    assert!(message.contains(identity), "{message}");
                }
            }
            other => panic!("expected original terminal failure, got {other:?}"),
        }
    }

    fn reject_and_check(mode: u64, needle: &str) {
        let (framework, viewport) = framework();
        assert_failure(
            framework
                .test_support_submit_virtual_geometry_frame(viewport, extract(mode))
                .unwrap_err(),
            needle,
        );
        assert_failure(framework.finish_submission().unwrap_err(), needle);
        assert_failure(
            framework
                .query_virtual_geometry_debug_snapshot()
                .unwrap_err(),
            needle,
        );
        assert_failure(
            framework
                .query_virtual_geometry_debug_snapshot_available()
                .unwrap_err(),
            needle,
        );
        assert_failure(
            framework
                .test_support_virtual_geometry_native_probe_scope(viewport)
                .unwrap_err(),
            needle,
        );
        assert_failure(
            framework
                .test_support_last_virtual_geometry_readback_outputs()
                .unwrap_err(),
            needle,
        );
        assert_failure(
            framework
                .test_support_last_virtual_geometry_native_inspection()
                .unwrap_err(),
            needle,
        );

        // Public submission accepts a replacement generation; do not call configuration
        // APIs that intentionally fence and report the previous terminal failure first.
        framework
            .submit_frame_extract(viewport, extract(0))
            .unwrap();
        // Finish the worker even if the old terminal diagnostic is still current.
        if let Err(error) = framework.finish_submission() {
            assert_failure(error, needle);
        }
        framework
            .test_support_finish_virtual_geometry_readbacks(viewport)
            .unwrap();
        framework.finish_submission().unwrap();
        framework.query_virtual_geometry_debug_snapshot().unwrap();
        assert!(
            framework
                .test_support_last_virtual_geometry_readback_outputs()
                .unwrap()
                .is_some(),
            "a successful clear-only payload is a real completion"
        );
        framework.destroy_viewport(viewport).unwrap();
    }

    #[test]
    fn public_framework_preserves_actual_terminal_decoder_failure_and_recovers() {
        reject_and_check(1, "public terminal decoder rejection");
    }
    #[test]
    fn public_framework_preserves_actual_inspection_decoder_failure_and_recovers() {
        reject_and_check(2, "public inspection decoder rejection");
    }

    #[test]
    fn current_terminal_failure_is_bounded_and_retires_with_viewport_generation() {
        let (framework, viewport) = framework();
        let healthy_viewport = framework
            .create_viewport(RenderViewportDescriptor::new(UVec2::new(16, 16)))
            .unwrap();
        for mode in [1, 2, 1] {
            framework
                .submit_frame_extract(viewport, extract(mode))
                .unwrap();
            if let Err(error) = framework.finish_submission() {
                let RenderFrameworkError::Backend(_) = error else {
                    panic!("{error:?}")
                };
            }
            assert!(framework
                .test_support_finish_virtual_geometry_readbacks(viewport)
                .is_err());
            let state = framework.lock_state();
            let record = state.viewports.get(&viewport).unwrap();
            assert_eq!(
                record.virtual_geometry_pending_readback_count(),
                1,
                "same-history terminal diagnostics must remain bounded"
            );
        }
        assert!(
            framework
                .test_support_virtual_geometry_native_probe_scope(healthy_viewport)
                .is_ok(),
            "a failed viewport must not poison another viewport's scoped query"
        );
        {
            let mut state = framework.lock_state();
            let epoch = state.renderer.virtual_geometry_device_epoch();
            let record = state.viewports.get_mut(&viewport).unwrap();
            assert!(record.virtual_geometry_readback_error(epoch).is_some());
            record.bump_generation();
            assert!(
                record.virtual_geometry_readback_error(epoch).is_none(),
                "previous viewport generation is no longer a current failure"
            );
        }
        framework.finish_submission().unwrap();
        framework.destroy_viewport(viewport).unwrap();
        framework.destroy_viewport(healthy_viewport).unwrap();
    }

    #[test]
    fn terminal_failure_is_not_reported_for_another_actual_device_epoch() {
        let (failed, viewport) = framework();
        assert_failure(
            failed
                .test_support_submit_virtual_geometry_frame(viewport, extract(1))
                .unwrap_err(),
            "public terminal decoder rejection",
        );
        let (other, other_viewport) = framework();
        let other_epoch = other.lock_state().renderer.virtual_geometry_device_epoch();
        {
            let mut state = failed.lock_state();
            let epoch = state.renderer.virtual_geometry_device_epoch();
            assert_ne!(
                epoch, other_epoch,
                "two real devices must have distinct epochs"
            );
            let record = state.viewports.get_mut(&viewport).unwrap();
            assert!(record.virtual_geometry_readback_error(epoch).is_some());
            assert!(record
                .virtual_geometry_readback_error(other_epoch)
                .is_none());
            record.bump_generation();
        }
        failed.destroy_viewport(viewport).unwrap();
        other.destroy_viewport(other_viewport).unwrap();
    }
    #[test]
    fn scoped_getters_observe_actual_callback_failure_without_prior_record_observation() {
        for getter in 0..3 {
            let (framework, viewport) = framework();
            let healthy = framework
                .create_viewport(RenderViewportDescriptor::new(UVec2::new(16, 16)))
                .unwrap();
            framework
                .submit_frame_extract(viewport, extract(1))
                .unwrap();
            framework.drain_submissions().unwrap();
            framework
                .lock_state()
                .renderer
                .finish_virtual_geometry_native_readbacks_for_test()
                .unwrap();
            // No finish_submission/test-support wait has observed the viewport record.
            let error = match getter {
                0 => framework
                    .test_support_virtual_geometry_native_probe_scope(viewport)
                    .unwrap_err()
                    .to_string(),
                1 => framework
                    .test_support_virtual_geometry_native_probe_source(viewport, 0, 0, false)
                    .unwrap_err(),
                _ => framework
                    .test_support_read_virtual_geometry_native_probe(viewport, 0, 0)
                    .unwrap_err(),
            };
            assert!(
                error.contains("public terminal decoder rejection"),
                "{error}"
            );
            assert!(framework
                .test_support_virtual_geometry_native_probe_scope(healthy)
                .unwrap()
                .is_none());
            framework.destroy_viewport(viewport).unwrap();
            framework.destroy_viewport(healthy).unwrap();
        }
    }

    #[cfg(all(target_os = "windows", feature = "platform-winit"))]
    #[test]
    fn public_win32_rebind_retires_terminal_readback_and_old_extent() {
        use crate::core::framework::render::RenderViewportSurfaceDescriptor;
        use crate::rhi::RenderNativeSurfaceTarget;
        use winit::{
            application::ApplicationHandler,
            dpi::{PhysicalSize, Size},
            event::WindowEvent,
            event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
            platform::windows::EventLoopBuilderExtWindows,
            raw_window_handle::{HasWindowHandle, RawWindowHandle},
            window::{WindowAttributes, WindowId},
        };
        struct RebindApp(Arc<std::sync::atomic::AtomicBool>, std::time::Instant);
        impl ApplicationHandler for RebindApp {
            fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
                let window = event_loop
                    .create_window(
                        WindowAttributes::default()
                            .with_title("VG real rebind regression")
                            .with_visible(false)
                            .with_surface_size(Size::Physical(PhysicalSize::new(32, 32))),
                    )
                    .expect("create actual Win32 surface regression window");
                let RawWindowHandle::Win32(handle) = window.window_handle().unwrap().as_raw()
                else {
                    panic!("Win32 window required");
                };
                let target = RenderNativeSurfaceTarget::Win32 {
                    hwnd: handle.hwnd.get() as usize as u64,
                    hinstance: handle.hinstance.map(|value| value.get() as usize as u64),
                };
                let (framework, viewport) = framework();
                framework
                    .bind_viewport_surface(
                        viewport,
                        RenderViewportSurfaceDescriptor::new(UVec2::new(16, 16), target),
                    )
                    .unwrap();
                assert_failure(
                    framework
                        .test_support_submit_virtual_geometry_frame(viewport, extract(1))
                        .unwrap_err(),
                    "public terminal decoder rejection",
                );
                let old_generation = framework
                    .lock_state()
                    .viewports
                    .get(&viewport)
                    .unwrap()
                    .generation();
                assert_failure(
                    framework.finish_submission().unwrap_err(),
                    "public terminal decoder rejection",
                );
                framework
                    .bind_viewport_surface(
                        viewport,
                        RenderViewportSurfaceDescriptor::new(UVec2::new(32, 32), target),
                    )
                    .expect("readback error must not block safe public rebind");
                {
                    let state = framework.lock_state();
                    let record = state.viewports.get(&viewport).unwrap();
                    assert_ne!(record.generation(), old_generation);
                    assert_eq!(record.size(), UVec2::new(32, 32));
                    assert_eq!(record.virtual_geometry_pending_readback_count(), 0);
                    assert!(record
                        .virtual_geometry_readback_error(
                            state.renderer.virtual_geometry_device_epoch(),
                        )
                        .is_none());
                }
                assert!(framework
                    .test_support_virtual_geometry_native_probe_scope(viewport)
                    .unwrap()
                    .is_none());
                framework.finish_submission().unwrap();
                framework
                    .test_support_submit_virtual_geometry_frame(
                        viewport,
                        extract(0).with_viewport_size(UVec2::new(32, 32)),
                    )
                    .unwrap();
                framework.destroy_viewport(viewport).unwrap();
                drop(framework);
                drop(window);
                self.0.store(true, std::sync::atomic::Ordering::SeqCst);
                event_loop.exit();
            }
            fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
                if std::time::Instant::now() >= self.1 {
                    event_loop.exit();
                } else {
                    event_loop.set_control_flow(ControlFlow::WaitUntil(self.1));
                }
            }
            fn window_event(&mut self, _: &dyn ActiveEventLoop, _: WindowId, _: WindowEvent) {}
        }
        let finished = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut builder = EventLoop::builder();
        builder.with_any_thread(true);
        builder
            .build()
            .expect("create actual Win32 event loop")
            .run_app(RebindApp(
                Arc::clone(&finished),
                std::time::Instant::now() + std::time::Duration::from_secs(30),
            ))
            .unwrap();
        assert!(
            finished.load(std::sync::atomic::Ordering::SeqCst),
            "actual surface creation callback must run; absence is a failure"
        );
    }
}
