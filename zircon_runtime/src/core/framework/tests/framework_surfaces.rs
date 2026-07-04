use super::*;
use crate::core::framework::render::RenderLayerSet;

#[test]
fn time_framework_tracks_real_virtual_and_fixed_clocks() {
    let mut real = Time::<Real>::default();
    real.advance_by(Duration::from_millis(16));

    assert_eq!(real.delta(), Duration::from_millis(16));
    assert_eq!(real.elapsed(), Duration::from_millis(16));
    assert_eq!(real.frame_index(), 1);

    let mut virtual_time = Time::<Virtual>::default();
    virtual_time.advance_from_real_delta(Duration::from_millis(500));
    assert_eq!(virtual_time.delta(), Duration::from_millis(250));
    assert_eq!(virtual_time.elapsed(), Duration::from_millis(250));

    virtual_time.set_relative_speed_f64(0.5);
    virtual_time.advance_from_real_delta(Duration::from_millis(100));
    assert_eq!(virtual_time.delta(), Duration::from_millis(50));
    assert_eq!(virtual_time.elapsed(), Duration::from_millis(300));
    assert_eq!(virtual_time.effective_speed_f64(), 0.5);

    virtual_time.pause();
    virtual_time.advance_from_real_delta(Duration::from_millis(100));
    assert_eq!(virtual_time.delta(), Duration::ZERO);
    assert_eq!(virtual_time.elapsed(), Duration::from_millis(300));
    assert!(virtual_time.is_paused());
    assert_eq!(virtual_time.effective_speed_f64(), 0.0);

    let mut fixed = Time::<Fixed>::from_duration(Duration::from_millis(10));
    fixed.accumulate_overstep(Duration::from_millis(35));
    let plan = fixed.drain_steps(3);

    assert_eq!(plan.step_count, 3);
    assert_eq!(plan.consumed, Duration::from_millis(30));
    assert_eq!(plan.remaining_overstep, Duration::from_millis(5));
    assert_eq!(fixed.delta(), Duration::from_millis(10));
    assert_eq!(fixed.elapsed(), Duration::from_millis(30));
    assert_eq!(fixed.frame_index(), 3);
    assert_eq!(fixed.overstep(), Duration::from_millis(5));

    fixed.accumulate_overstep(Duration::from_millis(30));
    let capped = fixed.drain_steps(2);
    assert_eq!(capped.step_count, 2);
    assert_eq!(capped.remaining_overstep, Duration::from_millis(15));
    assert_eq!(fixed.elapsed(), Duration::from_millis(50));
    assert_eq!(fixed.frame_index(), 5);
}

#[test]
fn task_framework_contracts_describe_pools_status_and_poll_budget() {
    let compute = TaskPoolDescriptor::compute().with_worker_threads(0);
    let async_compute = TaskPoolDescriptor::async_compute().with_thread_name("async-streaming");
    let io = TaskPoolDescriptor::io();
    let handle = AsyncTaskHandle::new(42);
    let descriptor = AsyncTaskDescriptor::new(handle, TaskPoolKind::AsyncCompute, "mesh-import")
        .with_cancellation_policy(TaskCancellationPolicy::DetachOnDrop);

    assert_eq!(compute.kind, TaskPoolKind::Compute);
    assert_eq!(compute.worker_threads, Some(1));
    assert_eq!(async_compute.kind, TaskPoolKind::AsyncCompute);
    assert_eq!(async_compute.thread_name, "async-streaming");
    assert_eq!(io.thread_name, TaskPoolKind::Io.default_thread_name());
    assert_eq!(descriptor.handle.raw(), 42);
    assert_eq!(descriptor.pool, TaskPoolKind::AsyncCompute);
    assert_eq!(
        descriptor.cancellation_policy,
        TaskCancellationPolicy::DetachOnDrop
    );

    let mut status = AsyncTaskStatus::pending(handle);
    assert_eq!(status.state, AsyncTaskState::Pending);
    assert!(!status.is_terminal());

    status.mark_running();
    status.record_poll();
    status.record_poll();
    assert_eq!(status.state, AsyncTaskState::Running);
    assert_eq!(status.poll_count, 2);

    status.mark_failed("importer returned no artifact");
    assert_eq!(status.state, AsyncTaskState::Failed);
    assert!(status.is_terminal());
    assert_eq!(
        status.failure_message.as_deref(),
        Some("importer returned no artifact")
    );

    let budget = TaskPollBudget::default();
    assert_eq!(
        budget.remaining_after(40),
        Some(DEFAULT_MAIN_THREAD_POLLS_PER_FRAME - 40)
    );
    assert!(budget.is_exhausted_after(DEFAULT_MAIN_THREAD_POLLS_PER_FRAME));
    assert!(!TaskPollBudget::unlimited().is_exhausted_after(u32::MAX));
}

#[test]
fn task_framework_root_stays_structural_after_folder_split() {
    let tasks_mod = include_str!("../tasks/mod.rs");

    for required in [
        "mod async_task_descriptor;",
        "mod async_task_handle;",
        "mod async_task_state;",
        "mod async_task_status;",
        "mod task_cancellation_policy;",
        "mod task_poll_budget;",
        "mod task_pool_descriptor;",
        "mod task_pool_kind;",
        "AsyncTaskDescriptor",
        "AsyncTaskHandle",
        "AsyncTaskState",
        "AsyncTaskStatus",
        "TaskCancellationPolicy",
        "TaskPollBudget",
        "TaskPoolDescriptor",
        "TaskPoolKind",
    ] {
        assert!(
            tasks_mod.contains(required),
            "tasks framework root should keep structural export `{required}`"
        );
    }

    for forbidden in [
        "pub struct AsyncTaskDescriptor",
        "pub struct AsyncTaskStatus",
        "pub struct TaskPoolDescriptor",
        "pub enum AsyncTaskState",
        "pub enum TaskPoolKind",
        "impl AsyncTaskStatus",
        "impl TaskPoolDescriptor",
    ] {
        assert!(
            !tasks_mod.contains(forbidden),
            "tasks framework root should not keep implementation detail `{forbidden}`"
        );
    }
}

#[test]
fn time_framework_root_stays_structural_after_folder_split() {
    let time_mod = include_str!("../time/mod.rs");

    for required in [
        "mod clock;",
        "mod fixed;",
        "mod fixed_step_plan;",
        "mod real;",
        "mod virtual_clock;",
        "FixedStepPlan",
        "Fixed",
        "Real",
        "Time",
        "Virtual",
    ] {
        assert!(
            time_mod.contains(required),
            "time framework root should keep structural export `{required}`"
        );
    }

    for forbidden in [
        "pub struct Time",
        "pub struct FixedStepPlan",
        "pub struct Fixed",
        "pub struct Virtual",
        "impl Time<Virtual>",
        "impl Time<Fixed>",
    ] {
        assert!(
            !time_mod.contains(forbidden),
            "time framework root should not keep implementation detail `{forbidden}`"
        );
    }
}

#[test]
fn physics_framework_root_stays_structural_after_folder_split() {
    let physics_mod = include_str!("../physics/mod.rs");

    for required in [
        "mod backend_state;",
        "mod backend_status;",
        "mod body_sync_state;",
        "mod body_type;",
        "mod collider_shape;",
        "mod collider_sync_state;",
        "mod combine_rule;",
        "mod contact_event;",
        "mod joint_sync_state;",
        "mod joint_type;",
        "mod manager;",
        "mod material_metadata;",
        "mod material_sync_state;",
        "mod query_filter;",
        "mod ray_cast_hit;",
        "mod ray_cast_query;",
        "mod settings;",
        "mod shape_cast_hit;",
        "mod shape_cast_query;",
        "mod shape_overlap_hit;",
        "mod shape_overlap_query;",
        "mod simulation_mode;",
        "mod trigger_event;",
        "mod trigger_event_kind;",
        "mod world_step_plan;",
        "mod world_sync_state;",
        "PhysicsBackendState",
        "PhysicsBackendStatus",
        "PhysicsBodySyncState",
        "PhysicsBodyType",
        "PhysicsColliderShape",
        "PhysicsColliderSyncState",
        "PhysicsCombineRule",
        "PhysicsContactEvent",
        "PhysicsJointSyncState",
        "PhysicsJointType",
        "PhysicsManager",
        "PhysicsMaterialMetadata",
        "PhysicsMaterialSyncState",
        "PhysicsQueryFilter",
        "PhysicsRayCastHit",
        "PhysicsRayCastQuery",
        "PhysicsSettings",
        "PhysicsShapeCastHit",
        "PhysicsShapeCastQuery",
        "PhysicsShapeOverlapHit",
        "PhysicsShapeOverlapQuery",
        "PhysicsSimulationMode",
        "PhysicsTriggerEvent",
        "PhysicsTriggerEventKind",
        "PhysicsWorldStepPlan",
        "PhysicsWorldSyncState",
    ] {
        assert!(
            physics_mod.contains(required),
            "physics framework root should keep structural export `{required}`"
        );
    }

    for forbidden in [
        "pub enum PhysicsCombineRule",
        "pub struct PhysicsMaterialMetadata",
        "pub struct PhysicsSettings",
        "pub trait PhysicsManager",
        "impl Default for PhysicsSettings",
        "impl Default for PhysicsWorldSyncState",
    ] {
        assert!(
            !physics_mod.contains(forbidden),
            "physics framework root should not keep implementation detail `{forbidden}`"
        );
    }
}

#[test]
fn animation_framework_root_stays_structural_after_folder_split() {
    let animation_mod = include_str!("../animation/mod.rs");

    for required in [
        "mod graph_clip_instance;",
        "mod avatar_mask;",
        "mod event;",
        "mod gpu_skinning;",
        "mod graph_blend_mode;",
        "mod graph_evaluation;",
        "mod manager;",
        "mod parameter_map;",
        "mod parameter_value;",
        "mod playback_settings;",
        "mod pose_bone;",
        "mod pose_output;",
        "mod pose_source;",
        "mod sequence_apply_report;",
        "mod state_machine_evaluation;",
        "mod tick;",
        "mod track_path;",
        "mod track_path_error;",
        "AnimationAvatarMask",
        "AnimationEventRecord",
        "AnimationGpuSkinningReadiness",
        "AnimationSkinningBackend",
        "AnimationGraphClipInstance",
        "AnimationGraphBlendMode",
        "AnimationGraphEvaluation",
        "AnimationManager",
        "AnimationParameterMap",
        "AnimationParameterValue",
        "AnimationPlaybackSettings",
        "AnimationPoseBone",
        "AnimationPoseOutput",
        "AnimationPoseSource",
        "AnimationSequenceApplyReport",
        "AnimationStateMachineEvaluation",
        "AnimationTickReport",
        "AnimationTickRequest",
        "AnimationTrackPath",
        "AnimationTrackPathError",
    ] {
        assert!(
            animation_mod.contains(required),
            "animation framework root should keep structural export `{required}`"
        );
    }

    for forbidden in [
        "pub enum AnimationParameterValue",
        "pub struct AnimationTrackPath",
        "pub struct AnimationPlaybackSettings",
        "pub struct AnimationGraphClipInstance",
        "pub trait AnimationManager",
        "impl Default for AnimationPlaybackSettings",
    ] {
        assert!(
            !animation_mod.contains(forbidden),
            "animation framework root should not keep implementation detail `{forbidden}`"
        );
    }
}

#[test]
fn net_framework_root_stays_structural_after_folder_split() {
    let net_mod = include_str!("../net/mod.rs");

    for required in [
        "mod endpoint;",
        "mod error;",
        "mod manager;",
        "mod packet;",
        "mod socket_id;",
        "NetEndpoint",
        "NetError",
        "NetManager",
        "NetPacket",
        "NetSocketId",
    ] {
        assert!(
            net_mod.contains(required),
            "net framework root should keep structural export `{required}`"
        );
    }

    for forbidden in [
        "pub struct NetEndpoint",
        "pub enum NetError",
        "pub trait NetManager",
        "pub struct NetPacket",
        "pub struct NetSocketId",
        "impl NetSocketId",
    ] {
        assert!(
            !net_mod.contains(forbidden),
            "net framework root should not keep implementation detail `{forbidden}`"
        );
    }
}

#[test]
fn render_frame_extract_roundtrip_preserves_split_light_lists() {
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: vec![RenderDirectionalLightSnapshot {
                node_id: 10,
                light_id: 10,
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
                direction: Vec3::new(-0.4, -1.0, -0.2),
                color: Vec3::new(1.0, 0.9, 0.8),
                intensity: 3.0,
                shadow: None,
            }],
            point_lights: vec![RenderPointLightSnapshot {
                node_id: 20,
                light_id: 20,
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
                position: Vec3::new(2.0, 3.0, 4.0),
                color: Vec3::new(0.2, 0.6, 1.0),
                intensity: 4.5,
                range: 9.0,
                shadow: None,
            }],
            spot_lights: vec![RenderSpotLightSnapshot {
                node_id: 30,
                light_id: 30,
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
                position: Vec3::new(-1.0, 5.0, 2.0),
                direction: Vec3::new(0.0, -1.0, 0.3),
                color: Vec3::new(1.0, 0.7, 0.2),
                intensity: 8.0,
                range: 14.0,
                inner_angle_radians: 0.25,
                outer_angle_radians: 0.5,
                shadow: None,
            }],
            ambient_lights: vec![RenderAmbientLightSnapshot {
                color: Vec3::new(0.05, 0.06, 0.07),
                intensity: 0.2,
                renderer_degraded: true,
                degradation_reason: Some(
                    "ambient light renderer path is deferred after M5A".to_string(),
                ),
            }],
            rect_lights: vec![RenderRectLightSnapshot {
                node_id: 40,
                light_id: 40,
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
                position: Vec3::new(1.0, 2.0, 3.0),
                direction: Vec3::new(0.0, -1.0, 0.0),
                color: Vec3::new(1.0, 0.8, 0.6),
                intensity: 6.0,
                range: 12.0,
                size: Vec2::new(2.0, 0.5),
                shadow: None,
                renderer_degraded: true,
                degradation_reason: Some(
                    "rect light renderer path is deferred after M5A".to_string(),
                ),
            }],
        },
        overlays: RenderOverlayExtract::default(),
        environment: crate::core::framework::render::EnvironmentExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: true,
            skybox_enabled: true,
            fallback_skybox: FallbackSkyboxKind::ProceduralGradient,
            clear_color: Vec4::new(0.1, 0.2, 0.3, 1.0),
        },
        virtual_geometry_debug: None,
    };

    let extract = RenderFrameExtract::from_snapshot(WorldHandle::new(7).into(), snapshot.clone());

    assert_eq!(
        extract.lighting.directional_lights,
        snapshot.scene.directional_lights
    );
    assert_eq!(extract.lighting.point_lights, snapshot.scene.point_lights);
    assert_eq!(extract.lighting.spot_lights, snapshot.scene.spot_lights);
    assert_eq!(
        extract.lighting.ambient_lights,
        snapshot.scene.ambient_lights
    );
    assert_eq!(extract.lighting.rect_lights, snapshot.scene.rect_lights);

    let roundtrip = extract.to_scene_snapshot();
    assert_eq!(
        roundtrip.scene.directional_lights,
        snapshot.scene.directional_lights
    );
    assert_eq!(roundtrip.scene.point_lights, snapshot.scene.point_lights);
    assert_eq!(roundtrip.scene.spot_lights, snapshot.scene.spot_lights);
    assert_eq!(
        roundtrip.scene.ambient_lights,
        snapshot.scene.ambient_lights
    );
    assert_eq!(roundtrip.scene.rect_lights, snapshot.scene.rect_lights);
}

#[test]
fn render_product_pbr_lighting_extract_carries_ambient_and_rect_degradation_contracts() {
    let mut extract = RenderFrameExtract::from_snapshot(
        WorldHandle::new(8).into(),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            environment: crate::core::framework::render::EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    );
    extract
        .lighting
        .ambient_lights
        .push(RenderAmbientLightSnapshot {
            color: Vec3::new(0.1, 0.2, 0.3),
            intensity: 0.4,
            renderer_degraded: true,
            degradation_reason: Some(
                "ambient light renderer path is deferred after M5A".to_string(),
            ),
        });
    extract.lighting.rect_lights.push(RenderRectLightSnapshot {
        node_id: 80,
        light_id: 80,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
        position: Vec3::new(1.0, 2.0, 3.0),
        direction: Vec3::new(0.0, -1.0, 0.0),
        color: Vec3::new(1.0, 0.8, 0.6),
        intensity: 5.0,
        range: 12.0,
        size: Vec2::new(2.0, 0.5),
        shadow: None,
        renderer_degraded: true,
        degradation_reason: Some("rect light renderer path is deferred after M5A".to_string()),
    });

    assert_eq!(extract.lighting.ambient_lights.len(), 1);
    assert_eq!(extract.lighting.rect_lights.len(), 1);
    assert!(extract.lighting.ambient_lights[0].renderer_degraded);
    assert!(extract.lighting.rect_lights[0].renderer_degraded);
    assert!(extract.lighting.ambient_lights[0]
        .degradation_reason
        .as_deref()
        .unwrap()
        .contains("deferred"));
    assert!(extract.lighting.rect_lights[0]
        .degradation_reason
        .as_deref()
        .unwrap()
        .contains("deferred"));
}

#[test]
fn hybrid_gi_extract_defaults_to_public_settings_and_empty_internal_fixture() {
    let extract = RenderHybridGiExtract::default();

    assert!(!extract.enabled);
    assert_eq!(extract.quality, RenderHybridGiQuality::Medium);
    assert_eq!(extract.trace_budget, 0);
    assert_eq!(extract.card_budget, 0);
    assert_eq!(extract.voxel_budget, 0);
    assert_eq!(extract.debug_view, RenderHybridGiDebugView::None);
    assert_eq!(extract.probe_budget, 0);
    assert_eq!(extract.tracing_budget, 0);
    assert!(extract.probes.is_empty());
    assert!(extract.trace_regions.is_empty());
    assert!(!RenderFeatureQualitySettings::default().hybrid_global_illumination);
}

#[test]
fn render_profile_default_bundle_enables_basic_products_without_advanced_paths() {
    let bundle = RenderProfileBundle::default_render();

    assert_eq!(bundle.profile(), RenderProductProfile::DefaultRender);
    assert!(bundle.enables(RenderProductProfile::Render2d));
    assert!(bundle.enables(RenderProductProfile::Render3d));
    assert!(bundle.enables(RenderProductProfile::Ui));
    assert!(!bundle.enables(RenderProductProfile::AdvancedRender));
    assert!(!bundle.enables(RenderProductProfile::SolariExperimental));
    assert!(!bundle.has_feature(RenderProductFeature::VirtualGeometry));
    assert!(!bundle.has_feature(RenderProductFeature::HybridGlobalIllumination));
    assert!(!bundle.has_feature(RenderProductFeature::Solari));
    assert!(bundle.validate().is_ok());
}

#[test]
fn render_profile_validation_rejects_missing_2d_dependencies() {
    let bundle = RenderProfileBundle::new(RenderProductProfile::Render2d).with_features(
        RenderProfileBundle::render_2d().features_without(RenderProductFeature::Sprite),
    );

    assert_eq!(
        bundle.validate(),
        Err(RenderProfileValidationError::MissingRequiredFeature {
            profile: RenderProductProfile::Render2d,
            feature: RenderProductFeature::Sprite,
        })
    );
}

#[test]
fn render_profile_validation_rejects_missing_3d_dependencies() {
    let bundle = RenderProfileBundle::new(RenderProductProfile::Render3d).with_features(
        RenderProfileBundle::render_3d().features_without(RenderProductFeature::Pbr),
    );

    assert_eq!(
        bundle.validate(),
        Err(RenderProfileValidationError::MissingRequiredFeature {
            profile: RenderProductProfile::Render3d,
            feature: RenderProductFeature::Pbr,
        })
    );
}

#[test]
fn render_profile_validation_rejects_missing_ui_dependencies() {
    let bundle = RenderProfileBundle::new(RenderProductProfile::Ui).with_features(
        RenderProfileBundle::ui().features_without(RenderProductFeature::RenderTarget),
    );

    assert_eq!(
        bundle.validate(),
        Err(RenderProfileValidationError::MissingRequiredFeature {
            profile: RenderProductProfile::Ui,
            feature: RenderProductFeature::RenderTarget,
        })
    );
}

#[test]
fn render_profile_validation_rejects_unsatisfied_advanced_capabilities() {
    let bundle = RenderProfileBundle::advanced_render();
    let capabilities = RenderStats::default().capabilities;

    assert_eq!(
        bundle.validate_capabilities(&capabilities),
        Err(RenderProfileValidationError::MissingBackendCapability {
            profile: RenderProductProfile::AdvancedRender,
            detail: RenderCapabilityMismatchDetail::new(RenderCapabilityKind::VirtualGeometry),
        })
    );
}
