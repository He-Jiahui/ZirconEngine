use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectId, SoundEffectKind, SoundGainEffect, SoundLimiterEffect,
    SoundMixerGraph, SoundMixerPresetDescriptor, SoundReverbEffect, SoundTrackDescriptor,
    SoundTrackId, SoundTrackSend,
};

use crate::SoundConfig;

pub const DEFAULT_MIXER_PRESET_LOCATOR: &str = "sound://mixer/default";
pub const MUSIC_SFX_MIXER_PRESET_LOCATOR: &str = "sound://mixer/music_sfx";
pub const SPATIAL_ROOM_MIXER_PRESET_LOCATOR: &str = "sound://mixer/spatial_room";

pub(crate) fn built_in_mixer_presets(config: &SoundConfig) -> Vec<SoundMixerPresetDescriptor> {
    vec![
        SoundMixerPresetDescriptor::new(
            DEFAULT_MIXER_PRESET_LOCATOR,
            "Default",
            default_graph(config),
        ),
        SoundMixerPresetDescriptor::new(
            MUSIC_SFX_MIXER_PRESET_LOCATOR,
            "Music and SFX",
            music_sfx_graph(config),
        ),
        SoundMixerPresetDescriptor::new(
            SPATIAL_ROOM_MIXER_PRESET_LOCATOR,
            "Spatial Room",
            spatial_room_graph(config),
        ),
    ]
}

fn default_graph(config: &SoundConfig) -> SoundMixerGraph {
    let mut graph = SoundMixerGraph::default_stereo(config.sample_rate_hz);
    graph.channel_count = config.channel_count.max(1);
    graph
}

fn music_sfx_graph(config: &SoundConfig) -> SoundMixerGraph {
    let mut graph = default_graph(config);
    graph.tracks = vec![
        SoundTrackDescriptor::master(),
        SoundTrackDescriptor::child(SoundTrackId::new(2), "Music"),
        SoundTrackDescriptor::child(SoundTrackId::new(3), "SFX"),
        SoundTrackDescriptor::child(SoundTrackId::new(4), "Ambience"),
    ];
    graph.tracks[0].effects.push(SoundEffectDescriptor::new(
        SoundEffectId::new(1),
        "Master Limiter",
        SoundEffectKind::Limiter(SoundLimiterEffect { ceiling: 1.0 }),
    ));
    graph.tracks[1].effects.push(SoundEffectDescriptor::new(
        SoundEffectId::new(2),
        "Music Trim",
        SoundEffectKind::Gain(SoundGainEffect { gain: 0.9 }),
    ));
    graph
}

fn spatial_room_graph(config: &SoundConfig) -> SoundMixerGraph {
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
