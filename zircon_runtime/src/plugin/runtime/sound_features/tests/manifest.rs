use super::join_string_parts;

#[test]
fn exact_sound_identifier_join_preserves_parts() {
    assert_eq!(
        join_string_parts(&["sound.", "timeline_animation_track", ".runtime"]),
        "sound.timeline_animation_track.runtime"
    );
}
