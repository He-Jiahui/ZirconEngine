//! 预设目录向作者展示默认、音乐及空间房间图；是否可在当前 Kira M1 后端启动由图编译另行决定。
use zircon_runtime::core::framework::sound::SoundMixerPresetDescriptor;

use crate::SoundConfig;

use super::default::default_graph;
use super::locators::{
    DEFAULT_MIXER_PRESET_LOCATOR, MUSIC_SFX_MIXER_PRESET_LOCATOR, SPATIAL_ROOM_MIXER_PRESET_LOCATOR,
};
use super::music_sfx::music_sfx_graph;
use super::spatial_room::spatial_room_graph;

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
            // TODO: [CR-SOUND-AUDIT-0006] 确认带效果器的音乐/房间预设是否应可启动；目录会提供它们，但 Kira M1 编译拒绝任何 effects，停机应用可先成功、启动才报错。
            music_sfx_graph(config),
        ),
        SoundMixerPresetDescriptor::new(
            SPATIAL_ROOM_MIXER_PRESET_LOCATOR,
            "Spatial Room",
            spatial_room_graph(config),
        ),
    ]
}
