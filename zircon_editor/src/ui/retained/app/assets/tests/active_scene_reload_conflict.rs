use super::active_scene_reload_display_subject;
use crate::core::notifications::MAX_DECISION_DISPLAY_SUBJECT_BYTES;

#[test]
fn reload_conflict_display_subject_preserves_a_bounded_utf8_tail() {
    let uri = format!("res://scenes/{}final.scene.toml", "场景/".repeat(80));

    let subject = active_scene_reload_display_subject(&uri);

    assert!(subject.len() <= MAX_DECISION_DISPLAY_SUBJECT_BYTES);
    assert!(subject.starts_with("..."));
    assert!(subject.ends_with("final.scene.toml"));
}
