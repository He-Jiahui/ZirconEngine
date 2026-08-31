use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset, AnimationTrackPath,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};

use super::AnimationEditorSession;

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after the Unix epoch")
        .as_nanos();
    PathBuf::from("E:/ZirconBuilds/zircon_editor_animation_tests")
        .join(format!("{prefix}_{unique}"))
}

fn write_sequence_asset(path: &Path) -> AnimationTrackPath {
    let track_path = AnimationTrackPath::new(
        EntityPath::parse("Root/Hero").expect("fixture entity path must be valid"),
        ComponentPropertyPath::parse("Transform.translation")
            .expect("fixture property path must be valid"),
    );
    let asset = AnimationSequenceAsset {
        name: Some("Hero Sequence".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            target_id: None,
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Hermite,
                    keys: vec![
                        AnimationChannelKeyAsset {
                            time_seconds: 0.0,
                            value: AnimationChannelValueAsset::Vec3([1.0, 2.0, 3.0]),
                            in_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 1.0, 2.0])),
                            out_tangent: Some(AnimationChannelValueAsset::Vec3([3.0, 4.0, 5.0])),
                        },
                        AnimationChannelKeyAsset {
                            time_seconds: 2.0,
                            value: AnimationChannelValueAsset::Vec3([4.0, 5.0, 6.0]),
                            in_tangent: Some(AnimationChannelValueAsset::Vec3([6.0, 7.0, 8.0])),
                            out_tangent: Some(AnimationChannelValueAsset::Vec3([9.0, 10.0, 11.0])),
                        },
                    ],
                },
            }],
        }],
    };
    fs::create_dir_all(path.parent().expect("fixture path must have a parent"))
        .expect("fixture directory must be created");
    fs::write(path, asset.to_bytes().expect("fixture must encode"))
        .expect("fixture bytes must write");
    track_path
}

#[test]
fn sequence_selection_is_transient_and_does_not_mutate_core_source_bytes() {
    let path = unique_temp_dir("zircon_animation_session_selection").join("hero.sequence.zranim");
    let track_path = write_sequence_asset(&path);
    let mut session = AnimationEditorSession::from_path(&path).expect("fixture must open");
    let before = session.document_bytes().expect("source must serialize");

    assert!(
        session
            .select_timeline_span(&track_path, 4, 28)
            .expect("selection must apply")
    );

    assert_eq!(
        session.document_bytes().expect("source must serialize"),
        before,
        "selection belongs to the UI session and must not create another persisted source state"
    );
    let _ = fs::remove_dir_all(path.parent().expect("fixture path must have a parent"));
}

#[test]
fn sequence_timeline_projection_reads_core_source_and_transient_transport() {
    let path = unique_temp_dir("zircon_animation_session_timeline").join("hero.sequence.zranim");
    let track_path = write_sequence_asset(&path);
    let mut session = AnimationEditorSession::from_path(&path).expect("fixture must open");
    session.set_timeline_range(10, 20).unwrap();
    session.scrub_timeline(15).unwrap();
    session.set_playback(true, true, 1.25).unwrap();
    session.select_timeline_span(&track_path, 10, 20).unwrap();

    let timeline = session
        .timeline_foundation()
        .expect("timeline must project");
    assert_eq!(timeline.tracks.len(), 1);
    assert!((timeline.playhead - 0.5).abs() < 1.0e-6);
    assert!(timeline.playback.playing);
    assert_eq!(timeline.playback.rate, 1.25);
    let curves = session
        .curve_foundation()
        .expect("curve projection must succeed");
    assert_eq!(curves.curves.len(), 3);
    assert_eq!(curves.curves[2].keys[1].in_tangent, Some(8.0));
    let _ = fs::remove_dir_all(path.parent().expect("fixture path must have a parent"));
}

#[test]
fn non_finite_playback_speed_preserves_transient_transport() {
    let path = unique_temp_dir("zircon_animation_session_playback").join("hero.sequence.zranim");
    write_sequence_asset(&path);
    let mut session = AnimationEditorSession::from_path(&path).expect("fixture must open");

    assert!(
        !session
            .set_playback(true, true, f32::NAN)
            .expect("invalid speed must be a no-op")
    );
    assert_eq!(
        session.pane_presentation().playback_label,
        "Paused • loop=false • speed=1.00"
    );
    let _ = fs::remove_dir_all(path.parent().expect("fixture path must have a parent"));
}
