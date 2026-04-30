use std::sync::Arc;

use crate::core::framework::render::{
    CapturedFrame, RenderFrameExtract, RenderFramework, RenderFrameworkError, RenderPipelineHandle,
    RenderQualityProfile, RenderStats, RenderViewportDescriptor, RenderViewportHandle,
    RenderVirtualGeometryDebugSnapshot,
};
use crate::core::{
    CoreRuntime, ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceObject, StartupMode,
};
use crate::engine_module::factory;
use crate::graphics::RenderPipelineAsset;

const DIAGNOSTICS_TEST_MODULE: &str = "DiagnosticsTestModule";

#[test]
fn runtime_diagnostics_reports_missing_runtime_contracts_without_panicking() {
    let runtime = CoreRuntime::new();

    let snapshot = crate::core::diagnostics::collect_runtime_diagnostics(&runtime.handle());

    assert!(!snapshot.render.available);
    assert!(snapshot.render.stats.is_none());
    assert!(snapshot.render.error.is_some());
    assert!(!snapshot.physics.available);
    assert!(snapshot.physics.backend_status.is_none());
    assert!(snapshot.physics.error.is_some());
    assert!(!snapshot.animation.available);
    assert!(snapshot.animation.playback_settings.is_none());
    assert!(snapshot.animation.error.is_some());
}

#[test]
fn runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins() {
    let runtime = CoreRuntime::new();
    runtime.register_module(fake_render_module()).unwrap();
    runtime.activate_module(DIAGNOSTICS_TEST_MODULE).unwrap();

    let snapshot = crate::core::diagnostics::collect_runtime_diagnostics(&runtime.handle());

    assert!(snapshot.render.available);
    let render_stats = snapshot.render.stats.expect("render stats");
    assert_eq!(render_stats.active_viewports, 2);
    assert_eq!(render_stats.submitted_frames, 7);
    assert_eq!(
        render_stats.capabilities.backend_name,
        "diagnostics-test-renderer"
    );
    assert!(!snapshot.render.virtual_geometry_debug_available);
    assert!(snapshot.render.error.is_none());

    assert!(!snapshot.physics.available);
    assert!(snapshot.physics.backend_status.is_none());
    assert!(snapshot.physics.error.is_some());

    assert!(!snapshot.animation.available);
    assert!(snapshot.animation.playback_settings.is_none());
    assert!(snapshot.animation.error.is_some());
}

fn fake_render_module() -> ModuleDescriptor {
    ModuleDescriptor::new(
        DIAGNOSTICS_TEST_MODULE,
        "runtime diagnostics fake render services",
    )
    .with_manager(ManagerDescriptor::new(
        RegistryName::new(crate::core::manager::RENDER_FRAMEWORK_NAME).unwrap(),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| {
            Ok(
                Arc::new(crate::core::manager::RenderFrameworkHandle::new(Arc::new(
                    FakeRenderFramework,
                ))) as ServiceObject,
            )
        }),
    ))
}

struct FakeRenderFramework;

impl RenderFramework for FakeRenderFramework {
    fn create_viewport(
        &self,
        _descriptor: RenderViewportDescriptor,
    ) -> Result<RenderViewportHandle, RenderFrameworkError> {
        Ok(RenderViewportHandle::new(1))
    }

    fn destroy_viewport(
        &self,
        _viewport: RenderViewportHandle,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn submit_frame_extract(
        &self,
        _viewport: RenderViewportHandle,
        _extract: RenderFrameExtract,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn submit_frame_extract_with_ui(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        _ui: Option<crate::ui::surface::UiRenderExtract>,
    ) -> Result<(), RenderFrameworkError> {
        self.submit_frame_extract(viewport, extract)
    }

    fn set_pipeline_asset(
        &self,
        _viewport: RenderViewportHandle,
        _pipeline: RenderPipelineHandle,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn register_pipeline_asset(
        &self,
        pipeline: RenderPipelineAsset,
    ) -> Result<RenderPipelineHandle, RenderFrameworkError> {
        Ok(pipeline.handle)
    }

    fn reload_pipeline(&self, _pipeline: RenderPipelineHandle) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn query_stats(&self) -> Result<RenderStats, RenderFrameworkError> {
        Ok(RenderStats {
            active_viewports: 2,
            submitted_frames: 7,
            capabilities: crate::core::framework::render::RenderCapabilitySummary {
                backend_name: "diagnostics-test-renderer".to_string(),
                virtual_geometry_supported: true,
                hybrid_global_illumination_supported: true,
                ..Default::default()
            },
            ..Default::default()
        })
    }

    fn query_virtual_geometry_debug_snapshot(
        &self,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        Ok(None)
    }

    fn capture_frame(
        &self,
        _viewport: RenderViewportHandle,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        Ok(None)
    }

    fn set_quality_profile(
        &self,
        _viewport: RenderViewportHandle,
        _profile: RenderQualityProfile,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }
}
