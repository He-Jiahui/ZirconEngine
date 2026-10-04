// 经停机轨道编辑入口核对历史帧预算，拒绝无界延迟声明，不执行混音渲染。
use super::super::super::*;

#[test]
fn mixer_graph_rejects_unbounded_track_delay_before_render() {
    let sound = DefaultSoundManager::default();
    let mut master = SoundTrackDescriptor::master();
    master.controls.delay_frames = 1_000_000;

    assert!(sound
        .add_or_update_track(master)
        .unwrap_err()
        .to_string()
        .contains("history budget"));
}
