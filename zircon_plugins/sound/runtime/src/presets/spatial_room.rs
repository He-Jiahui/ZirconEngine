//! 房间预设在音乐图上增加混响轨和发送路由；当前后端若拒绝效果器，调用方只能将它当作作者模板。
use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectId, SoundEffectKind, SoundMixerGraph, SoundReverbEffect,
    SoundTrackDescriptor, SoundTrackId, SoundTrackSend,
};

use crate::SoundConfig;

use super::music_sfx::music_sfx_graph;

pub(crate) fn spatial_room_graph(config: &SoundConfig) -> SoundMixerGraph {
    let mut graph = music_sfx_graph(config);
    let mut room = SoundTrackDescriptor::child(SoundTrackId::new(5), "Room Reverb");
    room.effects.push(SoundEffectDescriptor::new(
        SoundEffectId::new(3),
        "Room Reverb",
        SoundEffectKind::Reverb(SoundReverbEffect {
            room_size: 0.65,
            damping: 0.45,
            pre_delay_frames: 12,
            tail_frames: 96,
        }),
    ));
    graph.tracks.push(room);
    if let Some(sfx) = graph
        .tracks
        .iter_mut()
        .find(|track| track.id == SoundTrackId::new(3))
    {
        sfx.sends.push(SoundTrackSend {
            target: SoundTrackId::new(5),
            gain: 0.25,
            pre_effects: false,
        });
    }
    if let Some(ambience) = graph
        .tracks
        .iter_mut()
        .find(|track| track.id == SoundTrackId::new(4))
    {
        ambience.sends.push(SoundTrackSend {
            target: SoundTrackId::new(5),
            gain: 0.4,
            pre_effects: false,
        });
    }
    graph
}
