use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::framework::render::{
    CapturedFrame, RenderFrameExtract, RenderFramework, RenderFrameworkError, RenderPipelineHandle,
    RenderQualityProfile, RenderStats, RenderViewportDescriptor, RenderViewportHandle,
    RenderVirtualGeometryDebugSnapshot, UiRenderSubmission,
};
use crate::core::manager::{
    manager_service_handle, resolve_manager_service, RegisteredManagerService,
    RENDER_FRAMEWORK_NAME,
};
use crate::core::math::UVec2;
use crate::core::runtime::{ServiceEntryFactory, ServiceObject};
use crate::core::{
    CoreResult, LifecycleState, ModuleCleanupContext, ModuleContext, ModuleLifecycle, StartupMode,
};
use crate::text::{text_runtime_context_for_core, TextRuntimeContext, TEXT_MODULE_NAME};

use super::super::super::profile::RuntimeDynamicSessionProfile;
use super::super::{RuntimeDynamicSession, RuntimeDynamicSessionError, RuntimeRenderBridge};

#[test]
fn session_releases_its_text_families_before_core_drain_and_retries_external_lease() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless dynamic session");
    let core = session.runtime.handle();
    let text_token =
        manager_service_handle::<TextRuntimeContext>(&core, "TextModule.Manager.FontServices")
            .expect("real Text manager identity");
    let external_layout_session = {
        let context = text_runtime_context_for_core(&core).expect("normal Text call admission");
        let layout = context
            .create_layout_session()
            .expect("actual exported Text layout-family lease");
        assert!(
            context
                .health_snapshot()
                .active_layout_session_family_count()
                >= 2
        );
        layout
    };
    assert!(session.ui_extract_cache.is_some());

    assert!(!session.shutdown_before_library_unload_with_timeout(Duration::from_secs(1)));
    assert!(session.ui_extract_cache.is_none());
    assert_eq!(
        core.inner
            .modules
            .lock()
            .expect("actual module registry")
            .get(TEXT_MODULE_NAME)
            .expect("Text remains registered")
            .lifecycle,
        LifecycleState::Stopping
    );
    assert!(resolve_manager_service(&core, text_token.clone()).is_err());
    let world_tick_before = session.level.with_world(|world| world.read_change_tick());
    assert!(matches!(
        session.tick_frame(),
        Err(RuntimeDynamicSessionError::CoreStep {
            step: "tick stopped runtime session",
            ..
        })
    ));
    assert_eq!(
        session.level.with_world(|world| world.read_change_tick()),
        world_tick_before
    );
    assert!(matches!(
        session.current_ui_submission(),
        Err(RuntimeDynamicSessionError::CoreStep {
            step: "extract stopped runtime session UI",
            ..
        })
    ));

    drop(external_layout_session);
    assert!(session.shutdown_before_library_unload_with_timeout(Duration::from_secs(5)));
    assert!(core
        .inner
        .modules
        .lock()
        .expect("actual module registry")
        .values()
        .all(|entry| entry.lifecycle == LifecycleState::Unloaded));
    assert!(resolve_manager_service(&core, text_token).is_err());
    assert!(session.shutdown_before_library_unload_with_timeout(Duration::from_secs(1)));
}

#[derive(Default)]
struct ViewportObservations {
    fail_destroy: AtomicBool,
    creates: AtomicUsize,
    submissions: AtomicUsize,
    destroy_attempts: AtomicUsize,
    destroy_successes: AtomicUsize,
    cleanup_successes: AtomicUsize,
    surface_bindings:
        std::sync::Mutex<Vec<crate::core::framework::render::RenderViewportSurfaceDescriptor>>,
}

struct ControlledViewportFramework {
    observations: Arc<ViewportObservations>,
}

const CONTROLLED_VIEWPORT: RenderViewportHandle = RenderViewportHandle::new(71);

impl RenderFramework for ControlledViewportFramework {
    fn bind_viewport_surface(
        &self,
        viewport: RenderViewportHandle,
        descriptor: crate::core::framework::render::RenderViewportSurfaceDescriptor,
    ) -> Result<(), RenderFrameworkError> {
        assert_eq!(viewport, CONTROLLED_VIEWPORT);
        self.observations
            .surface_bindings
            .lock()
            .expect("binding observations")
            .push(descriptor);
        Ok(())
    }

    fn create_viewport(
        &self,
        _descriptor: RenderViewportDescriptor,
    ) -> Result<RenderViewportHandle, RenderFrameworkError> {
        assert_eq!(self.observations.creates.fetch_add(1, Ordering::SeqCst), 0);
        Ok(CONTROLLED_VIEWPORT)
    }

    fn destroy_viewport(&self, viewport: RenderViewportHandle) -> Result<(), RenderFrameworkError> {
        assert_eq!(viewport, CONTROLLED_VIEWPORT);
        self.observations
            .destroy_attempts
            .fetch_add(1, Ordering::SeqCst);
        if self.observations.fail_destroy.load(Ordering::SeqCst) {
            return Err(RenderFrameworkError::Backend(
                "controlled viewport release failure".to_owned(),
            ));
        }
        assert_eq!(
            self.observations
                .destroy_successes
                .fetch_add(1, Ordering::SeqCst),
            0
        );
        Ok(())
    }

    fn submit_frame_extract(
        &self,
        viewport: RenderViewportHandle,
        _extract: RenderFrameExtract,
    ) -> Result<(), RenderFrameworkError> {
        assert_eq!(viewport, CONTROLLED_VIEWPORT);
        self.observations.submissions.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn submit_frame_extract_with_ui(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        _ui: Option<Arc<UiRenderSubmission>>,
    ) -> Result<(), RenderFrameworkError> {
        self.submit_frame_extract(viewport, extract)
    }

    fn set_pipeline_asset(
        &self,
        _viewport: RenderViewportHandle,
        _pipeline: RenderPipelineHandle,
    ) -> Result<(), RenderFrameworkError> {
        Err(unsupported("pipeline asset"))
    }

    fn reload_pipeline(&self, _pipeline: RenderPipelineHandle) -> Result<(), RenderFrameworkError> {
        Err(unsupported("pipeline reload"))
    }

    fn query_stats(&self) -> Result<RenderStats, RenderFrameworkError> {
        Err(unsupported("render statistics"))
    }

    fn query_virtual_geometry_debug_snapshot(
        &self,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        Err(unsupported("virtual geometry"))
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
        Err(unsupported("quality profile"))
    }
}

fn unsupported(capability: &str) -> RenderFrameworkError {
    RenderFrameworkError::UnsupportedCapability {
        capability: capability.to_owned(),
    }
}

struct ObserveGraphicsCleanup {
    inner: Arc<dyn ModuleLifecycle>,
    observations: Arc<ViewportObservations>,
}

impl ModuleLifecycle for ObserveGraphicsCleanup {
    fn build(&self, context: &ModuleContext) -> CoreResult<()> {
        self.inner.build(context)
    }

    fn ready(&self, context: &ModuleContext) -> CoreResult<bool> {
        self.inner.ready(context)
    }

    fn finish(&self, context: &ModuleContext) -> CoreResult<()> {
        self.inner.finish(context)
    }

    fn cleanup(&self, context: &ModuleCleanupContext<'_>) -> CoreResult<()> {
        assert_eq!(
            self.observations.destroy_successes.load(Ordering::SeqCst),
            1
        );
        self.inner.cleanup(context)?;
        self.observations
            .cleanup_successes
            .fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn cleanup_until(
        &self,
        context: &ModuleCleanupContext<'_>,
        deadline: Instant,
    ) -> CoreResult<()> {
        assert_eq!(
            self.observations.destroy_successes.load(Ordering::SeqCst),
            1
        );
        self.inner.cleanup_until(context, deadline)?;
        self.observations
            .cleanup_successes
            .fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn viewport_surface_v2_clamp_preserves_requested_present_mode() {
    use crate::core::framework::render::RenderViewportSurfaceDescriptor;
    use crate::core::framework::window::WindowPresentMode as W;
    use crate::rhi::RenderNativeSurfaceTarget;
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless dynamic session");
    let core = session.runtime.handle();
    let observations = Arc::new(ViewportObservations::default());
    {
        let mut services = core.inner.services.lock().expect("actual service registry");
        let entry = services
            .get_mut(RENDER_FRAMEWORK_NAME)
            .expect("actual lazy graphics slot");
        assert_eq!(entry.lifecycle, LifecycleState::Registered);
        assert!(entry.instance.is_none());
        let factory_observations = Arc::clone(&observations);
        entry.factory = ServiceEntryFactory::Service(Arc::new(move |_| {
            let framework: Arc<dyn RenderFramework> = Arc::new(ControlledViewportFramework {
                observations: Arc::clone(&factory_observations),
            });
            Ok(Arc::new(RegisteredManagerService::new(framework)) as ServiceObject)
        }));
    }
    let mut bridge = RuntimeRenderBridge::new(&core).expect("actual manager binding");
    let modes = [
        W::Fifo,
        W::Immediate,
        W::Mailbox,
        W::FifoRelaxed,
        W::AutoVsync,
        W::AutoNoVsync,
    ];
    let target = RenderNativeSurfaceTarget::Win32 {
        hwnd: 73,
        hinstance: None,
    };
    for mode in modes {
        bridge
            .bind_surface(
                RenderViewportSurfaceDescriptor::new(UVec2::ZERO, target).with_present_mode(mode),
            )
            .expect("real bridge consumes descriptor");
    }
    let actual = observations
        .surface_bindings
        .lock()
        .expect("binding observations")
        .clone();
    assert_eq!(actual.len(), modes.len());
    for (descriptor, mode) in actual.iter().zip(modes) {
        assert_eq!(descriptor.size, UVec2::new(1, 1));
        assert_eq!(descriptor.target, target);
        assert_eq!(descriptor.present_mode, mode);
    }
    assert_eq!(observations.creates.load(Ordering::SeqCst), 1);
    session.render_bridge = Some(bridge);
    assert!(session.shutdown_before_library_unload_with_timeout(Duration::from_secs(5)));
    assert_eq!(observations.destroy_successes.load(Ordering::SeqCst), 1);
}

#[test]
fn viewport_release_failure_preserves_session_resources_and_core_until_retry() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless dynamic session");
    let core = session.runtime.handle();
    let observations = Arc::new(ViewportObservations::default());
    observations.fail_destroy.store(true, Ordering::SeqCst);
    {
        let mut services = core.inner.services.lock().expect("actual service registry");
        let entry = services
            .get_mut(RENDER_FRAMEWORK_NAME)
            .expect("actual lazy Graphics RenderFramework slot");
        assert_eq!(entry.startup_mode, StartupMode::Lazy);
        assert_eq!(entry.lifecycle, LifecycleState::Registered);
        assert!(entry.instance.is_none());
        assert!(entry.initialization_owner.is_none());
        assert!(!entry.admission_open);
        assert_eq!(entry.in_flight_calls, 0);
        let factory_observations = Arc::clone(&observations);
        entry.factory = ServiceEntryFactory::Service(Arc::new(move |_| {
            let framework: Arc<dyn RenderFramework> = Arc::new(ControlledViewportFramework {
                observations: Arc::clone(&factory_observations),
            });
            Ok(Arc::new(RegisteredManagerService::new(framework)) as ServiceObject)
        }));
    }
    {
        let mut modules = core.inner.modules.lock().expect("actual module registry");
        let entry = modules.get_mut("GraphicsModule").expect("Graphics module");
        assert_eq!(entry.lifecycle, LifecycleState::Running);
        entry.descriptor.lifecycle = Arc::new(ObserveGraphicsCleanup {
            inner: Arc::clone(&entry.descriptor.lifecycle),
            observations: Arc::clone(&observations),
        });
    }

    let mut bridge = RuntimeRenderBridge::new(&core).expect("normal registered manager resolution");
    assert!(bridge
        .submit_extract(session.current_extract(), UVec2::new(8, 8))
        .expect("create and submit the actual bridge viewport")
        .is_none());
    assert_eq!(observations.creates.load(Ordering::SeqCst), 1);
    assert_eq!(observations.submissions.load(Ordering::SeqCst), 1);
    session.render_bridge = Some(bridge);

    assert!(!session.shutdown_before_library_unload_with_timeout(Duration::from_secs(1)));
    assert!(session.render_bridge.is_some());
    assert!(session.ui_extract_cache.is_some());
    assert_eq!(observations.destroy_attempts.load(Ordering::SeqCst), 1);
    assert_eq!(observations.destroy_successes.load(Ordering::SeqCst), 0);
    assert_eq!(observations.cleanup_successes.load(Ordering::SeqCst), 0);
    assert_eq!(
        core.inner
            .modules
            .lock()
            .expect("actual module registry")
            .get("GraphicsModule")
            .expect("Graphics remains registered")
            .lifecycle,
        LifecycleState::Running
    );
    // A failed viewport release must precede Core admission closure.
    drop(
        core.resolve_manager::<RegisteredManagerService<dyn RenderFramework>>(
            RENDER_FRAMEWORK_NAME,
        )
        .expect("ordinary registered service still admits calls"),
    );

    observations.fail_destroy.store(false, Ordering::SeqCst);
    assert!(session.shutdown_before_library_unload_with_timeout(Duration::from_secs(5)));
    assert!(session.render_bridge.is_none());
    assert!(session.ui_extract_cache.is_none());
    assert_eq!(observations.destroy_attempts.load(Ordering::SeqCst), 2);
    assert_eq!(observations.destroy_successes.load(Ordering::SeqCst), 1);
    assert_eq!(observations.cleanup_successes.load(Ordering::SeqCst), 1);
    assert!(session.shutdown_before_library_unload_with_timeout(Duration::from_secs(1)));
    assert_eq!(observations.destroy_attempts.load(Ordering::SeqCst), 2);
    assert_eq!(observations.cleanup_successes.load(Ordering::SeqCst), 1);
}
