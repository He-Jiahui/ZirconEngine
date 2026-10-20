use super::*;

#[test]
fn radiance_cache_bootstrap_requires_successful_gpu_observation() {
    let mut bootstrap = RadianceCacheBootstrapState::default();

    let initial = bootstrap.begin_submission(false);
    assert!(initial.uses_bootstrap_snapshot);
    assert_eq!(initial.revision, 1);
    assert!(bootstrap.begin_submission(false).uses_bootstrap_snapshot);

    bootstrap.confirm_submission(initial.revision);
    let stable = bootstrap.begin_submission(false);
    assert!(!stable.uses_bootstrap_snapshot);
    assert_eq!(stable.revision, initial.revision);
}

#[test]
fn radiance_cache_update_retries_as_bootstrap_until_latest_revision_is_observed() {
    let mut bootstrap = RadianceCacheBootstrapState::default();
    let initial = bootstrap.begin_submission(false);
    bootstrap.confirm_submission(initial.revision);

    let incremental = bootstrap.begin_submission(true);
    assert!(!incremental.uses_bootstrap_snapshot);
    assert_eq!(incremental.revision, 2);
    let retry = bootstrap.begin_submission(false);
    assert!(retry.uses_bootstrap_snapshot);
    assert_eq!(retry.revision, incremental.revision);

    bootstrap.confirm_submission(initial.revision);
    assert!(bootstrap.begin_submission(false).uses_bootstrap_snapshot);
    bootstrap.confirm_submission(incremental.revision);
    assert!(!bootstrap.begin_submission(false).uses_bootstrap_snapshot);
}

#[test]
fn radiance_cache_readback_observation_is_bounded_to_the_shared_frame_ring() {
    let capacity = RuntimePrepareCollectorContext::MAX_IN_FLIGHT_GPU_READBACK_FRAMES;

    assert!(can_enqueue_readback_observation(capacity.saturating_sub(1)));
    assert!(!can_enqueue_readback_observation(capacity));
    assert!(!can_enqueue_readback_observation(
        capacity.saturating_add(1)
    ));
}

#[test]
fn global_sdf_deferred_requests_rotate_across_bounded_batches() {
    let mut requests = [0_u32, 1, 2, 3, 4];
    let mut cursor = 0;

    rotate_global_sdf_build_requests(&mut requests, &mut cursor, 2);
    assert_eq!(requests, [0, 1, 2, 3, 4]);
    assert_eq!(cursor, 2);

    requests.sort_unstable();
    rotate_global_sdf_build_requests(&mut requests, &mut cursor, 2);
    assert_eq!(requests, [2, 3, 4, 0, 1]);
    assert_eq!(cursor, 4);

    requests.sort_unstable();
    rotate_global_sdf_build_requests(&mut requests, &mut cursor, 2);
    assert_eq!(requests, [4, 0, 1, 2, 3]);
    assert_eq!(cursor, 1);
}

#[test]
fn global_sdf_readback_backpressure_reports_deferred_work() {
    let stats = GlobalSdfGpuBuildStats::deferred_by_readback_backpressure(7);

    assert_eq!(stats.deferred_page_count, 7);
    assert_eq!(stats.dispatched_page_count, 0);
    assert_eq!(stats.transient_upload_byte_count, 0);
}

#[test]
fn new_hgi_gpu_work_requires_shared_readback_admission_before_dispatch() {
    let source = include_str!("../outputs/runtime_prepare_collector.rs");
    let admission_gate = ["if !context.", "gpu_work_admitted() {"].concat();
    let admission_gate = source
        .find(&admission_gate)
        .expect("runtime prepare must reject new GPU work without a shared readback slot");
    let global_sdf_dispatch = source
        .find("gpu_resources.dispatch_global_sdf_pages(")
        .expect("global SDF dispatch must remain explicit");
    let prepare_dispatch = source
        .find("gpu_resources.execute_prepare(")
        .expect("HGI prepare dispatch must remain explicit");

    assert!(admission_gate < global_sdf_dispatch);
    assert!(admission_gate < prepare_dispatch);
}

#[test]
fn hgi_prepare_uses_queue_free_upload_and_state_transaction_recorders() {
    let collector = include_str!("../outputs/runtime_prepare_collector.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();
    let params = include_str!("../renderer/gpu/execute/queue_params.rs");
    let radiance = include_str!("../gpu/dispatch_radiance_cache/dispatch.rs");

    assert!(collector.contains("context.gpu_recording_context()"));
    assert!(collector.contains("mut buffer_uploads"));
    assert!(collector.contains("mut frame_transactions"));
    assert!(!collector.contains("context.queue"));
    assert!(!collector.contains("context.device"));
    assert!(!collector.contains("context.encoder"));
    assert!(!collector.contains("expect(\"hybrid GI GPU resources"));
    assert!(!params.contains("queue.write_buffer"));
    assert!(!radiance.contains("queue.write_buffer"));
}

#[test]
fn hgi_readback_futures_publish_only_after_frame_acceptance() {
    let source = include_str!("../outputs/runtime_prepare_collector.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();
    let global_enqueue = source
        .find("let future = pending.enqueue(context)?;")
        .expect("Global SDF readback enqueue");
    let global_publish = source[global_enqueue..]
        .find("register_global_sdf_readback_frame_commit(")
        .map(|offset| global_enqueue + offset)
        .expect("Global SDF readback frame commit");
    let radiance_enqueue = source[global_publish..]
        .find("let future = pending_readback.enqueue(context)?;")
        .map(|offset| global_publish + offset)
        .expect("radiance-cache readback enqueue");
    let radiance_publish = source[radiance_enqueue..]
        .find("register_radiance_cache_readback_frame_commit(")
        .map(|offset| radiance_enqueue + offset)
        .expect("radiance-cache readback frame commit");

    assert!(global_enqueue < global_publish);
    assert!(radiance_enqueue < radiance_publish);
    assert!(source.contains("hybrid-gi.global-sdf-readback"));
    assert!(source.contains("hybrid-gi.radiance-cache-readback"));
    assert!(source.contains(".saturating_add(staged_readback_count)"));
    assert_eq!(
        source
            .matches("context.register_frame_transaction(RuntimePrepareFrameTransaction::new(")
            .count(),
        2
    );
}

#[test]
fn hgi_gpu_dispatches_publish_named_shared_timer_scopes() {
    let source = include_str!("../outputs/runtime_prepare_collector.rs");
    let global_scope = ["context.begin_gpu_pass(", "GLOBAL_SDF_BUILD_PROFILE_NAME"].concat();
    let prepare_scope = ["context.begin_gpu_pass(", "HGI_PREPARE_PROFILE_NAME"].concat();
    let end_scope = ["context.end_gpu_", "pass("].concat();
    let global_dispatch = source
        .find("gpu_resources.dispatch_global_sdf_pages(")
        .expect("Global SDF dispatch must remain explicit");
    let prepare_dispatch = source
        .find("gpu_resources.execute_prepare(")
        .expect("radiance-cache prepare dispatch must remain explicit");
    let global_scope = source
        .find(&global_scope)
        .expect("Global SDF dispatch must open a shared timer scope");
    let prepare_scope = source
        .find(&prepare_scope)
        .expect("radiance-cache prepare dispatch must open a shared timer scope");
    let global_end = source[global_dispatch..]
        .find(&end_scope)
        .map(|offset| global_dispatch + offset)
        .expect("Global SDF dispatch must close its shared timer scope");
    let prepare_end = source[prepare_dispatch..]
        .find(&end_scope)
        .map(|offset| prepare_dispatch + offset)
        .expect("radiance-cache prepare dispatch must close its shared timer scope");

    assert!(source.contains("runtime_prepare.hybrid_gi.global_sdf_build"));
    assert!(source.contains("runtime_prepare.hybrid_gi.prepare"));
    assert!(global_scope < global_dispatch);
    assert!(global_dispatch < global_end);
    assert!(prepare_scope < prepare_dispatch);
    assert!(prepare_dispatch < prepare_end);
}

#[test]
fn empty_global_sdf_dispatch_closes_its_scope_without_publishing_a_profile() {
    let source = include_str!("../outputs/runtime_prepare_collector.rs");
    let global_dispatch = source
        .find("gpu_resources.dispatch_global_sdf_pages(")
        .expect("Global SDF dispatch must remain explicit");
    let dispatch_profile = source[global_dispatch..]
        .find("if dispatch.encoded_gpu_work() {")
        .map(|offset| global_dispatch + offset)
        .expect("Global SDF profile must be conditional on encoded GPU work");
    let discard_scope = source[dispatch_profile..]
        .find("context.discard_gpu_pass(gpu_pass);")
        .map(|offset| dispatch_profile + offset)
        .expect("empty Global SDF dispatch must close its timer scope");
    let stats = source[global_dispatch..]
        .find("global_sdf_build_stats = dispatch.stats();")
        .map(|offset| global_dispatch + offset)
        .expect("Global SDF dispatch statistics must remain available");

    assert!(global_dispatch < dispatch_profile);
    assert!(dispatch_profile < discard_scope);
    assert!(discard_scope < stats);
}

#[test]
fn radiance_cache_native_pending_readbacks_remain_with_their_original_instance_after_source_rotation(
) {
    use crate::hybrid_gi::hybrid_gi_scene_prepare_material_fixtures::{
        material_capture_test_assets, model_handle,
    };
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;
    use zircon_runtime::asset::pipeline::manager::ProjectAssetManagerAccess;
    use zircon_runtime::core::framework::render::*;
    use zircon_runtime::core::framework::scene::Mobility;
    use zircon_runtime::core::manager::{manager_service_handle, RegisteredManagerService};
    use zircon_runtime::core::math::{Transform, UVec2, Vec4};
    use zircon_runtime::core::runtime::ServiceObject;
    use zircon_runtime::core::{
        CoreRuntime, ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceKind, StartupMode,
    };
    use zircon_runtime::graphics::{RuntimePrepareCollectorRegistration, WgpuRenderFramework};

    #[derive(Default)]
    struct Observation {
        old_observed: bool,
        new_observed: bool,
        current_id: u64,
        old_radiance: Option<[u8; 4]>,
    }
    struct ObservedCollector {
        inner: HybridGiRuntimePrepareCollector,
        old_id: AtomicU64,
        observation: Arc<Mutex<Observation>>,
    }
    impl RuntimePrepareCollector for ObservedCollector {
        fn requests_gpu_readback(&self) -> bool {
            true
        }
        fn collect(
            &self,
            context: &mut RuntimePrepareCollectorContext<'_>,
        ) -> Result<RenderPluginRendererOutputs, GraphicsError> {
            let id = context
                .prepared_runtime_sidebands()
                .hybrid_gi_prepared_frame()
                .expect("real provider prepared frame")
                .radiance_cache_instance_id;
            assert_ne!(id, 0);
            let _ = self
                .old_id
                .compare_exchange(0, id, Ordering::Relaxed, Ordering::Relaxed);
            let old_id = self.old_id.load(Ordering::Relaxed);
            let mut observed = self.observation.lock().unwrap();
            if id == old_id {
                drop(observed);
                return self.inner.collect(context);
            }
            observed.current_id = id;
            let before = {
                let state = self.inner.lock_state()?;
                let old = state
                    .radiance_cache_instances
                    .get(&old_id)
                    .expect("old actual GPU instance retained");
                if !observed.old_observed {
                    assert!(
                        !old.pending_readbacks.is_empty(),
                        "old accepted frame must publish a real pending readback"
                    );
                }
                old.pending_readbacks.len()
            };
            let outputs = self.inner.collect(context)?;
            let mut state = self.inner.lock_state()?;
            let old = state.radiance_cache_instances.get_mut(&old_id).unwrap();
            assert_eq!(
                old.pending_readbacks.len(),
                before,
                "new instance collection must leave the old queue untouched"
            );
            if !observed.old_observed
                && old
                    .pending_readbacks
                    .front()
                    .is_some_and(|pending| pending.future.is_ready())
            {
                let readback = old.collect_ready_readbacks()?.expect(
                    "the real old GPU completion remains collectible only through its old instance",
                );
                let old_outputs = plugin_renderer_outputs_from_gpu_readback(Some(readback), None);
                assert!(!old_outputs
                    .hybrid_gi
                    .scene_prepare
                    .surface_cache_pages
                    .is_empty());
                assert!(old_outputs
                    .hybrid_gi
                    .scene_prepare
                    .surface_cache_pages
                    .iter()
                    .all(|page| page.owner_card_id == 17));
                assert!(old_outputs
                    .hybrid_gi
                    .scene_prepare
                    .surface_cache_pages
                    .iter()
                    .any(|page| page.radiance_rgba8[3] != 0));
                assert!(!old_outputs
                    .hybrid_gi
                    .scene_prepare
                    .voxel_clipmaps
                    .is_empty());
                observed.old_radiance =
                    Some(old_outputs.hybrid_gi.scene_prepare.surface_cache_pages[0].radiance_rgba8);
                observed.old_observed = true;
            }
            if !outputs
                .hybrid_gi
                .scene_prepare
                .surface_cache_pages
                .is_empty()
            {
                assert!(
                    outputs
                        .hybrid_gi
                        .scene_prepare
                        .surface_cache_pages
                        .iter()
                        .all(|page| page.owner_card_id == 29),
                    "new outputs may not contain old owner-card metadata"
                );
                assert!(outputs
                    .hybrid_gi
                    .scene_prepare
                    .surface_cache_pages
                    .iter()
                    .any(|page| page.radiance_rgba8[3] != 0));
                assert!(!outputs.hybrid_gi.scene_prepare.voxel_clipmaps.is_empty());
                if let Some(old_radiance) = observed.old_radiance {
                    assert!(
                        outputs
                            .hybrid_gi
                            .scene_prepare
                            .surface_cache_pages
                            .iter()
                            .all(|page| page.radiance_rgba8 != old_radiance),
                        "new black source must not receive old emissive radiance"
                    );
                }
                observed.new_observed = true;
            }
            Ok(outputs)
        }
    }

    let deadline = Instant::now() + Duration::from_secs(30);
    let (assets, _fixture_root, black, emissive) = material_capture_test_assets();
    let model = model_handle(&assets);
    // This is the existing real project fixture, including normal model import/cook.
    // Keep its manager runtime alive until after framework/readback destruction.
    let asset_runtime = CoreRuntime::new();
    asset_runtime
        .register_module(
            ModuleDescriptor::new("GiIsolationAssets", "GI isolation asset fixture").with_manager(
                ManagerDescriptor::new(
                    RegistryName::from_parts(
                        "GiIsolationAssets",
                        ServiceKind::Manager,
                        "ProjectAssetManager",
                    ),
                    StartupMode::Immediate,
                    vec![],
                    Arc::new(move |_| {
                        Ok(Arc::new(RegisteredManagerService::new(Arc::clone(&assets)))
                            as ServiceObject)
                    }),
                ),
            ),
        )
        .expect("real asset module registration");
    asset_runtime
        .activate_module("GiIsolationAssets")
        .expect("real asset module activation");
    let core = asset_runtime.handle();
    let service =
        manager_service_handle(&core, "GiIsolationAssets.Manager.ProjectAssetManager").unwrap();
    let observation = Arc::new(Mutex::new(Observation::default()));
    let collector = Arc::new(ObservedCollector {
        inner: HybridGiRuntimePrepareCollector::default(),
        old_id: AtomicU64::new(0),
        observation: Arc::clone(&observation),
    });
    let server = WgpuRenderFramework::new_with_plugin_render_extensions(
        ProjectAssetManagerAccess::new(core, service),
        [crate::render_feature_descriptor()],
        crate::render_pass_executor_registrations(),
        [RuntimePrepareCollectorRegistration::new_collector(
            HGI_RUNTIME_PREPARE_EXECUTOR_ID,
            collector.clone(),
        )],
        [crate::hybrid_gi_runtime_provider_registration()],
        vec![],
        asset_runtime.task_graph().worker_pool().clone(),
    )
    .expect("native isolation regression requires an actual GPU framework");
    let size = UVec2::new(32, 32);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(size))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("hgi-native-instance-isolation")
                .with_virtual_geometry(false)
                .with_hybrid_global_illumination(true)
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
    let extract = |node_id, material| {
        let mut camera = ViewportCameraSnapshot::default();
        camera.transform.translation = Vec3::new(0.0, 0.0, 4.0);
        camera.apply_viewport_size(size);
        let transform = Transform::default();
        let scene = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera,
                meshes: vec![RenderMeshSnapshot {
                    node_id,
                    stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
                    transform_revision: render_mesh_transform_revision(&transform),
                    transform,
                    model: model.clone(),
                    mesh: None,
                    material,
                    mesh_lod: None,
                    morph_weights: vec![],
                    tint: Vec4::ONE,
                    mobility: Mobility::Static,
                    static_state: RenderMeshStaticState::new(true, 1, 1),
                    common: RendererCommon {
                        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                        is_static: true,
                        ..RendererCommon::default()
                    },
                }],
                directional_lights: vec![],
                point_lights: vec![],
                spot_lights: vec![],
                ambient_lights: vec![],
                rect_lights: vec![],
            },
            overlays: RenderOverlayExtract::default(),
            environment: EnvironmentExtract::disabled(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        let mut frame = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(71).with_generation(1),
            scene,
        );
        frame.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            trace_budget: 1,
            card_budget: 1,
            voxel_budget: 1,
            ..Default::default()
        });
        frame
    };
    server
        .submit_frame_extract(viewport, extract(17, emissive))
        .unwrap();
    let new_source = extract(29, black);
    for _ in 0..RuntimePrepareCollectorContext::MAX_IN_FLIGHT_GPU_READBACK_FRAMES * 32 {
        assert!(
            Instant::now() < deadline,
            "real collector isolation deadline exceeded"
        );
        server
            .submit_frame_extract(viewport, new_source.clone())
            .unwrap();
        let observed = observation.lock().unwrap();
        if observed.old_observed && observed.new_observed {
            break;
        }
        drop(observed);
        std::thread::yield_now();
    }
    let observed = observation.lock().unwrap();
    assert!(
        observed.old_observed,
        "must positively collect actual old native GPU data from its own queue"
    );
    assert!(
        observed.new_observed,
        "must observe actual current-source GPU metadata and samples"
    );
    assert_ne!(
        observed.current_id,
        collector.old_id.load(Ordering::Relaxed)
    );
}
