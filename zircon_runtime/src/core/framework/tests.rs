use crate::core::math::{UVec2, Vec3, Vec4};

use super::{
    animation::{AnimationParameterValue, AnimationPlaybackSettings, AnimationTrackPath},
    input::{InputButton, InputEvent, InputEventRecord, InputSnapshot},
    net::{NetEndpoint, NetError, NetPacket, NetSocketId},
    physics::{PhysicsCombineRule, PhysicsMaterialMetadata, PhysicsSettings},
    render::{
        CapturedFrame, FallbackSkyboxKind, FrameHistoryHandle, PreviewEnvironmentExtract,
        RenderDirectionalLightSnapshot, RenderFeatureQualitySettings, RenderFrameExtract,
        RenderFrameworkError, RenderHybridGiDebugView, RenderHybridGiExtract,
        RenderHybridGiQuality, RenderOverlayExtract, RenderPipelineHandle,
        RenderPointLightSnapshot, RenderQualityProfile, RenderSceneGeometryExtract,
        RenderSceneSnapshot, RenderSpotLightSnapshot, RenderStats, RenderViewportDescriptor,
        RenderViewportHandle, RenderingBackendInfo, ViewportCameraSnapshot,
    },
    scene::{ComponentPropertyPath, EntityPath, LevelSummary, Mobility, WorldHandle},
};

#[test]
fn framework_contract_types_are_constructible() {
    let viewport = RenderViewportHandle::new(7);
    let pipeline = RenderPipelineHandle::new(11);
    let descriptor = RenderViewportDescriptor::new(UVec2::new(320, 240));
    let profile =
        RenderQualityProfile::new("editor-high").with_pipeline_asset(RenderPipelineHandle::new(11));
    let frame = CapturedFrame::new(320, 240, vec![0; 320 * 240 * 4], 3);
    let stats = RenderStats::default();
    let backend = RenderingBackendInfo {
        backend_name: "wgpu".into(),
        supports_runtime_preview: true,
        supports_shared_texture_viewports: true,
    };
    let input = InputSnapshot {
        cursor_position: [12.0, 24.0],
        pressed_buttons: vec![InputButton::MouseLeft],
        wheel_accumulator: 1.0,
    };
    let event = InputEventRecord {
        sequence: 1,
        timestamp_millis: 2,
        event: InputEvent::ButtonPressed(InputButton::MouseRight),
    };
    let entity_path = EntityPath::parse("Root/Hero").unwrap();
    let property_path = ComponentPropertyPath::parse("Transform.translation").unwrap();
    let track_path = AnimationTrackPath::new(entity_path.clone(), property_path.clone());
    let playback = AnimationPlaybackSettings::default();
    let socket = NetSocketId::new(5);
    let endpoint = NetEndpoint::new("127.0.0.1", 9000);
    let packet = NetPacket {
        source: endpoint.clone(),
        payload: vec![1, 2, 3],
    };
    let material = PhysicsMaterialMetadata::default();
    let physics = PhysicsSettings::default();
    let level = LevelSummary {
        handle: WorldHandle::new(42),
        entity_count: 5,
        active_camera: Some(4),
    };

    assert_eq!(viewport.raw(), 7);
    assert_eq!(pipeline.raw(), 11);
    assert_eq!(descriptor.size, UVec2::new(320, 240));
    assert_eq!(
        profile.pipeline_override,
        Some(RenderPipelineHandle::new(11))
    );
    assert_eq!(frame.generation, 3);
    assert_eq!(stats.active_viewports, 0);
    assert_eq!(backend.backend_name, "wgpu");
    assert_eq!(input.cursor_position, [12.0, 24.0]);
    assert_eq!(event.sequence, 1);
    assert_eq!(entity_path.to_string(), "Root/Hero");
    assert_eq!(property_path.to_string(), "Transform.translation");
    assert_eq!(track_path.to_string(), "Root/Hero:Transform.translation");
    assert_eq!(socket.raw(), 5);
    assert_eq!(endpoint.host, "127.0.0.1");
    assert_eq!(endpoint.port, 9000);
    assert_eq!(packet.payload, vec![1, 2, 3]);
    assert_eq!(
        NetError::UnknownSocket { socket },
        NetError::UnknownSocket { socket }
    );
    assert_eq!(
        AnimationParameterValue::Trigger,
        AnimationParameterValue::Trigger
    );
    assert!(playback.enabled && playback.property_tracks);
    assert_eq!(material.friction_combine, PhysicsCombineRule::Average);
    assert_eq!(physics.fixed_hz, 60);
    assert_eq!(level.handle.get(), 42);
    assert_eq!(Mobility::default(), Mobility::Dynamic);
    assert_eq!(
        RenderFrameworkError::UnknownPipeline { pipeline: 9 },
        RenderFrameworkError::UnknownPipeline { pipeline: 9 }
    );

    let history = FrameHistoryHandle::new(19);
    assert_eq!(history.raw(), 19);
}

#[test]
fn physics_framework_root_stays_structural_after_folder_split() {
    let physics_mod = include_str!("physics/mod.rs");

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
        "mod ray_cast_hit;",
        "mod ray_cast_query;",
        "mod settings;",
        "mod simulation_mode;",
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
        "PhysicsRayCastHit",
        "PhysicsRayCastQuery",
        "PhysicsSettings",
        "PhysicsSimulationMode",
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
    let animation_mod = include_str!("animation/mod.rs");

    for required in [
        "mod graph_clip_instance;",
        "mod graph_evaluation;",
        "mod manager;",
        "mod parameter_map;",
        "mod parameter_value;",
        "mod playback_settings;",
        "mod pose_bone;",
        "mod pose_output;",
        "mod pose_source;",
        "mod state_machine_evaluation;",
        "mod track_path;",
        "mod track_path_error;",
        "AnimationGraphClipInstance",
        "AnimationGraphEvaluation",
        "AnimationManager",
        "AnimationParameterMap",
        "AnimationParameterValue",
        "AnimationPlaybackSettings",
        "AnimationPoseBone",
        "AnimationPoseOutput",
        "AnimationPoseSource",
        "AnimationStateMachineEvaluation",
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
    let net_mod = include_str!("net/mod.rs");

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
                direction: Vec3::new(-0.4, -1.0, -0.2),
                color: Vec3::new(1.0, 0.9, 0.8),
                intensity: 3.0,
            }],
            point_lights: vec![RenderPointLightSnapshot {
                node_id: 20,
                position: Vec3::new(2.0, 3.0, 4.0),
                color: Vec3::new(0.2, 0.6, 1.0),
                intensity: 4.5,
                range: 9.0,
            }],
            spot_lights: vec![RenderSpotLightSnapshot {
                node_id: 30,
                position: Vec3::new(-1.0, 5.0, 2.0),
                direction: Vec3::new(0.0, -1.0, 0.3),
                color: Vec3::new(1.0, 0.7, 0.2),
                intensity: 8.0,
                range: 14.0,
                inner_angle_radians: 0.25,
                outer_angle_radians: 0.5,
            }],
        },
        overlays: RenderOverlayExtract::default(),
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

    let roundtrip = extract.to_scene_snapshot();
    assert_eq!(
        roundtrip.scene.directional_lights,
        snapshot.scene.directional_lights
    );
    assert_eq!(roundtrip.scene.point_lights, snapshot.scene.point_lights);
    assert_eq!(roundtrip.scene.spot_lights, snapshot.scene.spot_lights);
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
