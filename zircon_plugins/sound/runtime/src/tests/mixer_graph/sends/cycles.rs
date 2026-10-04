// 经停机发送编辑入口核对图校验拒绝循环路由，不启动 Kira 后端。
use super::super::super::*;

#[test]
fn mixer_graph_rejects_track_send_cycles() {
    let sound = DefaultSoundManager::default();
    let a = SoundTrackId::new(2);
    let b = SoundTrackId::new(3);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(a, "A"))
        .unwrap();
    sound
        .add_or_update_track(SoundTrackDescriptor::child(b, "B"))
        .unwrap();
    sound
        .add_or_update_track_send(
            a,
            SoundTrackSend {
                target: b,
                gain: 1.0,
                pre_effects: false,
            },
        )
        .unwrap();

    let error = sound
        .add_or_update_track_send(
            b,
            SoundTrackSend {
                target: a,
                gain: 1.0,
                pre_effects: false,
            },
        )
        .unwrap_err();

    assert!(error.to_string().contains("cycle"));
}
