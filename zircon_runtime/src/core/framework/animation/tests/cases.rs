use crate::core::framework::scene::WorldHandle;
use crate::core::resource::{AssetReference, ResourceLocator};

use super::*;

#[test]
fn avatar_mask_filters_exact_leaf_and_excluded_targets() {
    let mask = AnimationAvatarMask {
        id: "upper_body".to_string(),
        included_target_ids: vec!["Root/Spine".to_string(), "LeftArm".to_string()],
        excluded_target_ids: vec!["Root/Spine/Head".to_string()],
        weight: 1.25,
    };

    assert!(mask.allows_target("Root/Spine"));
    assert!(mask.allows_target("LeftArm"));
    assert!(mask.allows_target("Root/LeftArm"));
    assert!(!mask.allows_target("Root/Spine/Head"));
    assert!(!mask.allows_target("RightLeg"));
    assert_eq!(mask.normalized_weight(), 1.0);
}

#[test]
fn animation_tick_contract_records_work_events_and_sanitized_delta() {
    let request = AnimationTickRequest::new(WorldHandle::new(7), f32::NAN).with_frame_index(99);
    assert_eq!(request.world, WorldHandle::new(7));
    assert_eq!(request.frame_index, 99);
    assert_eq!(request.sanitized_delta_seconds(), 0.0);

    let event = AnimationEventRecord::new(42, "footstep")
        .with_target_id("Root/Foot.L")
        .with_payload("stone")
        .at_times(0.25, 1.25);
    let mut report = AnimationTickReport::new(WorldHandle::new(7)).with_event(event);
    report.sampled_clips = 1;
    report.posed_entities = 1;

    assert!(report.has_runtime_work());
    assert_eq!(report.emitted_events[0].entity, 42);
    assert_eq!(
        report.emitted_events[0].target_id.as_deref(),
        Some("Root/Foot.L")
    );
}

#[test]
fn gpu_skinning_readiness_requires_enabled_gpu_resources() {
    let disabled = AnimationGpuSkinningReadiness::default();
    assert!(!disabled.ready_for_gpu_skinning());

    let missing = AnimationGpuSkinningReadiness {
        enabled: true,
        backend: AnimationSkinningBackend::Gpu,
        skinned_entities: 2,
        mesh_targets: 1,
        bone_palette_bytes: 4096,
        morph_target_bytes: 1024,
        missing_gpu_resources: vec!["bone-palette-buffer".to_string()],
        diagnostics: Vec::new(),
    };
    assert!(!missing.ready_for_gpu_skinning());

    let ready = AnimationGpuSkinningReadiness {
        missing_gpu_resources: Vec::new(),
        ..missing
    };
    assert!(ready.ready_for_gpu_skinning());
}

#[test]
fn timeline_track_masks_and_clip_status_sanitize_contract_values() {
    let masked = AnimationTimelineTrackDescriptor::bone_transform("Root/Spine/Hand", 3)
        .with_avatar_mask(AnimationAvatarMask {
            id: "upper_body".to_string(),
            included_target_ids: vec!["Spine".to_string(), "Hand".to_string()],
            excluded_target_ids: vec!["Root/Spine/Head".to_string()],
            weight: 0.75,
        });

    assert!(masked.allows_target("Hand"));
    assert!(masked.allows_target("Root/Spine/Hand"));
    assert!(!masked.allows_target("Root/Spine/Head"));
    assert!(!masked.clone().muted(true).allows_target("Hand"));

    let clip = AnimationTimelineClipDescriptor {
        start_seconds: -1.0,
        duration_seconds: f32::NAN,
        playback_speed: f32::INFINITY,
        weight: 1.5,
        ..AnimationTimelineClipDescriptor::default()
    };
    assert_eq!(clip.sanitized_start_seconds(), 0.0);
    assert_eq!(clip.sanitized_duration_seconds(), 0.0);
    assert_eq!(clip.sanitized_playback_speed(), 0.0);
    assert_eq!(clip.normalized_weight(), 1.0);
}

#[test]
fn runtime_status_reports_player_rig_and_gpu_readiness() {
    let world = WorldHandle::new(11);
    let mut player = AnimationPlayerRuntimeStatus::new(world, 7, AnimationPlayerKind::Clip)
        .with_source(asset_reference("res://animation/hero.clip.zranim"))
        .with_diagnostic("sampled");
    player.state = AnimationPlayerRuntimeState::Playing;
    player.time_seconds = f32::NAN;
    player.playback_speed = -2.0;
    player.weight = -1.0;

    let rig = AnimationRigRuntimeStatus::new(world, 7)
        .with_skeleton(asset_reference("res://animation/hero.skeleton.zranim"), 64);
    let ready_gpu = AnimationGpuSkinningReadiness {
        enabled: true,
        backend: AnimationSkinningBackend::Gpu,
        skinned_entities: 1,
        mesh_targets: 1,
        bone_palette_bytes: 4096,
        morph_target_bytes: 0,
        missing_gpu_resources: Vec::new(),
        diagnostics: Vec::new(),
    };
    let mut rig = rig;
    rig.posed_bone_count = 32;
    rig.gpu_skinning = ready_gpu;

    let status = AnimationRuntimeStatus::new(world)
        .with_player(player)
        .with_rig(rig)
        .with_diagnostic("ok");

    assert!(status.has_runtime_work());
    assert_eq!(status.active_player_count(), 1);
    assert_eq!(status.posed_rig_count(), 1);
    assert_eq!(status.gpu_ready_rig_count(), 1);
    assert_eq!(status.players[0].sanitized_time_seconds(), 0.0);
    assert_eq!(status.players[0].sanitized_playback_speed(), 0.0);
    assert_eq!(status.players[0].normalized_weight(), 0.0);
    assert_eq!(status.rigs[0].pose_coverage(), 0.5);
    assert!(status.rigs[0].ready_for_pose());

    let roundtrip =
        serde_json::from_value::<AnimationRuntimeStatus>(serde_json::to_value(&status).unwrap())
            .unwrap();
    assert_eq!(roundtrip, status.sanitized_snapshot());
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(ResourceLocator::parse(uri).unwrap())
}
