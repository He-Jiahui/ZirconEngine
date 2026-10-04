use super::*;
#[test]
fn explicit_gpu_profile_submits_full_current_stream_without_relabeling_damage() {
    let damage = FrameRect {
        x: 4.0,
        y: 8.0,
        width: 12.0,
        height: 16.0,
    };
    assert!(observed_stream_damage(Some(&damage), true, true).is_none());
    assert_eq!(
        observed_stream_damage(Some(&damage), true, false),
        Some(&damage)
    );
    assert!(observed_stream_damage(Some(&damage), false, false).is_none());
}
